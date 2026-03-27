mod managed_process;
mod tray_model;
#[cfg(test)]
mod tray_model_tests;

use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use keyring::Entry;
use managed_process::ManagedProcess;
use serde::{Deserialize, Serialize};
use sshtunnel_core::{
    models::{AuthKind, TunnelDefinition},
    ssh_launch::build_launch_plan,
};
use tauri::{
    menu::{Menu, MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    tray::TrayIconBuilder,
    Manager, State, WindowEvent,
};
use tauri_plugin_autostart::{Builder as AutostartBuilder, ManagerExt as AutostartExt};
use tray_model::{
    order_recent_tunnels, recent_tray_items, tray_action_id, tray_action_label, TrayTunnelItem,
};

const SERVICE_NAME: &str = "sshtunnel-manager";
const MAIN_WINDOW_LABEL: &str = "main";
const MAIN_TRAY_ID: &str = "main-tray";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
enum TunnelStatus {
    Idle,
    Connected,
    Error,
}

#[derive(Debug, Clone, Serialize)]
struct TunnelView {
    definition: TunnelDefinition,
    status: TunnelStatus,
    last_error: Option<String>,
    recent_log: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct AppSnapshot {
    tunnels: Vec<TunnelView>,
    ssh_available: bool,
    autostart_enabled: bool,
    config_path: String,
}

#[derive(Debug, Deserialize)]
struct SaveTunnelPayload {
    tunnel: TunnelDefinition,
    password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct StoredConfig {
    tunnels: Vec<TunnelDefinition>,
}

struct TunnelRuntime {
    process: Option<ManagedProcess>,
    last_error: Option<String>,
    recent_log: Vec<String>,
}

impl Default for TunnelRuntime {
    fn default() -> Self {
        Self {
            process: None,
            last_error: None,
            recent_log: Vec::new(),
        }
    }
}

#[derive(Default)]
struct InnerState {
    tunnels: Vec<TunnelDefinition>,
    runtimes: HashMap<String, TunnelRuntime>,
    recent_tunnel_ids: Vec<String>,
}

struct AppState {
    config_path: PathBuf,
    inner: Mutex<InnerState>,
}

impl AppState {
    fn new() -> Self {
        let config_path = resolve_config_path();
        let tunnels = load_config(&config_path).unwrap_or_default().tunnels;

        Self {
            config_path,
            inner: Mutex::new(InnerState {
                recent_tunnel_ids: tunnels.iter().map(|item| item.id.clone()).collect(),
                tunnels,
                runtimes: HashMap::new(),
            }),
        }
    }
}

#[tauri::command]
fn load_state(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<AppSnapshot, String> {
    snapshot(&state, &app)
}

#[tauri::command]
fn save_tunnel(
    app: tauri::AppHandle,
    payload: SaveTunnelPayload,
    state: State<'_, AppState>,
) -> Result<AppSnapshot, String> {
    let mut tunnel = payload.tunnel;
    if tunnel.id.trim().is_empty() {
        tunnel.id = generate_id(&tunnel.name);
    }

    if tunnel.auth_kind == AuthKind::Password {
        tunnel.private_key_path = None;
        tunnel.password_entry = Some(password_entry_name(&tunnel.id));
    } else {
        tunnel.password_entry = None;
    }

    tunnel.validate()?;

    {
        let mut inner = state
            .inner
            .lock()
            .map_err(|_| "state poisoned".to_string())?;
        let recent_id = tunnel.id.clone();

        if tunnel.auth_kind == AuthKind::Password {
            if let Some(password) = payload.password.as_deref() {
                if !password.is_empty() {
                    set_password(&tunnel.id, password)?;
                }
            }
        } else {
            let _ = delete_password(&tunnel.id);
        }

        match inner
            .tunnels
            .iter_mut()
            .find(|existing| existing.id == tunnel.id)
        {
            Some(existing) => *existing = tunnel,
            None => inner.tunnels.push(tunnel),
        }

        touch_recent_tunnel(&mut inner, &recent_id);
        persist_config(&state.config_path, &inner.tunnels)?;
        refresh_tray_menu(&app, &inner)?;
    }

    snapshot(&state, &app)
}

#[tauri::command]
fn delete_tunnel(
    app: tauri::AppHandle,
    id: String,
    state: State<'_, AppState>,
) -> Result<AppSnapshot, String> {
    {
        let mut inner = state
            .inner
            .lock()
            .map_err(|_| "state poisoned".to_string())?;
        disconnect_runtime(&mut inner, &id);
        inner.tunnels.retain(|item| item.id != id);
        inner.runtimes.remove(&id);
        remove_recent_tunnel(&mut inner, &id);
        let _ = delete_password(&id);
        persist_config(&state.config_path, &inner.tunnels)?;
        refresh_tray_menu(&app, &inner)?;
    }

    snapshot(&state, &app)
}

#[tauri::command]
fn connect_tunnel(
    app: tauri::AppHandle,
    id: String,
    state: State<'_, AppState>,
) -> Result<AppSnapshot, String> {
    ensure_ssh_available()?;

    let mut inner = state
        .inner
        .lock()
        .map_err(|_| "state poisoned".to_string())?;
    let tunnel = inner
        .tunnels
        .iter()
        .find(|item| item.id == id)
        .cloned()
        .ok_or_else(|| format!("unknown tunnel id: {id}"))?;

    tunnel.validate()?;
    disconnect_runtime(&mut inner, &id);

    if tunnel.auth_kind == AuthKind::Password {
        if get_password(&tunnel.id).is_err() {
            let runtime = inner.runtimes.entry(id).or_default();
            runtime.last_error = Some("credential is missing in the system keychain".into());
            push_log(runtime, "missing password credential");
            return snapshot_from_inner(&state.config_path, &app, &mut inner);
        }
    }

    let password = if tunnel.auth_kind == AuthKind::Password {
        Some(get_password(&tunnel.id)?)
    } else {
        None
    };
    let plan = build_launch_plan(&tunnel, password.as_deref())?;
    let process = ManagedProcess::spawn(plan)?;
    let pid = process.pid();
    let recent_id = id.clone();

    {
        let runtime = inner.runtimes.entry(id).or_default();
        runtime.process = Some(process);
        runtime.last_error = None;
        match pid {
            Some(pid) => push_log(runtime, &format!("spawned ssh process pid={pid}")),
            None => push_log(runtime, "spawned interactive ssh process"),
        }
    }
    touch_recent_tunnel(&mut inner, &recent_id);

    refresh_tray_menu(&app, &inner)?;
    snapshot_from_inner(&state.config_path, &app, &mut inner)
}

#[tauri::command]
fn disconnect_tunnel(
    app: tauri::AppHandle,
    id: String,
    state: State<'_, AppState>,
) -> Result<AppSnapshot, String> {
    let mut inner = state
        .inner
        .lock()
        .map_err(|_| "state poisoned".to_string())?;
    disconnect_runtime(&mut inner, &id);
    touch_recent_tunnel(&mut inner, &id);
    refresh_tray_menu(&app, &inner)?;
    snapshot_from_inner(&state.config_path, &app, &mut inner)
}

#[tauri::command]
fn set_autostart(
    app: tauri::AppHandle,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<AppSnapshot, String> {
    let autolaunch = app.autolaunch();
    if enabled {
        autolaunch.enable().map_err(|error| error.to_string())?;
    } else {
        autolaunch.disable().map_err(|error| error.to_string())?;
    }

    snapshot(&state, &app)
}

#[tauri::command]
fn reveal_config_path(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.config_path.display().to_string())
}

fn resolve_config_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("sshtunnel-manager").join("config.json")
}

fn load_config(path: &PathBuf) -> Result<StoredConfig, String> {
    if !path.exists() {
        return Ok(StoredConfig::default());
    }

    let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str(&content).map_err(|error| error.to_string())
}

fn persist_config(path: &PathBuf, tunnels: &[TunnelDefinition]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let body = serde_json::to_string_pretty(&StoredConfig {
        tunnels: tunnels.to_vec(),
    })
    .map_err(|error| error.to_string())?;

    fs::write(path, body).map_err(|error| error.to_string())
}

fn ensure_ssh_available() -> Result<(), String> {
    std::process::Command::new("ssh")
        .arg("-V")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|_| "system ssh binary was not found in PATH".to_string())
        .and_then(|status| {
            if status.success() {
                Ok(())
            } else {
                Err("system ssh binary returned a non-zero status".to_string())
            }
        })
}

fn password_entry_name(id: &str) -> String {
    format!("profile:{id}")
}

fn set_password(id: &str, password: &str) -> Result<(), String> {
    Entry::new(SERVICE_NAME, &password_entry_name(id))
        .map_err(|error| error.to_string())?
        .set_password(password)
        .map_err(|error| error.to_string())
}

fn get_password(id: &str) -> Result<String, String> {
    Entry::new(SERVICE_NAME, &password_entry_name(id))
        .map_err(|error| error.to_string())?
        .get_password()
        .map_err(|error| error.to_string())
}

fn delete_password(id: &str) -> Result<(), String> {
    Entry::new(SERVICE_NAME, &password_entry_name(id))
        .map_err(|error| error.to_string())?
        .delete_credential()
        .map_err(|error| error.to_string())
}

fn generate_id(name: &str) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);

    let slug = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    format!(
        "{}-{millis}",
        if slug.is_empty() { "tunnel" } else { &slug }
    )
}

