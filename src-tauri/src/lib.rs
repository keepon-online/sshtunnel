mod managed_process;

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
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Manager, State, WindowEvent,
};
use tauri_plugin_autostart::Builder as AutostartBuilder;

const SERVICE_NAME: &str = "sshtunnel-manager";
const MAIN_WINDOW_LABEL: &str = "main";

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
                tunnels,
                runtimes: HashMap::new(),
            }),
        }
    }
}

#[tauri::command]
fn load_state(state: State<'_, AppState>) -> Result<AppSnapshot, String> {
    snapshot(&state)
}

#[tauri::command]
fn save_tunnel(
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

        persist_config(&state.config_path, &inner.tunnels)?;
    }

    snapshot(&state)
}

#[tauri::command]
fn delete_tunnel(id: String, state: State<'_, AppState>) -> Result<AppSnapshot, String> {
    {
        let mut inner = state
            .inner
            .lock()
            .map_err(|_| "state poisoned".to_string())?;
        disconnect_runtime(&mut inner, &id);
        inner.tunnels.retain(|item| item.id != id);
        inner.runtimes.remove(&id);
        let _ = delete_password(&id);
        persist_config(&state.config_path, &inner.tunnels)?;
    }

    snapshot(&state)
}

#[tauri::command]
fn connect_tunnel(id: String, state: State<'_, AppState>) -> Result<AppSnapshot, String> {
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
            return snapshot_from_inner(&state.config_path, &mut inner);
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

    let runtime = inner.runtimes.entry(id).or_default();
    runtime.process = Some(process);
    runtime.last_error = None;
    match pid {
        Some(pid) => push_log(runtime, &format!("spawned ssh process pid={pid}")),
        None => push_log(runtime, "spawned interactive ssh process"),
    }

    snapshot_from_inner(&state.config_path, &mut inner)
}

#[tauri::command]
fn disconnect_tunnel(id: String, state: State<'_, AppState>) -> Result<AppSnapshot, String> {
    let mut inner = state
        .inner
        .lock()
        .map_err(|_| "state poisoned".to_string())?;
    disconnect_runtime(&mut inner, &id);
    snapshot_from_inner(&state.config_path, &mut inner)
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

fn disconnect_runtime(inner: &mut InnerState, id: &str) {
    if let Some(runtime) = inner.runtimes.get_mut(id) {
        let _ = flush_process_logs(runtime);
        if let Some(process) = runtime.process.as_mut() {
            let _ = process.kill();
            push_log(runtime, "stopped ssh process");
        }
        runtime.process = None;
    }
}

fn refresh_runtime(runtime: &mut TunnelRuntime) -> TunnelStatus {
    let _ = flush_process_logs(runtime);

    if let Some(process) = runtime.process.as_mut() {
        match process.try_wait() {
            Ok(Some(status)) => {
                runtime.process = None;
                if status.contains("exit status: 0") || status.contains("code: 0") {
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

fn snapshot(state: &AppState) -> Result<AppSnapshot, String> {
    let mut inner = state
        .inner
        .lock()
        .map_err(|_| "state poisoned".to_string())?;
    snapshot_from_inner(&state.config_path, &mut inner)
}

fn snapshot_from_inner(
    config_path: &PathBuf,
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

fn build_tray(app: &tauri::App) -> tauri::Result<()> {
    let open = MenuItemBuilder::with_id("open", "Open SSH Tunnel Manager").build(app)?;
    let disconnect_all = MenuItemBuilder::with_id("disconnect_all", "Disconnect All").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&open)
        .separator()
        .item(&disconnect_all)
        .item(&quit)
        .build()?;

    TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .tooltip("SSH Tunnel Manager")
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => show_main_window(app),
            "disconnect_all" => {
                let state = app.state::<AppState>();
                let lock_result = state.inner.lock();
                if let Ok(mut inner) = lock_result {
                    let ids: Vec<String> =
                        inner.tunnels.iter().map(|item| item.id.clone()).collect();
                    for id in ids {
                        disconnect_runtime(&mut inner, &id);
                    }
                }
            }
            "quit" => app.exit(0),
            _ => {}
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
            reveal_config_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
