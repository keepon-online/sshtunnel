const invoke = (command, args = {}) => window.__TAURI__.core.invoke(command, args);
const {
  describeTunnelActions,
  describeTunnelListItem,
  describeTunnelStatus,
  summarizeSnapshotMeta,
} = window.SshTunnelViewModel;

const state = {
  snapshot: null,
  selectedId: null,
};

const refs = {
  list: document.getElementById("tunnel-list"),
  sshStatus: document.getElementById("ssh-status"),
  autostartStatus: document.getElementById("autostart-status"),
  autostartToggle: document.getElementById("autostart-toggle"),
  configPath: document.getElementById("config-path"),
  formTitle: document.getElementById("form-title"),
  form: document.getElementById("tunnel-form"),
  statusCard: document.getElementById("status-card"),
  recentLog: document.getElementById("recent-log"),
  newTunnel: document.getElementById("new-tunnel"),
  connectBtn: document.getElementById("connect-btn"),
  disconnectBtn: document.getElementById("disconnect-btn"),
  deleteBtn: document.getElementById("delete-btn"),
};

function currentTunnel() {
  return state.snapshot?.tunnels.find((item) => item.definition.id === state.selectedId) ?? null;
}

function setStatusCard(tunnel) {
  if (!tunnel) {
    refs.statusCard.className = "status-card idle";
    refs.statusCard.textContent = "未选择隧道";
    refs.recentLog.innerHTML = "";
    return;
  }

  const statusCopy = describeTunnelStatus(tunnel.status);
  refs.statusCard.className = `status-card ${statusCopy.tone}`;
  refs.statusCard.textContent = [
    `状态: ${statusCopy.text}`,
    `SSH: ${tunnel.definition.username}@${tunnel.definition.ssh_host}`,
    `转发: ${tunnel.definition.local_bind_address}:${tunnel.definition.local_bind_port} -> ${tunnel.definition.remote_host}:${tunnel.definition.remote_port}`,
    tunnel.last_error ? `错误: ${tunnel.last_error}` : null,
  ].filter(Boolean).join(" | ");

  refs.recentLog.innerHTML = "";
  const lines = tunnel.recent_log.length ? tunnel.recent_log : ["暂无日志"];
  for (const line of lines) {
    const div = document.createElement("div");
    div.className = "log-line";
    div.textContent = line;
    refs.recentLog.appendChild(div);
  }
}

function fillForm(tunnel) {
  refs.formTitle.textContent = tunnel ? `编辑: ${tunnel.definition.name}` : "新建隧道";
  document.getElementById("tunnel-id").value = tunnel?.definition.id ?? "";
  document.getElementById("name").value = tunnel?.definition.name ?? "";
  document.getElementById("auth-kind").value = tunnel?.definition.auth_kind ?? "private_key";
  document.getElementById("ssh-host").value = tunnel?.definition.ssh_host ?? "";
  document.getElementById("ssh-port").value = tunnel?.definition.ssh_port ?? 22;
  document.getElementById("username").value = tunnel?.definition.username ?? "";
  document.getElementById("private-key-path").value = tunnel?.definition.private_key_path ?? "";
  document.getElementById("local-bind-address").value = tunnel?.definition.local_bind_address ?? "127.0.0.1";
  document.getElementById("local-bind-port").value = tunnel?.definition.local_bind_port ?? 15432;
  document.getElementById("remote-host").value = tunnel?.definition.remote_host ?? "";
  document.getElementById("remote-port").value = tunnel?.definition.remote_port ?? 5432;
  document.getElementById("password").value = "";
  document.getElementById("auto-connect").checked = tunnel?.definition.auto_connect ?? false;
  document.getElementById("auto-reconnect").checked = tunnel?.definition.auto_reconnect ?? true;
  syncAuthFields();
}

function renderList() {
  refs.list.innerHTML = "";

  for (const tunnel of state.snapshot?.tunnels ?? []) {
    const itemView = describeTunnelListItem(tunnel, state.selectedId);
    const item = document.createElement("button");
    item.type = "button";
    item.className = `tunnel-item${itemView.isActive ? " active" : ""}`;
    item.innerHTML = `
      <h3>${itemView.title}</h3>
      <p>${itemView.subtitle}</p>
      <p>${itemView.forwardText}</p>
      <span class="badge ${itemView.badgeTone}">${itemView.badgeText}</span>
    `;
    item.addEventListener("click", () => {
      state.selectedId = tunnel.definition.id;
      render();
    });
    refs.list.appendChild(item);
  }
}