fn push_log(runtime: &mut TunnelRuntime, entry: &str) {
    runtime.recent_log.push(entry.to_string());
    if runtime.recent_log.len() > 12 {
        let trim = runtime.recent_log.len() - 12;
        runtime.recent_log.drain(0..trim);
    }
}

fn touch_recent_tunnel(inner: &mut InnerState, id: &str) {
    inner.recent_tunnel_ids.retain(|item| item != id);
    inner.recent_tunnel_ids.insert(0, id.to_string());
}

fn remove_recent_tunnel(inner: &mut InnerState, id: &str) {
    inner.recent_tunnel_ids.retain(|item| item != id);
}

fn is_connected(inner: &InnerState, id: &str) -> bool {
    inner
        .runtimes
        .get(id)
        .and_then(|runtime| runtime.process.as_ref())
        .is_some()
}

fn recent_tray_menu_items(inner: &InnerState) -> Vec<TrayTunnelItem> {
    let ordered = order_recent_tunnels(&inner.tunnels, &inner.recent_tunnel_ids);
    recent_tray_items(ordered, |id| is_connected(inner, id))
}

fn disconnect_runtime(inner: &mut InnerState, id: &str) {
    if let Some(runtime) = inner.runtimes.get_mut(id) {
        let _ = flush_process_logs(runtime);
        if let Some(process) = runtime.process.as_mut() {
            let _ = process.kill();
            push_log(runtime, "stopped ssh process");
        }
        runtime.process = None;
        runtime.last_error = None;
    }
}

