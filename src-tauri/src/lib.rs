mod debug_trace;
mod managed_process;
mod tray_model;
#[cfg(test)]
mod tray_model_tests;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::Mutex,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

use keyring::Entry;
use debug_trace::{init_debug_trace, trace_debug};
use managed_process::ManagedProcess;
use serde::{Deserialize, Serialize};
use sshtunnel_core::{
    models::{AuthKind, TunnelDefinition},
    ssh_args::build_ssh_probe_args,
    ssh_launch::{build_launch_plan, CommandSpec, LaunchPlan},
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
const TEST_LOG_LIMIT: usize = 24;
const CONNECTIVITY_TEST_TIMEOUT: Duration = Duration::from_secs(15);
const SSH_LOGIN_OK_MARKER: &str = "__SSHTUNNEL_LOGIN_OK__";
const TARGET_OK_MARKER: &str = "__SSHTUNNEL_TARGET_OK__";
const TARGET_FAIL_MARKER: &str = "__SSHTUNNEL_TARGET_FAIL__";
const TARGET_TOOL_MISSING_MARKER: &str = "__SSHTUNNEL_TARGET_TOOL_MISSING__";

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
    test_recent_log: Vec<String>,
}

#[derive(Debug)]
struct PreparedConnect {
    tunnel: TunnelDefinition,
    password: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ConnectivityTestResult {
    ssh_ok: bool,
    target_ok: bool,
    summary: String,
    ssh_summary: String,
    target_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ConnectivityTestResponse {
    snapshot: AppSnapshot,
    result: ConnectivityTestResult,
}

#[derive(Debug)]
enum ConnectPreparation {
    Launch(PreparedConnect),
    MissingCredential,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectivityProbeKind {
    SshLogin,
    TargetReachability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConnectivityProbeResult {
    ok: bool,
    summary: String,
    logs: Vec<String>,
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
    disconnect_requested: bool,
    reconnect_pending: bool,
}

impl Default for TunnelRuntime {
    fn default() -> Self {
        Self {
            process: None,
            last_error: None,
            recent_log: Vec::new(),
            disconnect_requested: false,
            reconnect_pending: false,
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
    test_recent_log: Mutex<Vec<String>>,
    tray_menu_signature: Mutex<Option<String>>,
}

impl AppState {
    fn new() -> Self {
        let config_path = resolve_config_path();
        init_debug_trace(&config_path);
        trace_debug("app_state", &format!("config path: {}", config_path.display()));
        let tunnels = load_config(&config_path).unwrap_or_default().tunnels;

        Self {
            config_path,
            inner: Mutex::new(InnerState {
                recent_tunnel_ids: tunnels.iter().map(|item| item.id.clone()).collect(),
                tunnels,
                runtimes: HashMap::new(),
            }),
            test_recent_log: Mutex::new(Vec::new()),
            tray_menu_signature: Mutex::new(None),
        }
    }
}

#[tauri::command]
fn load_state(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<AppSnapshot, String> {
    trace_debug("load_state", "begin");
    snapshot(&state, &app)
}

#[tauri::command]
fn save_tunnel(
    app: tauri::AppHandle,
    payload: SaveTunnelPayload,
    state: State<'_, AppState>,
) -> Result<AppSnapshot, String> {
    let tunnel = prepare_tunnel_for_save(payload.tunnel)?;
    ensure_password_credential_for_save(&tunnel, payload.password.as_deref(), get_password)?;

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

        save_tunnel_to_inner(&mut inner, tunnel);
        persist_config(&state.config_path, &inner.tunnels)?;
        refresh_tray_menu(&app, &inner)?;
    }

    snapshot(&state, &app)
}

#[tauri::command]
fn test_tunnel_connectivity(
    app: tauri::AppHandle,
    payload: SaveTunnelPayload,
    state: State<'_, AppState>,
) -> Result<ConnectivityTestResponse, String> {
    ensure_ssh_available()?;

    let prepared = prepare_tunnel_for_test(payload)?;
    let (result, logs) = run_connectivity_test_with_runner(&prepared, run_connectivity_probe)?;

    {
        let mut test_recent_log = state
            .test_recent_log
            .lock()
            .map_err(|_| "test log state poisoned".to_string())?;
        *test_recent_log = trim_recent_logs(logs, TEST_LOG_LIMIT);
    }

    Ok(ConnectivityTestResponse {
        snapshot: snapshot(&state, &app)?,
        result,
    })
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
        delete_tunnel_from_inner(&mut inner, &id);
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
    trace_debug("connect_tunnel", &format!("begin id={id}"));
    ensure_ssh_available()?;

    let mut inner = state
        .inner
        .lock()
        .map_err(|_| "state poisoned".to_string())?;
    trace_debug("connect_tunnel", &format!("lock acquired id={id}"));
    launch_tunnel(&mut inner, &id, get_password)?;
    trace_debug("connect_tunnel", &format!("launch_tunnel completed id={id}"));

    refresh_tray_menu(&app, &inner)?;
    trace_debug("connect_tunnel", &format!("refresh_tray_menu completed id={id}"));
    snapshot_from_inner(&state, &app, &mut inner)
}

#[tauri::command]
fn disconnect_tunnel(
    app: tauri::AppHandle,
    id: String,
    state: State<'_, AppState>,
) -> Result<AppSnapshot, String> {
    trace_debug("disconnect_tunnel", &format!("begin id={id}"));
    let mut inner = state
        .inner
        .lock()
        .map_err(|_| "state poisoned".to_string())?;
    trace_debug("disconnect_tunnel", &format!("lock acquired id={id}"));
    apply_disconnect(&mut inner, &id);
    trace_debug("disconnect_tunnel", &format!("apply_disconnect completed id={id}"));
    refresh_tray_menu(&app, &inner)?;
    trace_debug("disconnect_tunnel", &format!("refresh_tray_menu completed id={id}"));
    snapshot_from_inner(&state, &app, &mut inner)
}

#[tauri::command]
fn set_autostart(
    app: tauri::AppHandle,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<AppSnapshot, String> {
    let autolaunch = app.autolaunch();
    apply_autostart_choice(
        enabled,
        || autolaunch.enable(),
        || autolaunch.disable(),
    )?;

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
    let mut cmd = std::process::Command::new("ssh");
    cmd.arg("-V")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    cmd.status()
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

fn prepare_tunnel_for_save(mut tunnel: TunnelDefinition) -> Result<TunnelDefinition, String> {
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
    Ok(tunnel)
}

fn prepare_tunnel_for_test(payload: SaveTunnelPayload) -> Result<PreparedConnect, String> {
    let tunnel = prepare_tunnel_for_save(payload.tunnel)?;
    let password = payload.password.and_then(|value| {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    });

    if tunnel.auth_kind == AuthKind::Password && password.is_none() {
        return Err("password auth requires a password value".into());
    }

    Ok(PreparedConnect { tunnel, password })
}

fn ensure_password_credential_for_save<F>(
    tunnel: &TunnelDefinition,
    password: Option<&str>,
    load_password: F,
) -> Result<(), String>
where
    F: FnOnce(&str) -> Result<String, String>,
{
    if tunnel.auth_kind != AuthKind::Password {
        return Ok(());
    }

    if password.is_some_and(|value| !value.trim().is_empty()) {
        return Ok(());
    }

    match load_password(&tunnel.id) {
        Ok(existing) if !existing.trim().is_empty() => Ok(()),
        _ => Err("password auth requires a password or an existing stored credential".into()),
    }
}

fn prepare_connect_request<F>(
    inner: &mut InnerState,
    id: &str,
    load_password: F,
) -> Result<ConnectPreparation, String>
where
    F: FnOnce(&str) -> Result<String, String>,
{
    let tunnel = inner
        .tunnels
        .iter()
        .find(|item| item.id == id)
        .cloned()
        .ok_or_else(|| format!("unknown tunnel id: {id}"))?;

    tunnel.validate()?;
    disconnect_runtime(inner, id);

    if tunnel.auth_kind == AuthKind::Password {
        match load_password(&tunnel.id) {
            Ok(password) => Ok(ConnectPreparation::Launch(PreparedConnect {
                tunnel,
                password: Some(password),
            })),
            Err(_) => {
                let runtime = inner.runtimes.entry(id.to_string()).or_default();
                runtime.last_error = Some("credential is missing in the system keychain".into());
                push_log(runtime, "missing password credential");
                Ok(ConnectPreparation::MissingCredential)
            }
        }
    } else {
        Ok(ConnectPreparation::Launch(PreparedConnect {
            tunnel,
            password: None,
        }))
    }
}

fn launch_tunnel<F>(inner: &mut InnerState, id: &str, load_password: F) -> Result<(), String>
where
    F: FnOnce(&str) -> Result<String, String>,
{
    trace_debug("launch_tunnel", &format!("prepare begin id={id}"));
    let prepared = match prepare_connect_request(inner, id, load_password)? {
        ConnectPreparation::Launch(prepared) => prepared,
        ConnectPreparation::MissingCredential => return Ok(()),
    };
    trace_debug("launch_tunnel", &format!("prepare completed id={id}"));
    let plan = build_launch_plan(&prepared.tunnel, prepared.password.as_deref())?;
    trace_debug("launch_tunnel", &format!("plan built id={id}"));
    let process = ManagedProcess::spawn(plan)?;
    trace_debug("launch_tunnel", &format!("process spawned id={id}"));
    apply_connected_runtime(inner, id, process);
    trace_debug("launch_tunnel", &format!("runtime applied id={id}"));
    Ok(())
}

fn collect_auto_connect_ids(tunnels: &[TunnelDefinition]) -> Vec<String> {
    tunnels
        .iter()
        .filter(|tunnel| tunnel.auto_connect)
        .map(|tunnel| tunnel.id.clone())
        .collect()
}

fn collect_pending_reconnect_ids(inner: &InnerState) -> Vec<String> {
    inner
        .tunnels
        .iter()
        .filter(|tunnel| tunnel.auto_reconnect)
        .filter_map(|tunnel| {
            inner
                .runtimes
                .get(&tunnel.id)
                .filter(|runtime| {
                    runtime.reconnect_pending
                        && !runtime.disconnect_requested
                        && runtime.process.is_none()
                })
                .map(|_| tunnel.id.clone())
        })
        .collect()
}

fn apply_startup_auto_connect(inner: &mut InnerState) {
    for id in collect_auto_connect_ids(&inner.tunnels) {
        let _ = launch_tunnel(inner, &id, get_password);
    }
}

fn maintain_runtime_connections(inner: &mut InnerState) {
    trace_debug("maintain_runtime_connections", "begin");
    let ids: Vec<String> = inner.tunnels.iter().map(|tunnel| tunnel.id.clone()).collect();
    for id in ids {
        if let Some(runtime) = inner.runtimes.get_mut(&id) {
            trace_debug("maintain_runtime_connections", &format!("refresh id={id}"));
            let _ = refresh_runtime(runtime);
        }
    }

    for id in collect_pending_reconnect_ids(inner) {
        if let Some(runtime) = inner.runtimes.get_mut(&id) {
            runtime.reconnect_pending = false;
        }
        trace_debug("maintain_runtime_connections", &format!("relaunch id={id}"));
        let _ = launch_tunnel(inner, &id, get_password);
    }
}

fn spawn_runtime_maintenance_loop(app: tauri::AppHandle) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        let state = app.state::<AppState>();
        if let Ok(mut inner) = state.inner.lock() {
            maintain_runtime_connections(&mut inner);
            let _ = refresh_tray_menu(&app, &inner);
        };
    });
}

fn apply_connected_runtime(inner: &mut InnerState, id: &str, process: ManagedProcess) {
    let pid = process.pid();
    let runtime = inner.runtimes.entry(id.to_string()).or_default();
    runtime.process = Some(process);
    runtime.last_error = None;
    runtime.disconnect_requested = false;
    runtime.reconnect_pending = false;
    match pid {
        Some(pid) => push_log(runtime, &format!("spawned ssh process pid={pid}")),
        None => push_log(runtime, "spawned interactive ssh process"),
    }
    touch_recent_tunnel(inner, id);
}

fn apply_disconnect(inner: &mut InnerState, id: &str) {
    disconnect_runtime(inner, id);
    let runtime = inner.runtimes.entry(id.to_string()).or_default();
    runtime.disconnect_requested = true;
    runtime.reconnect_pending = false;
    touch_recent_tunnel(inner, id);
}

fn apply_disconnect_all(inner: &mut InnerState) {
    let ids: Vec<String> = inner.tunnels.iter().map(|item| item.id.clone()).collect();
    for id in ids {
        disconnect_runtime(inner, &id);
        let runtime = inner.runtimes.entry(id.to_string()).or_default();
        runtime.disconnect_requested = true;
        runtime.reconnect_pending = false;
        touch_recent_tunnel(inner, &id);
    }
}

fn apply_autostart_choice<E, Enable, Disable>(
    enabled: bool,
    enable: Enable,
    disable: Disable,
) -> Result<(), String>
where
    E: ToString,
    Enable: FnOnce() -> Result<(), E>,
    Disable: FnOnce() -> Result<(), E>,
{
    if enabled {
        enable().map_err(|error| error.to_string())
    } else {
        disable().map_err(|error| error.to_string())
    }
}

fn save_tunnel_to_inner(inner: &mut InnerState, tunnel: TunnelDefinition) {
    let recent_id = tunnel.id.clone();

    match inner
        .tunnels
        .iter_mut()
        .find(|existing| existing.id == tunnel.id)
    {
        Some(existing) => *existing = tunnel,
        None => inner.tunnels.push(tunnel),
    }

    touch_recent_tunnel(inner, &recent_id);
}

fn delete_tunnel_from_inner(inner: &mut InnerState, id: &str) {
    disconnect_runtime(inner, id);
    inner.tunnels.retain(|item| item.id != id);
    inner.runtimes.remove(id);
    remove_recent_tunnel(inner, id);
}

fn push_log(runtime: &mut TunnelRuntime, entry: &str) {
    push_recent_entry(&mut runtime.recent_log, entry.to_string(), 12);
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
        .is_some_and(runtime_is_connected_for_tray)
}

fn runtime_is_connected_for_tray(runtime: &TunnelRuntime) -> bool {
    let Some(process) = runtime.process.as_ref() else {
        return false;
    };

    if runtime.last_error.is_some() {
        return false;
    }

    if process.needs_connection_signal() && !has_connection_signal(&runtime.recent_log) {
        return false;
    }

    true
}

fn recent_tray_menu_items(inner: &InnerState) -> Vec<TrayTunnelItem> {
    let ordered = order_recent_tunnels(&inner.tunnels, &inner.recent_tunnel_ids);
    recent_tray_items(ordered, |id| is_connected(inner, id))
}

fn tray_menu_signature(recent_items: &[TrayTunnelItem]) -> String {
    recent_items
        .iter()
        .map(|item| {
            format!(
                "{}|{}|{}|{}",
                item.tunnel_id,
                item.title,
                item.detail,
                tray_action_id(item.action, &item.tunnel_id)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn should_refresh_tray_menu(
    last_signature: &mut Option<String>,
    recent_items: &[TrayTunnelItem],
) -> bool {
    let next_signature = tray_menu_signature(recent_items);
    if last_signature.as_deref() == Some(next_signature.as_str()) {
        return false;
    }

    *last_signature = Some(next_signature);
    true
}

fn disconnect_runtime(inner: &mut InnerState, id: &str) {
    if let Some(runtime) = inner.runtimes.get_mut(id) {
        trace_debug("disconnect_runtime", &format!("begin id={id}"));
        let _ = flush_process_logs(runtime);
        if let Some(process) = runtime.process.as_mut() {
            trace_debug("disconnect_runtime", &format!("kill begin id={id}"));
            let _ = process.kill();
            trace_debug("disconnect_runtime", &format!("kill end id={id}"));
            push_log(runtime, "stopped ssh process");
        }
        runtime.process = None;
        runtime.last_error = None;
        trace_debug("disconnect_runtime", &format!("end id={id}"));
    }
}

fn refresh_runtime(runtime: &mut TunnelRuntime) -> TunnelStatus {
    let _ = flush_process_logs(runtime);

    if let Some(process) = runtime.process.as_mut() {
        trace_debug("refresh_runtime", "try_wait begin");
        match process.try_wait() {
            Ok(Some(status)) => {
                trace_debug("refresh_runtime", &format!("try_wait exited status={status}"));
                let prompted_exited_before_connecting =
                    process.needs_connection_signal() && !has_connection_signal(&runtime.recent_log);
                runtime.process = None;
                if status.contains("exit status: 0") || status.contains("code: 0") {
                    runtime.last_error = None;
                    runtime.reconnect_pending = false;
                    push_log(runtime, "ssh process exited");
                    TunnelStatus::Idle
                } else {
                    runtime.last_error = Some(format!("ssh exited with status {status}"));
                    // 认证错误（如密码错误）不应自动重连，否则会不断产生新进程
                    let has_auth_error = is_auth_error(&runtime.recent_log);
                    runtime.reconnect_pending =
                        !runtime.disconnect_requested
                            && !has_auth_error
                            && !prompted_exited_before_connecting;
                    push_log(runtime, &format!("ssh exited with status {status}"));
                    if has_auth_error {
                        push_log(runtime, "auto-reconnect skipped: authentication error detected");
                    } else if prompted_exited_before_connecting {
                        push_log(
                            runtime,
                            "auto-reconnect skipped: prompted session exited before connecting",
                        );
                    }
                    TunnelStatus::Error
                }
            }
            Ok(None) => {
                trace_debug("refresh_runtime", "try_wait pending");
                if let Some(error) = detect_runtime_error(&runtime.recent_log) {
                    let prompted_exited_before_connecting =
                        process.needs_connection_signal() && !has_connection_signal(&runtime.recent_log);
                    runtime.last_error = Some(error.clone());
                    // 进程仍在运行但检测到致命错误，主动 kill 并阻止重连
                    trace_debug("refresh_runtime", &format!("detected error: {error}"));
                    trace_debug("refresh_runtime", "kill begin");
                    let _ = process.kill();
                    trace_debug("refresh_runtime", "kill end");
                    runtime.process = None;
                    runtime.reconnect_pending =
                        !runtime.disconnect_requested
                            && !is_auth_error(&runtime.recent_log)
                            && !prompted_exited_before_connecting;
                    push_log(runtime, "killed ssh process due to detected error");
                    if prompted_exited_before_connecting {
                        push_log(
                            runtime,
                            "auto-reconnect skipped: prompted session exited before connecting",
                        );
                    }
                    return TunnelStatus::Error;
                }

                if process.needs_connection_signal() && !has_connection_signal(&runtime.recent_log) {
                    runtime.last_error = None;
                    return TunnelStatus::Idle;
                }

                runtime.last_error = None;
                TunnelStatus::Connected
            }
            Err(error) => {
                trace_debug("refresh_runtime", &format!("try_wait error: {error}"));
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

/// 认证类错误模式，这些错误不应触发自动重连（重连也会以同样方式失败）
const AUTH_ERROR_PATTERNS: &[&str] = &[
    "permission denied",
    "host key verification failed",
    "too many authentication failures",
    "no supported authentication methods",
    "authentication failed",
];

/// 网络类错误模式，这些错误可以尝试重连
const NETWORK_ERROR_PATTERNS: &[&str] = &[
    "connection refused",
    "connection timed out",
    "could not resolve hostname",
    "name or service not known",
    "no route to host",
    "network is unreachable",
    "operation timed out",
    "connection reset by peer",
    "broken pipe",
];

fn detect_runtime_error(logs: &[String]) -> Option<String> {
    let all_patterns: Vec<&str> = AUTH_ERROR_PATTERNS
        .iter()
        .chain(NETWORK_ERROR_PATTERNS.iter())
        .copied()
        .collect();

    logs.iter().rev().find_map(|line| {
        let lower = line.to_ascii_lowercase();
        all_patterns
            .iter()
            .any(|pattern| lower.contains(pattern))
            .then(|| line.clone())
    })
}

/// 判断日志中是否包含认证类错误
fn is_auth_error(logs: &[String]) -> bool {
    logs.iter().rev().any(|line| {
        let lower = line.to_ascii_lowercase();
        AUTH_ERROR_PATTERNS
            .iter()
            .any(|pattern| lower.contains(pattern))
    })
}

fn has_connection_signal(logs: &[String]) -> bool {
    const SUCCESS_PATTERNS: &[&str] = &[
        "authenticated to ",
        "entering interactive session",
        "pledge: network",
    ];

    logs.iter().any(|line| {
        let lower = line.to_ascii_lowercase();
        SUCCESS_PATTERNS
            .iter()
            .any(|pattern| lower.contains(pattern))
    })
}

fn flush_process_logs(runtime: &mut TunnelRuntime) -> Result<(), String> {
    if let Some(process) = runtime.process.as_ref() {
        for line in process.take_logs() {
            push_log(runtime, &line);
        }
    }

    Ok(())
}

fn push_recent_entry(entries: &mut Vec<String>, entry: String, limit: usize) {
    entries.push(entry);
    if entries.len() > limit {
        let trim = entries.len() - limit;
        entries.drain(0..trim);
    }
}

fn trim_recent_logs(entries: Vec<String>, limit: usize) -> Vec<String> {
    let mut trimmed = Vec::new();
    for entry in entries {
        push_recent_entry(&mut trimmed, entry, limit);
    }
    trimmed
}

fn run_connectivity_test_with_runner<F>(
    prepared: &PreparedConnect,
    mut runner: F,
) -> Result<(ConnectivityTestResult, Vec<String>), String>
where
    F: FnMut(
        &TunnelDefinition,
        Option<&str>,
        ConnectivityProbeKind,
    ) -> Result<ConnectivityProbeResult, String>,
{
    let mut logs = vec![format!(
        "[测试状态] 开始测试 {}: {}@{} -> {}:{}",
        prepared.tunnel.name,
        prepared.tunnel.username,
        prepared.tunnel.ssh_host,
        prepared.tunnel.remote_host,
        prepared.tunnel.remote_port
    )];

    let ssh_result = runner(
        &prepared.tunnel,
        prepared.password.as_deref(),
        ConnectivityProbeKind::SshLogin,
    )?;
    append_connectivity_probe_logs(&mut logs, &ssh_result.logs);

    if !ssh_result.ok {
        let summary = format!("SSH 登录失败：{}", ssh_result.summary);
        logs.push(format!("[测试状态] {summary}"));
        return Ok((
            ConnectivityTestResult {
                ssh_ok: false,
                target_ok: false,
                summary,
                ssh_summary: ssh_result.summary,
                target_summary: None,
            },
            logs,
        ));
    }

    logs.push(format!("[测试状态] SSH 登录成功：{}", ssh_result.summary));

    let target_result = runner(
        &prepared.tunnel,
        prepared.password.as_deref(),
        ConnectivityProbeKind::TargetReachability,
    )?;
    append_connectivity_probe_logs(&mut logs, &target_result.logs);

    let (summary, target_status_line) = if target_result.ok {
        (
            "SSH 登录成功，远端目标可达。".to_string(),
            format!("[测试状态] 远端目标可达：{}", target_result.summary),
        )
    } else {
        (
            format!("SSH 登录成功，但远端目标不可达：{}", target_result.summary),
            format!("[测试状态] 远端目标不可达：{}", target_result.summary),
        )
    };

    logs.push(target_status_line);

    Ok((
        ConnectivityTestResult {
            ssh_ok: true,
            target_ok: target_result.ok,
            summary,
            ssh_summary: ssh_result.summary,
            target_summary: Some(target_result.summary),
        },
        logs,
    ))
}

fn append_connectivity_probe_logs(entries: &mut Vec<String>, logs: &[String]) {
    for line in logs {
        let trimmed = line.trim();
        if trimmed.is_empty() || is_probe_marker_line(trimmed) {
            continue;
        }
        entries.push(format!("[测试输出] {trimmed}"));
    }
}

fn is_probe_marker_line(line: &str) -> bool {
    [
        SSH_LOGIN_OK_MARKER,
        TARGET_OK_MARKER,
        TARGET_FAIL_MARKER,
        TARGET_TOOL_MISSING_MARKER,
    ]
    .iter()
    .any(|marker| line.contains(marker))
}

fn run_connectivity_probe(
    tunnel: &TunnelDefinition,
    password: Option<&str>,
    kind: ConnectivityProbeKind,
) -> Result<ConnectivityProbeResult, String> {
    let remote_command = build_connectivity_probe_remote_command(tunnel, kind);
    let plan = build_connectivity_probe_launch_plan(tunnel, password, &remote_command)?;
    let mut process = ManagedProcess::spawn(plan)?;
    let deadline = SystemTime::now() + CONNECTIVITY_TEST_TIMEOUT;
    let mut logs = Vec::new();

    loop {
        logs.extend(process.take_logs());

        if let Some(status) = process.try_wait()? {
            logs.extend(process.take_logs());
            return Ok(parse_connectivity_probe_result(kind, &status, logs));
        }

        if SystemTime::now() >= deadline {
            let _ = process.kill();
            logs.extend(process.take_logs());
            logs.push("connectivity probe timed out".into());
            return Ok(ConnectivityProbeResult {
                ok: false,
                summary: "测试超时".into(),
                logs,
            });
        }

        thread::sleep(Duration::from_millis(50));
    }
}

fn build_connectivity_probe_launch_plan(
    tunnel: &TunnelDefinition,
    password: Option<&str>,
    remote_command: &str,
) -> Result<LaunchPlan, String> {
    tunnel.validate()?;

    let command = CommandSpec {
        program: "ssh".to_string(),
        args: build_ssh_probe_args(tunnel, remote_command),
    };

    match tunnel.auth_kind {
        AuthKind::PrivateKey => Ok(LaunchPlan::Native(command)),
        AuthKind::Password => {
            let password = password
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| "password auth requires a password value".to_string())?;

            Ok(LaunchPlan::PromptedPassword {
                command,
                password: password.to_string(),
                prompt: "assword:".to_string(),
            })
        }
    }
}

fn build_connectivity_probe_remote_command(
    tunnel: &TunnelDefinition,
    kind: ConnectivityProbeKind,
) -> String {
    match kind {
        ConnectivityProbeKind::SshLogin => {
            let script = format!("printf '%s\\n' \"{SSH_LOGIN_OK_MARKER}\" >&2");
            format!("sh -lc {}", shell_single_quote(&script))
        }
        ConnectivityProbeKind::TargetReachability => {
            let host_quoted = shell_single_quote(&tunnel.remote_host);
            let host_for_dev_tcp = tunnel
                .remote_host
                .replace('\\', "\\\\")
                .replace('"', "\\\"");
            let script = format!(
                "if command -v nc >/dev/null 2>&1; then \
                    nc -z -w 5 {host_quoted} {port} >/dev/null 2>&1 && printf '%s\\n' \"{ok}\" >&2 || {{ code=$?; printf '%s\\n' \"{fail}\" >&2; exit \"$code\"; }}; \
                elif command -v bash >/dev/null 2>&1; then \
                    bash -lc \"exec 3<>/dev/tcp/{host_for_dev_tcp}/{port}\" >/dev/null 2>&1 && printf '%s\\n' \"{ok}\" >&2 || {{ code=$?; printf '%s\\n' \"{fail}\" >&2; exit \"$code\"; }}; \
                else \
                    printf '%s\\n' \"{missing}\" >&2; exit 9; \
                fi",
                port = tunnel.remote_port,
                ok = TARGET_OK_MARKER,
                fail = TARGET_FAIL_MARKER,
                missing = TARGET_TOOL_MISSING_MARKER,
            );
            format!("sh -lc {}", shell_single_quote(&script))
        }
    }
}

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn parse_connectivity_probe_result(
    kind: ConnectivityProbeKind,
    status: &str,
    logs: Vec<String>,
) -> ConnectivityProbeResult {
    let success = command_status_is_success(status);

    match kind {
        ConnectivityProbeKind::SshLogin => {
            if logs.iter().any(|line| line.contains(SSH_LOGIN_OK_MARKER)) {
                ConnectivityProbeResult {
                    ok: true,
                    summary: "已完成 SSH 握手".into(),
                    logs,
                }
            } else {
                ConnectivityProbeResult {
                    ok: false,
                    summary: summarize_probe_failure(&logs, status, "SSH 登录未通过"),
                    logs,
                }
            }
        }
        ConnectivityProbeKind::TargetReachability => {
            if logs.iter().any(|line| line.contains(TARGET_OK_MARKER)) {
                ConnectivityProbeResult {
                    ok: true,
                    summary: "远端目标可达".into(),
                    logs,
                }
            } else if logs
                .iter()
                .any(|line| line.contains(TARGET_TOOL_MISSING_MARKER))
            {
                ConnectivityProbeResult {
                    ok: false,
                    summary: "远端主机缺少 nc 或 bash，无法检查目标端口".into(),
                    logs,
                }
            } else if logs.iter().any(|line| line.contains(TARGET_FAIL_MARKER)) || !success {
                ConnectivityProbeResult {
                    ok: false,
                    summary: summarize_probe_failure(&logs, status, "远端目标不可达"),
                    logs,
                }
            } else {
                ConnectivityProbeResult {
                    ok: false,
                    summary: "远端目标检查未返回成功标记".into(),
                    logs,
                }
            }
        }
    }
}

fn summarize_probe_failure(logs: &[String], status: &str, fallback: &str) -> String {
    logs.iter()
        .rev()
        .find_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || is_probe_marker_line(trimmed) {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .unwrap_or_else(|| {
            if status.trim().is_empty() {
                fallback.to_string()
            } else {
                format!("{fallback} ({status})")
            }
        })
}

fn command_status_is_success(status: &str) -> bool {
    let lower = status.to_ascii_lowercase();
    lower.contains("exit status: 0")
        || lower.contains("code: 0")
        || lower.contains("unix_wait_status(0)")
}

fn autostart_enabled(app: &tauri::AppHandle) -> bool {
    app.autolaunch().is_enabled().unwrap_or(false)
}

fn snapshot(state: &AppState, app: &tauri::AppHandle) -> Result<AppSnapshot, String> {
    trace_debug("snapshot", "begin");
    let mut inner = state
        .inner
        .lock()
        .map_err(|_| "state poisoned".to_string())?;
    trace_debug("snapshot", "lock acquired");
    snapshot_from_inner(state, app, &mut inner)
}

fn snapshot_from_inner(
    state: &AppState,
    app: &tauri::AppHandle,
    inner: &mut InnerState,
) -> Result<AppSnapshot, String> {
    trace_debug(
        "snapshot_from_inner",
        &format!("begin tunnels={}", inner.tunnels.len()),
    );
    let tunnels = inner
        .tunnels
        .iter()
        .map(|definition| {
            let runtime = inner.runtimes.entry(definition.id.clone()).or_default();
            trace_debug(
                "snapshot_from_inner",
                &format!("refreshing definition_id={}", definition.id),
            );
            TunnelView {
                definition: definition.clone(),
                status: refresh_runtime(runtime),
                last_error: runtime.last_error.clone(),
                recent_log: runtime.recent_log.clone(),
            }
        })
        .collect();
    trace_debug("snapshot_from_inner", "runtime refresh completed");

    Ok(AppSnapshot {
        tunnels,
        ssh_available: ensure_ssh_available().is_ok(),
        autostart_enabled: autostart_enabled(app),
        config_path: state.config_path.display().to_string(),
        test_recent_log: state
            .test_recent_log
            .lock()
            .map_err(|_| "test log state poisoned".to_string())?
            .clone(),
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
    let open = MenuItemBuilder::with_id("open", "打开主界面").build(manager)?;
    let disconnect_all = MenuItemBuilder::with_id("disconnect_all", "全部断开").build(manager)?;
    let quit = MenuItemBuilder::with_id("quit", "退出").build(manager)?;
    let submenu = if recent_items.is_empty() {
        let placeholder = MenuItemBuilder::with_id("recent:none", "暂无隧道配置")
            .enabled(false)
            .build(manager)?;
        SubmenuBuilder::new(manager, "最近隧道")
            .item(&placeholder)
            .build()?
    } else {
        let mut builder = SubmenuBuilder::new(manager, "最近隧道");
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
    trace_debug("refresh_tray_menu", "begin");
    let tray = app
        .tray_by_id(MAIN_TRAY_ID)
        .ok_or_else(|| "tray icon not found".to_string())?;
    let recent_items = recent_tray_menu_items(inner);
    let state = app.state::<AppState>();
    let mut signature = state
        .tray_menu_signature
        .lock()
        .map_err(|_| "tray signature poisoned".to_string())?;
    if !should_refresh_tray_menu(&mut signature, &recent_items) {
        trace_debug("refresh_tray_menu", "skipped");
        return Ok(());
    }
    let menu = build_tray_menu(app, &recent_items).map_err(|error| error.to_string())?;
    let result = tray.set_menu(Some(menu)).map_err(|error| error.to_string());
    trace_debug("refresh_tray_menu", "end");
    result
}

fn build_tray(app: &tauri::App) -> tauri::Result<()> {
    let initial_items = {
        let state = app.state::<AppState>();
        let inner = state.inner.lock().map_err(|_| tauri::Error::InvalidWindowHandle)?;
        recent_tray_menu_items(&inner)
    };
    {
        let state = app.state::<AppState>();
        let mut signature = state
            .tray_menu_signature
            .lock()
            .map_err(|_| tauri::Error::InvalidWindowHandle)?;
        *signature = Some(tray_menu_signature(&initial_items));
    }
    let menu = build_tray_menu(app, &initial_items)?;
    let mut tray = TrayIconBuilder::with_id(MAIN_TRAY_ID).menu(&menu);
    if let Some(icon) = app.default_window_icon().cloned() {
        tray = tray.icon(icon);
    }

    tray.tooltip("SSH Tunnel Manager").on_menu_event(|app, event| {
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
                        apply_disconnect_all(&mut inner);
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
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .setup(|app| {
            build_tray(app)?;
            {
                let state = app.state::<AppState>();
                if let Ok(mut inner) = state.inner.lock() {
                    apply_startup_auto_connect(&mut inner);
                    let _ = refresh_tray_menu(app.handle(), &inner);
                };
            }
            spawn_runtime_maintenance_loop(app.handle().clone());

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
            test_tunnel_connectivity,
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

    use sshtunnel_core::models::{AuthKind, TunnelDefinition};
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
    fn refresh_runtime_does_not_report_connected_while_prompted_session_is_waiting() {
        let mut runtime = runtime_with_launch(LaunchPlan::PromptedPassword {
            command: prompted_waiting_command(),
            password: "s3cr3t".into(),
            prompt: "assword:".into(),
        });

        let status = wait_for_status_with_log(&mut runtime, |runtime| {
            runtime
                .recent_log
                .iter()
                .any(|line| line.contains("password sent to interactive ssh session"))
        });

        disconnect_runtime_from_runtime(&mut runtime);

        assert!(matches!(status, TunnelStatus::Idle));
        assert!(
            runtime
                .recent_log
                .iter()
                .any(|line| line.contains("password sent to interactive ssh session"))
        );
    }

    #[test]
    fn refresh_runtime_returns_error_when_live_session_emits_auth_failure() {
        let mut runtime = runtime_with_launch(LaunchPlan::PromptedPassword {
            command: prompted_auth_failure_command(),
            password: "s3cr3t".into(),
            prompt: "assword:".into(),
        });

        let status = wait_for_status_with_log(&mut runtime, |runtime| {
            runtime
                .recent_log
                .iter()
                .any(|line| line.to_ascii_lowercase().contains("permission denied"))
        });

        disconnect_runtime_from_runtime(&mut runtime);

        assert!(matches!(status, TunnelStatus::Error));
        assert!(
            runtime
                .last_error
                .as_deref()
                .is_some_and(|value| value.to_ascii_lowercase().contains("permission denied"))
        );
    }

    #[test]
    fn refresh_runtime_skips_reconnect_when_prompted_session_exits_before_connecting() {
        let mut runtime = runtime_with_launch(LaunchPlan::PromptedPassword {
            command: prompted_early_failure_command(),
            password: "s3cr3t".into(),
            prompt: "assword:".into(),
        });

        let status = wait_for_status_with_log(&mut runtime, |runtime| runtime.process.is_none());

        assert!(matches!(status, TunnelStatus::Error));
        assert!(!runtime.reconnect_pending);
        assert!(
            runtime.recent_log.iter().any(|line| {
                line.contains("auto-reconnect skipped: prompted session exited before connecting")
            }),
            "expected prompted reconnect skip log, got {:?}",
            runtime.recent_log
        );
    }

    #[test]
    fn recent_tray_items_do_not_mark_prompted_waiting_session_connected() {
        let mut runtime = runtime_with_launch(LaunchPlan::PromptedPassword {
            command: prompted_waiting_command(),
            password: "s3cr3t".into(),
            prompt: "assword:".into(),
        });

        let _ = wait_for_status_with_log(&mut runtime, |runtime| {
            runtime
                .recent_log
                .iter()
                .any(|line| line.contains("password sent to interactive ssh session"))
        });

        let mut inner = super::InnerState {
            tunnels: vec![tunnel("db")],
            runtimes: [("db".to_string(), runtime)].into_iter().collect(),
            recent_tunnel_ids: vec!["db".into()],
        };

        let items = super::recent_tray_menu_items(&inner);
        disconnect_runtime(&mut inner, "db");

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].action, super::tray_model::TrayTunnelAction::Connect);
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

    fn wait_for_status_with_log(
        runtime: &mut TunnelRuntime,
        ready: impl Fn(&TunnelRuntime) -> bool,
    ) -> TunnelStatus {
        let deadline = Instant::now() + Duration::from_secs(2);

        loop {
            let status = refresh_runtime(runtime);
            if ready(runtime) {
                return status;
            }

            assert!(
                Instant::now() < deadline,
                "runtime did not emit expected logs: {:?}",
                runtime.recent_log
            );
            thread::sleep(Duration::from_millis(25));
        }
    }

    fn runtime_with_process(command: CommandSpec) -> TunnelRuntime {
        runtime_with_launch(LaunchPlan::Native(command))
    }

    fn runtime_with_launch(plan: LaunchPlan) -> TunnelRuntime {
        TunnelRuntime {
            process: Some(ManagedProcess::spawn(plan).expect("spawn managed process for runtime test")),
            last_error: None,
            recent_log: Vec::new(),
            disconnect_requested: false,
            reconnect_pending: false,
        }
    }

    fn disconnect_runtime_from_runtime(runtime: &mut TunnelRuntime) {
        if let Some(process) = runtime.process.as_mut() {
            let _ = process.kill();
        }
        runtime.process = None;
    }

    fn tunnel(id: &str) -> TunnelDefinition {
        TunnelDefinition {
            id: id.into(),
            name: format!("{id}-name"),
            ssh_host: "bastion.example.com".into(),
            ssh_port: 22,
            username: "deploy".into(),
            local_bind_address: "127.0.0.1".into(),
            local_bind_port: 15432,
            remote_host: "10.0.0.12".into(),
            remote_port: 5432,
            auth_kind: AuthKind::PrivateKey,
            private_key_path: Some("~/.ssh/id_ed25519".into()),
            auto_connect: false,
            auto_reconnect: true,
            password_entry: None,
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

    #[cfg(target_os = "windows")]
    fn prompted_waiting_command() -> CommandSpec {
        CommandSpec {
            program: "cmd".into(),
            args: vec![
                "/C".into(),
                "set /p =Password: < nul & ping -n 6 127.0.0.1 > nul".into(),
            ],
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn prompted_waiting_command() -> CommandSpec {
        CommandSpec {
            program: "sh".into(),
            args: vec!["-c".into(), "printf 'Password:'; sleep 5".into()],
        }
    }

    #[cfg(target_os = "windows")]
    fn prompted_auth_failure_command() -> CommandSpec {
        CommandSpec {
            program: "cmd".into(),
            args: vec![
                "/C".into(),
                "(echo Permission denied 1>&2) & ping -n 6 127.0.0.1 > nul".into(),
            ],
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn prompted_auth_failure_command() -> CommandSpec {
        CommandSpec {
            program: "sh".into(),
            args: vec!["-c".into(), "echo 'Permission denied' >&2; sleep 5".into()],
        }
    }

    #[cfg(target_os = "windows")]
    fn prompted_early_failure_command() -> CommandSpec {
        CommandSpec {
            program: "cmd".into(),
            args: vec!["/C".into(), "(echo fatal startup failure 1>&2) & exit /b 42".into()],
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn prompted_early_failure_command() -> CommandSpec {
        CommandSpec {
            program: "sh".into(),
            args: vec!["-c".into(), "echo 'fatal startup failure' >&2; exit 42".into()],
        }
    }
}

#[cfg(test)]
mod save_delete_tests {
    use std::{
        env,
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use sshtunnel_core::models::{AuthKind, TunnelDefinition};

    use super::{
        load_config, persist_config, InnerState, TunnelRuntime,
    };

    #[test]
    fn prepare_tunnel_for_save_normalizes_password_auth_fields() {
        let tunnel = TunnelDefinition {
            auth_kind: AuthKind::Password,
            private_key_path: Some("~/.ssh/id_ed25519".into()),
            password_entry: None,
            ..sample_tunnel("db")
        };

        let prepared = super::prepare_tunnel_for_save(tunnel).expect("normalize tunnel");

        assert_eq!(prepared.private_key_path, None);
        assert_eq!(prepared.password_entry.as_deref(), Some("profile:db"));
    }

    #[test]
    fn save_tunnel_to_inner_inserts_new_tunnel_and_updates_recent_order() {
        let mut inner = InnerState {
            tunnels: vec![sample_tunnel("cache")],
            runtimes: Default::default(),
            recent_tunnel_ids: vec!["cache".into()],
        };

        let tunnel = super::prepare_tunnel_for_save(sample_tunnel("db")).expect("normalize tunnel");

        super::save_tunnel_to_inner(&mut inner, tunnel);

        assert_eq!(inner.tunnels.len(), 2);
        let stored = inner
            .tunnels
            .iter()
            .find(|item| item.id == "db")
            .expect("saved tunnel present");
        assert_eq!(stored.password_entry, None);
        assert_eq!(inner.recent_tunnel_ids, vec!["db".to_string(), "cache".to_string()]);
    }

    #[test]
    fn save_tunnel_to_inner_replaces_existing_tunnel_without_duplicates() {
        let mut inner = InnerState {
            tunnels: vec![sample_tunnel("db")],
            runtimes: Default::default(),
            recent_tunnel_ids: vec!["cache".into(), "db".into()],
        };
        let mut updated = sample_tunnel("db");
        updated.name = "Database Prod".into();
        updated.local_bind_port = 25432;
        let tunnel = super::prepare_tunnel_for_save(updated).expect("normalize tunnel");

        super::save_tunnel_to_inner(&mut inner, tunnel);

        assert_eq!(inner.tunnels.len(), 1);
        assert_eq!(inner.tunnels[0].name, "Database Prod");
        assert_eq!(inner.tunnels[0].local_bind_port, 25432);
        assert_eq!(inner.recent_tunnel_ids, vec!["db".to_string(), "cache".to_string()]);
    }

    #[test]
    fn delete_tunnel_from_inner_removes_runtime_recent_entry_and_persisted_config() {
        let config_path = unique_temp_path("delete-tunnel");
        let survivor = sample_tunnel("cache");
        let deleted = sample_tunnel("db");
        let mut inner = InnerState {
            tunnels: vec![deleted.clone(), survivor.clone()],
            runtimes: [
                ("db".to_string(), TunnelRuntime::default()),
                ("cache".to_string(), TunnelRuntime::default()),
            ]
            .into_iter()
            .collect(),
            recent_tunnel_ids: vec!["db".into(), "cache".into()],
        };
        persist_config(&config_path, &inner.tunnels).expect("persist initial config");

        super::delete_tunnel_from_inner(&mut inner, "db");
        persist_config(&config_path, &inner.tunnels).expect("persist updated config");
        let stored = load_config(&config_path).expect("load updated config");

        assert_eq!(inner.tunnels, vec![survivor.clone()]);
        assert!(!inner.runtimes.contains_key("db"));
        assert_eq!(inner.recent_tunnel_ids, vec!["cache".to_string()]);
        assert_eq!(stored.tunnels, vec![survivor]);

        let _ = fs::remove_file(config_path);
    }

    #[test]
    fn password_auth_save_requires_new_or_existing_credential() {
        let tunnel = super::prepare_tunnel_for_save(password_tunnel("db")).expect("normalize tunnel");

        let result = super::ensure_password_credential_for_save(&tunnel, None, |_| {
            Err("missing secret".into())
        });

        assert_eq!(
            result,
            Err("password auth requires a password or an existing stored credential".into())
        );
    }

    #[test]
    fn password_auth_save_allows_existing_stored_credential() {
        let tunnel = super::prepare_tunnel_for_save(password_tunnel("db")).expect("normalize tunnel");

        let result = super::ensure_password_credential_for_save(&tunnel, None, |_| {
            Ok("stored-secret".into())
        });

        assert_eq!(result, Ok(()));
    }

    fn sample_tunnel(id: &str) -> TunnelDefinition {
        TunnelDefinition {
            id: id.into(),
            name: format!("{id}-name"),
            ssh_host: "bastion.example.com".into(),
            ssh_port: 22,
            username: "deploy".into(),
            local_bind_address: "127.0.0.1".into(),
            local_bind_port: 15432,
            remote_host: "10.0.0.12".into(),
            remote_port: 5432,
            auth_kind: AuthKind::PrivateKey,
            private_key_path: Some("~/.ssh/id_ed25519".into()),
            auto_connect: false,
            auto_reconnect: true,
            password_entry: Some("profile:stale".into()),
        }
    }

    fn password_tunnel(id: &str) -> TunnelDefinition {
        TunnelDefinition {
            auth_kind: AuthKind::Password,
            private_key_path: None,
            password_entry: Some(format!("profile:{id}")),
            ..sample_tunnel(id)
        }
    }

    fn unique_temp_path(prefix: &str) -> PathBuf {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before epoch")
            .as_millis();
        env::temp_dir().join(format!("sshtunnel-{prefix}-{millis}.json"))
    }
}

#[cfg(test)]
mod command_flow_tests {
    use std::cell::Cell;

    use sshtunnel_core::{
        models::{AuthKind, TunnelDefinition},
        ssh_launch::{CommandSpec, LaunchPlan},
    };

    use super::{apply_autostart_choice, InnerState, ManagedProcess, TunnelRuntime};

    #[test]
    fn prepare_connect_request_returns_error_for_unknown_tunnel() {
        let mut inner = InnerState::default();

        let result = super::prepare_connect_request(&mut inner, "missing", |_| Ok(String::new()));

        assert_eq!(result.expect_err("unknown id should fail"), "unknown tunnel id: missing");
    }

    #[test]
    fn prepare_connect_request_marks_runtime_when_password_credential_is_missing() {
        let mut inner = InnerState {
            tunnels: vec![password_tunnel("db")],
            runtimes: Default::default(),
            recent_tunnel_ids: vec![],
        };

        let result = super::prepare_connect_request(&mut inner, "db", |_| Err("no secret".into()))
            .expect("helper should not fail for missing credential");

        assert!(matches!(result, super::ConnectPreparation::MissingCredential));
        let runtime = inner.runtimes.get("db").expect("runtime should be created");
        assert_eq!(
            runtime.last_error.as_deref(),
            Some("credential is missing in the system keychain")
        );
        assert!(
            runtime
                .recent_log
                .iter()
                .any(|line| line.contains("missing password credential"))
        );
    }

    #[test]
    fn apply_connected_runtime_replaces_old_runtime_clears_error_and_updates_recent_order() {
        let mut inner = InnerState {
            tunnels: vec![private_key_tunnel("db"), private_key_tunnel("cache")],
            runtimes: [(
                "db".to_string(),
                TunnelRuntime {
                    process: Some(spawn_process(long_running_command())),
                    last_error: Some("stale failure".into()),
                    recent_log: Vec::new(),
                    disconnect_requested: false,
                    reconnect_pending: false,
                },
            )]
            .into_iter()
            .collect(),
            recent_tunnel_ids: vec!["cache".into(), "db".into()],
        };

        let prepared = match super::prepare_connect_request(&mut inner, "db", |_| Ok(String::new()))
            .expect("prepare connect")
        {
            super::ConnectPreparation::Launch(prepared) => prepared,
            super::ConnectPreparation::MissingCredential => {
                panic!("expected launch preparation")
            }
        };

        super::apply_connected_runtime(
            &mut inner,
            prepared.tunnel.id.as_str(),
            spawn_process(long_running_command()),
        );

        let runtime = inner.runtimes.get("db").expect("runtime should remain present");
        assert!(runtime.process.is_some());
        assert_eq!(runtime.last_error, None);
        assert!(
            runtime
                .recent_log
                .iter()
                .any(|line| line.contains("stopped ssh process"))
        );
        assert!(
            runtime
                .recent_log
                .iter()
                .any(|line| line.contains("spawned ssh process"))
        );
        assert_eq!(inner.recent_tunnel_ids[0], "db");
    }

    #[test]
    fn apply_disconnect_stops_runtime_and_updates_recent_order() {
        let mut inner = InnerState {
            tunnels: vec![private_key_tunnel("db"), private_key_tunnel("cache")],
            runtimes: [(
                "db".to_string(),
                TunnelRuntime {
                    process: Some(spawn_process(long_running_command())),
                    last_error: Some("old error".into()),
                    recent_log: Vec::new(),
                    disconnect_requested: false,
                    reconnect_pending: false,
                },
            )]
            .into_iter()
            .collect(),
            recent_tunnel_ids: vec!["cache".into(), "db".into()],
        };

        super::apply_disconnect(&mut inner, "db");

        let runtime = inner.runtimes.get("db").expect("runtime should still exist");
        assert!(runtime.process.is_none());
        assert_eq!(runtime.last_error, None);
        assert!(
            runtime
                .recent_log
                .iter()
                .any(|line| line.contains("stopped ssh process"))
        );
        assert_eq!(inner.recent_tunnel_ids, vec!["db".to_string(), "cache".to_string()]);
    }

    #[test]
    fn apply_disconnect_is_idempotent_without_runtime() {
        let mut inner = InnerState {
            tunnels: vec![private_key_tunnel("db"), private_key_tunnel("cache")],
            runtimes: Default::default(),
            recent_tunnel_ids: vec!["cache".into(), "db".into()],
        };

        super::apply_disconnect(&mut inner, "db");

        let runtime = inner.runtimes.get("db").expect("disconnect should record runtime state");
        assert!(runtime.process.is_none());
        assert!(runtime.disconnect_requested);
        assert!(!runtime.reconnect_pending);
        assert_eq!(inner.recent_tunnel_ids, vec!["db".to_string(), "cache".to_string()]);
    }

    #[test]
    fn collect_auto_connect_ids_returns_only_marked_tunnels_in_order() {
        let mut db = private_key_tunnel("db");
        db.auto_connect = true;
        let cache = private_key_tunnel("cache");
        let mut metrics = private_key_tunnel("metrics");
        metrics.auto_connect = true;

        let ids = super::collect_auto_connect_ids(&[db, cache, metrics]);

        assert_eq!(ids, vec!["db".to_string(), "metrics".to_string()]);
    }

    #[test]
    fn run_connectivity_test_short_circuits_after_ssh_login_failure() {
        let prepared = super::prepare_tunnel_for_test(super::SaveTunnelPayload {
            tunnel: password_tunnel("db"),
            password: Some("s3cr3t".into()),
        })
        .expect("prepare password test payload");
        let mut seen = Vec::new();

        let (result, logs) =
            super::run_connectivity_test_with_runner(&prepared, |_, _, kind| {
                seen.push(kind);
                Ok(match kind {
                    super::ConnectivityProbeKind::SshLogin => super::ConnectivityProbeResult {
                        ok: false,
                        summary: "Permission denied".into(),
                        logs: vec!["Permission denied".into()],
                    },
                    super::ConnectivityProbeKind::TargetReachability => {
                        panic!("target probe should not run after ssh failure")
                    }
                })
            })
            .expect("run connectivity test");

        assert_eq!(seen, vec![super::ConnectivityProbeKind::SshLogin]);
        assert!(!result.ssh_ok);
        assert!(!result.target_ok);
        assert!(result.summary.contains("SSH 登录失败"));
        assert!(logs.iter().any(|line| line.contains("[测试状态] SSH 登录失败")));
    }

    #[test]
    fn run_connectivity_test_reports_target_failure_after_ssh_login_success() {
        let prepared = super::prepare_tunnel_for_test(super::SaveTunnelPayload {
            tunnel: password_tunnel("db"),
            password: Some("s3cr3t".into()),
        })
        .expect("prepare password test payload");
        let mut seen = Vec::new();

        let (result, logs) =
            super::run_connectivity_test_with_runner(&prepared, |_, _, kind| {
                seen.push(kind);
                Ok(match kind {
                    super::ConnectivityProbeKind::SshLogin => super::ConnectivityProbeResult {
                        ok: true,
                        summary: "SSH login ok".into(),
                        logs: vec!["__SSHTUNNEL_LOGIN_OK__".into()],
                    },
                    super::ConnectivityProbeKind::TargetReachability => {
                        super::ConnectivityProbeResult {
                            ok: false,
                            summary: "remote target unreachable".into(),
                            logs: vec!["Connection refused".into()],
                        }
                    }
                })
            })
            .expect("run connectivity test");

        assert_eq!(
            seen,
            vec![
                super::ConnectivityProbeKind::SshLogin,
                super::ConnectivityProbeKind::TargetReachability,
            ]
        );
        assert!(result.ssh_ok);
        assert!(!result.target_ok);
        assert!(result.summary.contains("远端目标"));
        assert!(logs.iter().any(|line| line.contains("[测试状态] SSH 登录成功")));
        assert!(logs.iter().any(|line| line.contains("[测试状态] 远端目标不可达")));
    }

    #[test]
    fn should_refresh_tray_menu_skips_rebuilding_when_signature_is_unchanged() {
        let items = vec![super::TrayTunnelItem {
            tunnel_id: "db".into(),
            title: "Database".into(),
            detail: "deploy@db.example.com".into(),
            action: super::tray_model::TrayTunnelAction::Connect,
        }];
        let mut signature = None;

        assert!(super::should_refresh_tray_menu(&mut signature, &items));
        assert!(!super::should_refresh_tray_menu(&mut signature, &items));
    }

    #[test]
    fn collect_pending_reconnect_ids_ignores_disabled_and_user_disconnected_tunnels() {
        let enabled = private_key_tunnel("db");
        let mut disabled = private_key_tunnel("cache");
        disabled.auto_reconnect = false;
        let ignored = private_key_tunnel("metrics");

        let mut inner = InnerState {
            tunnels: vec![enabled, disabled, ignored],
            runtimes: Default::default(),
            recent_tunnel_ids: vec![],
        };
        inner.runtimes.insert(
            "db".into(),
            TunnelRuntime {
                reconnect_pending: true,
                ..TunnelRuntime::default()
            },
        );
        inner.runtimes.insert(
            "cache".into(),
            TunnelRuntime {
                reconnect_pending: true,
                ..TunnelRuntime::default()
            },
        );
        inner.runtimes.insert(
            "metrics".into(),
            TunnelRuntime {
                reconnect_pending: true,
                disconnect_requested: true,
                ..TunnelRuntime::default()
            },
        );

        let ids = super::collect_pending_reconnect_ids(&inner);

        assert_eq!(ids, vec!["db".to_string()]);
    }

    #[test]
    fn apply_disconnect_marks_runtime_to_suppress_reconnect() {
        let mut inner = InnerState {
            tunnels: vec![private_key_tunnel("db")],
            runtimes: [(
                "db".to_string(),
                TunnelRuntime {
                    process: Some(spawn_process(long_running_command())),
                    last_error: None,
                    recent_log: Vec::new(),
                    disconnect_requested: false,
                    reconnect_pending: true,
                },
            )]
            .into_iter()
            .collect(),
            recent_tunnel_ids: vec![],
        };

        super::apply_disconnect(&mut inner, "db");

        let runtime = inner.runtimes.get("db").expect("db runtime should remain");
        assert!(runtime.disconnect_requested);
        assert!(!runtime.reconnect_pending);
    }

    #[test]
    fn apply_autostart_choice_uses_enable_branch_when_requested() {
        let enabled = Cell::new(false);
        let disabled = Cell::new(false);

        apply_autostart_choice(
            true,
            || {
                enabled.set(true);
                Ok::<(), &'static str>(())
            },
            || {
                disabled.set(true);
                Ok::<(), &'static str>(())
            },
        )
        .expect("enable branch");

        assert!(enabled.get());
        assert!(!disabled.get());
    }

    #[test]
    fn apply_autostart_choice_uses_disable_branch_and_preserves_error_text() {
        let enabled = Cell::new(false);
        let disabled = Cell::new(false);

        let result = apply_autostart_choice(
            false,
            || {
                enabled.set(true);
                Ok::<(), &'static str>(())
            },
            || {
                disabled.set(true);
                Err::<(), _>("disable failed")
            },
        );

        assert_eq!(result, Err("disable failed".into()));
        assert!(!enabled.get());
        assert!(disabled.get());
    }

    fn private_key_tunnel(id: &str) -> TunnelDefinition {
        TunnelDefinition {
            id: id.into(),
            name: format!("{id}-name"),
            ssh_host: "bastion.example.com".into(),
            ssh_port: 22,
            username: "deploy".into(),
            local_bind_address: "127.0.0.1".into(),
            local_bind_port: 15432,
            remote_host: "10.0.0.12".into(),
            remote_port: 5432,
            auth_kind: AuthKind::PrivateKey,
            private_key_path: Some("~/.ssh/id_ed25519".into()),
            auto_connect: false,
            auto_reconnect: true,
            password_entry: None,
        }
    }

    fn password_tunnel(id: &str) -> TunnelDefinition {
        TunnelDefinition {
            auth_kind: AuthKind::Password,
            private_key_path: None,
            password_entry: Some(format!("profile:{id}")),
            ..private_key_tunnel(id)
        }
    }

    fn spawn_process(command: CommandSpec) -> ManagedProcess {
        ManagedProcess::spawn(LaunchPlan::Native(command)).expect("spawn managed process")
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

#[cfg(test)]
mod tray_disconnect_all_tests {
    use sshtunnel_core::{
        models::{AuthKind, TunnelDefinition},
        ssh_launch::{CommandSpec, LaunchPlan},
    };

    use super::{InnerState, ManagedProcess, TunnelRuntime};

    #[test]
    fn apply_disconnect_all_stops_running_runtimes_and_clears_errors() {
        let mut inner = InnerState {
            tunnels: vec![tunnel("db"), tunnel("cache"), tunnel("metrics")],
            runtimes: [
                (
                    "db".to_string(),
                    TunnelRuntime {
                        process: Some(spawn_process(long_running_command())),
                        last_error: Some("old db error".into()),
                        recent_log: Vec::new(),
                        disconnect_requested: false,
                        reconnect_pending: false,
                    },
                ),
                (
                    "cache".to_string(),
                    TunnelRuntime {
                        process: Some(spawn_process(long_running_command())),
                        last_error: Some("old cache error".into()),
                        recent_log: Vec::new(),
                        disconnect_requested: false,
                        reconnect_pending: false,
                    },
                ),
                (
                    "metrics".to_string(),
                    TunnelRuntime {
                        process: None,
                        last_error: Some("idle warning".into()),
                        recent_log: Vec::new(),
                        disconnect_requested: false,
                        reconnect_pending: false,
                    },
                ),
            ]
            .into_iter()
            .collect(),
            recent_tunnel_ids: vec!["cache".into()],
        };

        super::apply_disconnect_all(&mut inner);

        let db = inner.runtimes.get("db").expect("db runtime");
        let cache = inner.runtimes.get("cache").expect("cache runtime");
        let metrics = inner.runtimes.get("metrics").expect("metrics runtime");

        assert!(db.process.is_none());
        assert!(cache.process.is_none());
        assert_eq!(db.last_error, None);
        assert_eq!(cache.last_error, None);
        assert_eq!(metrics.last_error, None);
        assert!(db.recent_log.iter().any(|line| line.contains("stopped ssh process")));
        assert!(cache.recent_log.iter().any(|line| line.contains("stopped ssh process")));
        assert!(!metrics.recent_log.iter().any(|line| line.contains("stopped ssh process")));
    }

    #[test]
    fn apply_disconnect_all_updates_recent_order_without_duplicates() {
        let mut inner = InnerState {
            tunnels: vec![tunnel("db"), tunnel("cache"), tunnel("metrics")],
            runtimes: Default::default(),
            recent_tunnel_ids: vec!["cache".into(), "db".into()],
        };

        super::apply_disconnect_all(&mut inner);

        assert_eq!(
            inner.recent_tunnel_ids,
            vec![
                "metrics".to_string(),
                "cache".to_string(),
                "db".to_string(),
            ]
        );
    }

    #[test]
    fn apply_disconnect_all_does_not_create_new_runtime_entries_for_idle_tunnels() {
        let mut inner = InnerState {
            tunnels: vec![tunnel("db"), tunnel("cache")],
            runtimes: [(
                "db".to_string(),
                TunnelRuntime {
                    process: Some(spawn_process(long_running_command())),
                    last_error: None,
                    recent_log: Vec::new(),
                    disconnect_requested: false,
                    reconnect_pending: false,
                },
            )]
            .into_iter()
            .collect(),
            recent_tunnel_ids: vec![],
        };

        super::apply_disconnect_all(&mut inner);

        let db = inner.runtimes.get("db").expect("db runtime");
        let cache = inner.runtimes.get("cache").expect("cache runtime");
        assert!(db.disconnect_requested);
        assert!(cache.disconnect_requested);
        assert!(db.process.is_none());
        assert!(cache.process.is_none());
        assert_eq!(inner.recent_tunnel_ids, vec!["cache".to_string(), "db".to_string()]);
    }

    fn tunnel(id: &str) -> TunnelDefinition {
        TunnelDefinition {
            id: id.into(),
            name: format!("{id}-name"),
            ssh_host: "bastion.example.com".into(),
            ssh_port: 22,
            username: "deploy".into(),
            local_bind_address: "127.0.0.1".into(),
            local_bind_port: 15432,
            remote_host: "10.0.0.12".into(),
            remote_port: 5432,
            auth_kind: AuthKind::PrivateKey,
            private_key_path: Some("~/.ssh/id_ed25519".into()),
            auto_connect: false,
            auto_reconnect: true,
            password_entry: None,
        }
    }

    fn spawn_process(command: CommandSpec) -> ManagedProcess {
        ManagedProcess::spawn(LaunchPlan::Native(command)).expect("spawn managed process")
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

#[cfg(test)]
mod tray_menu_copy_tests {
    #[test]
    fn tray_open_menu_label_uses_open_main_window_copy() {
        let source = include_str!("lib.rs");
        let production_source = source
            .split("\n#[cfg(test)]\nmod tray_menu_copy_tests")
            .next()
            .expect("production source prefix");

        assert!(
            production_source.contains(r#"MenuItemBuilder::with_id("open", "打开主界面")"#),
            "tray open action should use the approved open-main-window copy"
        );
    }

    #[test]
    fn tauri_config_declares_windows_icon_asset() {
        let config = include_str!("../tauri.conf.json");

        assert!(
            config.contains(r#""icons/icon.ico""#),
            "bundle icon config should include icons/icon.ico for Windows tray compatibility"
        );
    }
}
