const invoke = (command, args = {}) => window.__TAURI__.core.invoke(command, args);
const {
  describeTunnelActions,
  describeTunnelListItem,
  describeTunnelStatus,
  describeDiagnosticLogPanel,
  describeEditorSheet,
  describeStatusSummaryCards,
  describeWorkspacePanel,
  summarizeSnapshotMeta,
} = window.SshTunnelViewModel;

const state = {
  snapshot: null,
  selectedId: null,
  editorOpen: false,
  editingId: null,
};

const refs = {
  list: document.getElementById("tunnel-list"),
  sshStatus: document.getElementById("ssh-status"),
  autostartStatus: document.getElementById("autostart-status"),
  autostartToggle: document.getElementById("autostart-toggle"),
  configPath: document.getElementById("config-path"),
  workspaceTitle: document.getElementById("workspace-title"),
  workspaceSubtitle: document.getElementById("workspace-subtitle"),
  formTitle: document.getElementById("form-title"),
  form: document.getElementById("tunnel-form"),
  saveTunnel: document.getElementById("save-tunnel"),
  statusLabel: document.getElementById("status-label"),
  statusCard: document.getElementById("status-card"),
  statusSubtitle: document.getElementById("status-subtitle"),
  statusError: document.getElementById("status-error"),
  forwardLabel: document.getElementById("forward-label"),
  forwardCard: document.getElementById("forward-card"),
  authLabel: document.getElementById("auth-label"),
  authCard: document.getElementById("auth-card"),
  authMeta: document.getElementById("auth-meta"),
  logSummary: document.getElementById("log-summary"),
  statusEventsLog: document.getElementById("status-events-log"),
  sshOutputLog: document.getElementById("ssh-output-log"),
  newTunnel: document.getElementById("new-tunnel"),
  connectBtn: document.getElementById("connect-btn"),
  disconnectBtn: document.getElementById("disconnect-btn"),
  editBtn: document.getElementById("edit-btn"),
  deleteBtn: document.getElementById("delete-btn"),
  drawerBackdrop: document.getElementById("drawer-backdrop"),
  editorDrawer: document.getElementById("editor-drawer"),
  closeDrawer: document.getElementById("close-drawer"),
  cancelEdit: document.getElementById("cancel-edit"),
};

function currentTunnel() {
  return state.snapshot?.tunnels.find((item) => item.definition.id === state.selectedId) ?? null;
}

function currentEditingTunnel() {
  if (!state.editingId) {
    return null;
  }

  return state.snapshot?.tunnels.find((item) => item.definition.id === state.editingId) ?? null;
}

function renderWorkspace(tunnel) {
  const workspace = describeWorkspacePanel(tunnel);
  const summary = describeStatusSummaryCards(tunnel);
  const logPanel = describeDiagnosticLogPanel(tunnel?.recent_log ?? []);
  refs.workspaceTitle.textContent = workspace.title;
  refs.workspaceSubtitle.textContent = workspace.subtitle;
  refs.statusLabel.textContent = summary.primaryLabel;
  refs.statusCard.className = `status-card ${summary.primaryTone}`;
  refs.statusCard.textContent = summary.primaryText;
  refs.statusSubtitle.textContent = summary.primarySubtitle;
  refs.forwardLabel.textContent = summary.forwardLabel;
  refs.forwardCard.textContent = summary.forwardText;
  refs.authLabel.textContent = summary.authLabel;
  refs.authCard.textContent = summary.authText;
  refs.authMeta.textContent = summary.authMeta;
  refs.authMeta.classList.toggle("hidden", !summary.authMeta);
  refs.statusError.textContent = summary.errorText;
  refs.statusError.classList.toggle("hidden", !summary.errorText);
  refs.logSummary.textContent = logPanel.summaryText;

  if (!tunnel) {
    refs.forwardCard.className = "summary-value muted";
    refs.authCard.className = "summary-value muted";
    renderLogSection(refs.statusEventsLog, [], "从左侧选择一条隧道后显示状态事件。");
    renderLogSection(refs.sshOutputLog, [], "从左侧选择一条隧道后显示 SSH 输出。");
    return;
  }

  refs.forwardCard.className = "summary-value";
  refs.authCard.className = "summary-value";
  renderLogSection(refs.statusEventsLog, logPanel.statusEvents, logPanel.emptyStatusText);
  renderLogSection(refs.sshOutputLog, logPanel.sshOutput, logPanel.emptySshText);
}

function renderLogSection(container, entries, emptyText) {
  container.innerHTML = "";
  if (!entries.length) {
    const empty = document.createElement("div");
    empty.className = "log-line";
    empty.textContent = emptyText;
    container.appendChild(empty);
    return;
  }

  for (const entry of entries) {
    const div = document.createElement("div");
    div.className = `log-line${entry.tone === "error" ? " error" : ""}`;
    div.textContent = entry.text;
    container.appendChild(div);
  }
}

function fillForm(tunnel) {
  const editorCopy = describeEditorSheet(tunnel);
  refs.formTitle.textContent = editorCopy.title;
  refs.saveTunnel.textContent = editorCopy.submitText;
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

function openEditor(tunnel) {
  state.editorOpen = true;
  state.editingId = tunnel?.definition.id ?? null;
  fillForm(tunnel);
  refs.drawerBackdrop.classList.remove("hidden");
  refs.editorDrawer.classList.remove("hidden");
  refs.editorDrawer.setAttribute("aria-hidden", "false");
  refs.drawerBackdrop.setAttribute("aria-hidden", "false");
}

function closeEditor() {
  state.editorOpen = false;
  state.editingId = null;
  refs.drawerBackdrop.classList.add("hidden");
  refs.editorDrawer.classList.add("hidden");
  refs.editorDrawer.setAttribute("aria-hidden", "true");
  refs.drawerBackdrop.setAttribute("aria-hidden", "true");
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
  refs.editBtn.disabled = !tunnel;
  refs.deleteBtn.disabled = !tunnel;
  renderList();
  renderWorkspace(tunnel);

  if (!state.editorOpen) {
    closeEditor();
  } else {
    fillForm(currentEditingTunnel());
  }
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
  const savedId = payload.tunnel.id || state.snapshot.tunnels.at(-1)?.definition.id;
  state.selectedId = state.snapshot.tunnels.find((item) => item.definition.id === savedId)?.definition.id
    ?? state.snapshot.tunnels.at(-1)?.definition.id
    ?? state.selectedId;
  closeEditor();
  render();
});

refs.newTunnel.addEventListener("click", () => {
  openEditor(null);
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

refs.editBtn.addEventListener("click", () => {
  const tunnel = currentTunnel();
  if (!tunnel) return;
  openEditor(tunnel);
});

refs.deleteBtn.addEventListener("click", async () => {
  if (!state.selectedId) return;
  state.snapshot = await invoke("delete_tunnel", { id: state.selectedId });
  state.selectedId = state.snapshot.tunnels[0]?.definition.id ?? null;
  render();
});

refs.closeDrawer.addEventListener("click", closeEditor);
refs.cancelEdit.addEventListener("click", closeEditor);
refs.drawerBackdrop.addEventListener("click", closeEditor);

document.getElementById("auth-kind").addEventListener("change", syncAuthFields);

setInterval(() => {
  if (document.visibilityState === "visible") {
    refresh();
  }
}, 4000);

refresh();