fn refresh_runtime(runtime: &mut TunnelRuntime) -> TunnelStatus {
    let _ = flush_process_logs(runtime);

    if let Some(process) = runtime.process.as_mut() {
        match process.try_wait() {
            Ok(Some(status)) => {
                runtime.process = None;
                if status.contains("exit status: 0") || status.contains("code: 0") {
                    runtime.last_error = None;
                    push_log(runtime, "ssh process exited");
                    TunnelStatus::Idle
                } else {
                    runtime.last_error = Some(format!("ssh exited with status {status}"));
                    push_log(runtime, &format!("ssh exited with status {status}"));
                    TunnelStatus::Error
                }
            }
            Ok(None) => TunnelStatus::Connected,
            Err(error) => {
                runtime.last_error = Some(error.to_string());
                push_log(runtime, &format!("failed to query process status: {error}"));
                TunnelStatus::Error
            }
        }
    } else if runtime.last_error.is_some() {
        TunnelStatus::Error
    } else {
        TunnelStatus::Idle
    }
}

fn flush_process_logs(runtime: &mut TunnelRuntime) -> Result<(), String> {
    if let Some(process) = runtime.process.as_ref() {
        for line in process.take_logs() {
            push_log(runtime, &line);
        }
    }

    Ok(())
}

fn autostart_enabled(app: &tauri::AppHandle) -> bool {
    app.autolaunch().is_enabled().unwrap_or(false)
}

fn snapshot(state: &AppState, app: &tauri::AppHandle) -> Result<AppSnapshot, String> {
    let mut inner = state
        .inner
        .lock()
        .map_err(|_| "state poisoned".to_string())?;
    snapshot_from_inner(&state.config_path, app, &mut inner)
}