function render() {
  const meta = summarizeSnapshotMeta(state.snapshot);
  refs.sshStatus.textContent = meta.sshText;
  refs.autostartStatus.textContent = meta.autostartText;
  refs.autostartToggle.textContent = meta.autostartAction;
  refs.configPath.textContent = meta.configPath;

  if (!state.selectedId && state.snapshot?.tunnels?.length) {
    state.selectedId = state.snapshot.tunnels[0].definition.id;
  }

  const tunnel = currentTunnel();
  const actions = describeTunnelActions(tunnel);
  refs.connectBtn.textContent = actions.connectText;
  refs.connectBtn.disabled = actions.connectDisabled;
  refs.disconnectBtn.textContent = actions.disconnectText;
  refs.disconnectBtn.disabled = actions.disconnectDisabled;
  renderList();
  fillForm(tunnel);
  setStatusCard(tunnel);
}

async function refresh() {
  try {
    state.snapshot = await invoke("load_state");
    render();
  } catch (error) {
    refs.statusCard.className = "status-card error";
    refs.statusCard.textContent = String(error);
  }
}

function formPayload() {
  const authKind = document.getElementById("auth-kind").value;
  const id = document.getElementById("tunnel-id").value.trim();

  return {
    tunnel: {
      id,
      name: document.getElementById("name").value.trim(),
      ssh_host: document.getElementById("ssh-host").value.trim(),
      ssh_port: Number(document.getElementById("ssh-port").value),
      username: document.getElementById("username").value.trim(),
      local_bind_address: document.getElementById("local-bind-address").value.trim(),
      local_bind_port: Number(document.getElementById("local-bind-port").value),
      remote_host: document.getElementById("remote-host").value.trim(),
      remote_port: Number(document.getElementById("remote-port").value),
      auth_kind: authKind,
      private_key_path: document.getElementById("private-key-path").value.trim() || null,
      auto_connect: document.getElementById("auto-connect").checked,
      auto_reconnect: document.getElementById("auto-reconnect").checked,
      password_entry: id ? `profile:${id}` : null,
    },
    password: document.getElementById("password").value || null,
  };
}

function syncAuthFields() {
  const authKind = document.getElementById("auth-kind").value;
  document.getElementById("private-key-path").disabled = authKind !== "private_key";
  document.getElementById("password").disabled = authKind !== "password";
}

refs.form.addEventListener("submit", async (event) => {
  event.preventDefault();

  const payload = formPayload();
  state.snapshot = await invoke("save_tunnel", { payload });
  state.selectedId = state.snapshot.tunnels.at(-1)?.definition.id ?? state.selectedId;
  render();
});

refs.newTunnel.addEventListener("click", () => {
  state.selectedId = null;
  fillForm(null);
  setStatusCard(null);
  renderList();
});

refs.autostartToggle.addEventListener("click", async () => {
  const nextValue = !Boolean(state.snapshot?.autostart_enabled);
  state.snapshot = await invoke("set_autostart", { enabled: nextValue });
  render();
});

refs.connectBtn.addEventListener("click", async () => {
  if (!state.selectedId) return;
  state.snapshot = await invoke("connect_tunnel", { id: state.selectedId });
  render();
});

refs.disconnectBtn.addEventListener("click", async () => {
  if (!state.selectedId) return;
  state.snapshot = await invoke("disconnect_tunnel", { id: state.selectedId });
  render();
});

refs.deleteBtn.addEventListener("click", async () => {
  if (!state.selectedId) return;
  state.snapshot = await invoke("delete_tunnel", { id: state.selectedId });
  state.selectedId = state.snapshot.tunnels[0]?.definition.id ?? null;
  render();
});

document.getElementById("auth-kind").addEventListener("change", syncAuthFields);

setInterval(() => {
  if (document.visibilityState === "visible") {
    refresh();
  }
}, 4000);

refresh();