fn snapshot_from_inner(
    config_path: &PathBuf,
    app: &tauri::AppHandle,
    inner: &mut InnerState,
) -> Result<AppSnapshot, String> {
    let tunnels = inner
        .tunnels
        .iter()
        .map(|definition| {
            let runtime = inner.runtimes.entry(definition.id.clone()).or_default();
            TunnelView {
                definition: definition.clone(),
                status: refresh_runtime(runtime),
                last_error: runtime.last_error.clone(),
                recent_log: runtime.recent_log.clone(),
            }
        })
        .collect();

    Ok(AppSnapshot {
        tunnels,
        ssh_available: ensure_ssh_available().is_ok(),
        autostart_enabled: autostart_enabled(app),
        config_path: config_path.display().to_string(),
    })
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn build_tray_menu<R: tauri::Runtime, M: Manager<R>>(
    manager: &M,
    recent_items: &[TrayTunnelItem],
) -> tauri::Result<Menu<R>> {
    let open = MenuItemBuilder::with_id("open", "Open SSH Tunnel Manager").build(manager)?;
    let disconnect_all =
        MenuItemBuilder::with_id("disconnect_all", "Disconnect All").build(manager)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(manager)?;
    let submenu = if recent_items.is_empty() {
        let placeholder = MenuItemBuilder::with_id("recent:none", "No tunnels configured")
            .enabled(false)
            .build(manager)?;
        SubmenuBuilder::new(manager, "Recent Tunnels")
            .item(&placeholder)
            .build()?
    } else {
        let mut builder = SubmenuBuilder::new(manager, "Recent Tunnels");
        for item in recent_items {
            builder = builder.text(
                tray_action_id(item.action, &item.tunnel_id),
                tray_action_label(item),
            );
        }
        builder.build()?
    };

    MenuBuilder::new(manager)
        .item(&open)
        .item(&submenu)
        .separator()
        .item(&disconnect_all)
        .item(&quit)
        .build()
}

fn refresh_tray_menu(app: &tauri::AppHandle, inner: &InnerState) -> Result<(), String> {
    let tray = app
        .tray_by_id(MAIN_TRAY_ID)
        .ok_or_else(|| "tray icon not found".to_string())?;
    let recent_items = recent_tray_menu_items(inner);
    let menu = build_tray_menu(app, &recent_items).map_err(|error| error.to_string())?;
    tray.set_menu(Some(menu)).map_err(|error| error.to_string())
}

fn build_tray(app: &tauri::App) -> tauri::Result<()> {
    let initial_items = {
        let state = app.state::<AppState>();
        let inner = state.inner.lock().map_err(|_| tauri::Error::InvalidWindowHandle)?;
        recent_tray_menu_items(&inner)
    };
    let menu = build_tray_menu(app, &initial_items)?;

    TrayIconBuilder::with_id(MAIN_TRAY_ID)
        .menu(&menu)
        .tooltip("SSH Tunnel Manager")
        .on_menu_event(|app, event| {
            let event_id = event.id().as_ref().to_string();
            if let Some(id) = event_id.strip_prefix("connect:") {
                let state = app.state::<AppState>();
                let _ = connect_tunnel(app.clone(), id.to_string(), state);
                return;
            }

            if let Some(id) = event_id.strip_prefix("disconnect:") {
                let state = app.state::<AppState>();
                let _ = disconnect_tunnel(app.clone(), id.to_string(), state);
                return;
            }

            match event_id.as_str() {
                "open" => show_main_window(app),
                "disconnect_all" => {
                    let state = app.state::<AppState>();
                    let lock_result = state.inner.lock();
                    if let Ok(mut inner) = lock_result {
                        let ids: Vec<String> =
                            inner.tunnels.iter().map(|item| item.id.clone()).collect();
                        for id in ids {
                            disconnect_runtime(&mut inner, &id);
                            touch_recent_tunnel(&mut inner, &id);
                        }
                        let _ = refresh_tray_menu(app, &inner);
                    }
                }
                "quit" => app.exit(0),
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(AutostartBuilder::new().build())
        .manage(AppState::new())
        .setup(|app| {
            build_tray(app)?;

            if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
                let handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        if let Some(window) = handle.get_webview_window(MAIN_WINDOW_LABEL) {
                            let _ = window.hide();
                        }
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_state,
            save_tunnel,
            delete_tunnel,
            connect_tunnel,
            disconnect_tunnel,
            set_autostart,
            reveal_config_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod runtime_tests {
    use std::{
        thread,
        time::{Duration, Instant},
    };

    use sshtunnel_core::ssh_launch::{CommandSpec, LaunchPlan};

    use super::{
        disconnect_runtime, push_log, refresh_runtime, ManagedProcess, TunnelRuntime, TunnelStatus,
    };

    #[test]
    fn push_log_keeps_only_the_latest_twelve_entries() {
        let mut runtime = TunnelRuntime::default();

        for index in 1..=14 {
            push_log(&mut runtime, &format!("log-{index}"));
        }

        assert_eq!(runtime.recent_log.len(), 12);
        assert_eq!(runtime.recent_log.first().map(String::as_str), Some("log-3"));
        assert_eq!(runtime.recent_log.last().map(String::as_str), Some("log-14"));
    }

    #[test]
    fn refresh_runtime_returns_idle_after_successful_exit_and_clears_old_error() {
        let mut runtime = runtime_with_process(success_exit_command());
        runtime.last_error = Some("stale error".into());

        let status = wait_for_terminal_status(&mut runtime);

        assert!(matches!(status, TunnelStatus::Idle));
        assert!(runtime.process.is_none());
        assert_eq!(runtime.last_error, None);
        assert!(runtime.recent_log.iter().any(|line| line.contains("ssh process exited")));
    }

    #[test]
    fn refresh_runtime_returns_error_after_non_zero_exit() {
        let mut runtime = runtime_with_process(failing_exit_command());

        let status = wait_for_terminal_status(&mut runtime);

        assert!(matches!(status, TunnelStatus::Error));
        assert!(runtime.process.is_none());
        assert!(
            runtime
                .last_error
                .as_deref()
                .is_some_and(|value| value.contains("status"))
        );
        assert!(
            runtime
                .recent_log
                .iter()
                .any(|line| line.contains("ssh exited with status"))
        );
    }

    #[test]
    fn disconnect_runtime_clears_process_adds_stop_log_and_resets_error() {
        let mut runtime = runtime_with_process(long_running_command());
        runtime.last_error = Some("previous failure".into());
        let mut inner = super::InnerState::default();
        inner.runtimes.insert("db".into(), runtime);

        disconnect_runtime(&mut inner, "db");

        let runtime = inner.runtimes.get("db").expect("runtime should remain present");
        assert!(runtime.process.is_none());
        assert_eq!(runtime.last_error, None);
        assert!(
            runtime
                .recent_log
                .iter()
                .any(|line| line.contains("stopped ssh process"))
        );
    }

    fn wait_for_terminal_status(runtime: &mut TunnelRuntime) -> TunnelStatus {
        let deadline = Instant::now() + Duration::from_secs(2);

        loop {
            let status = refresh_runtime(runtime);
            if !matches!(status, TunnelStatus::Connected) {
                return status;
            }

            assert!(
                Instant::now() < deadline,
                "runtime did not reach a terminal state: {:?}",
                runtime.recent_log
            );
            thread::sleep(Duration::from_millis(25));
        }
    }

    fn runtime_with_process(command: CommandSpec) -> TunnelRuntime {
        TunnelRuntime {
            process: Some(
                ManagedProcess::spawn(LaunchPlan::Native(command))
                    .expect("spawn managed process for runtime test"),
            ),
            last_error: None,
            recent_log: Vec::new(),
        }
    }

    #[cfg(target_os = "windows")]
    fn success_exit_command() -> CommandSpec {
        CommandSpec {
            program: "cmd".into(),
            args: vec!["/C".into(), "exit 0".into()],
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn success_exit_command() -> CommandSpec {
        CommandSpec {
            program: "sh".into(),
            args: vec!["-c".into(), "exit 0".into()],
        }
    }

    #[cfg(target_os = "windows")]
    fn failing_exit_command() -> CommandSpec {
        CommandSpec {
            program: "cmd".into(),
            args: vec!["/C".into(), "(echo boom 1>&2) & exit 7".into()],
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn failing_exit_command() -> CommandSpec {
        CommandSpec {
            program: "sh".into(),
            args: vec!["-c".into(), "echo boom >&2; exit 7".into()],
        }
    }

    #[cfg(target_os = "windows")]
    fn long_running_command() -> CommandSpec {
        CommandSpec {
            program: "cmd".into(),
            args: vec!["/C".into(), "ping -n 6 127.0.0.1 > nul".into()],
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn long_running_command() -> CommandSpec {
        CommandSpec {
            program: "sh".into(),
            args: vec!["-c".into(), "sleep 5".into()],
        }
    }
}
