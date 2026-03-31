(function (root, factory) {
  const api = factory();

  if (typeof module !== "undefined" && module.exports) {
    module.exports = api;
  }

  if (root.document && root.SshTunnelViewModel) {
    api.bootstrap(root, root.document);
  }
})(typeof globalThis !== "undefined" ? globalThis : this, () => {
  const DESKTOP_ONLY_MESSAGE = "请通过桌面应用打开此页面。";

  function toErrorMessage(error) {
    if (error instanceof Error) {
      return error.message;
    }

    return String(error);
  }

  function normalizeDialogSelection(selection) {
    if (typeof selection === "string" && selection) {
      return selection;
    }

    if (Array.isArray(selection)) {
      return selection.find((item) => typeof item === "string" && item) ?? null;
    }

    return null;
  }

  function createDesktopBridge(host) {
    const invokeImpl = host?.__TAURI__?.core?.invoke;
    const openImpl = host?.__TAURI__?.dialog?.open;

    return {
      isDesktop: typeof invokeImpl === "function",
      canPickFile: typeof openImpl === "function",
      invoke(command, args = {}) {
        if (typeof invokeImpl !== "function") {
          return Promise.reject(new Error(DESKTOP_ONLY_MESSAGE));
        }

        return invokeImpl(command, args);
      },
      async pickPrivateKeyPath() {
        if (typeof openImpl !== "function") {
          throw new Error(DESKTOP_ONLY_MESSAGE);
        }

        const selection = await openImpl({
          multiple: false,
          directory: false,
          title: "选择私钥文件",
        });

        return normalizeDialogSelection(selection);
      },
    };
  }

  function setEditorError(refs, message) {
    if (!refs?.editorError) {
      return;
    }

    refs.editorError.textContent = message || "";
    refs.editorError.classList.toggle("hidden", !message);
  }

  function clearEditorError(refs) {
    setEditorError(refs, "");
  }

  function fillPrivateKeyPath(input, path) {
    if (!input || !path) {
      return;
    }

    input.value = path;
  }

  function shouldRefreshSnapshot(state, visibilityState) {
    return visibilityState === "visible" && !state?.editorOpen;
  }

  function hasText(value) {
    return typeof value === "string" && value.trim().length > 0;
  }

  function isValidPort(value) {
    return Number.isInteger(value) && value >= 1 && value <= 65535;
  }

  function validateTunnelPayload(payload) {
    const tunnel = payload?.tunnel ?? {};
    const requiredFields = [
      [tunnel.name, "请输入名称。"],
      [tunnel.ssh_host, "请输入 SSH 主机。"],
      [tunnel.username, "请输入用户名。"],
      [tunnel.local_bind_address, "请输入本地监听地址。"],
      [tunnel.remote_host, "请输入远端目标主机。"],
    ];

    for (const [value, message] of requiredFields) {
      if (!hasText(value)) {
        return message;
      }
    }

    if (!isValidPort(tunnel.ssh_port)) {
      return "请输入有效的 SSH 端口。";
    }

    if (!isValidPort(tunnel.local_bind_port)) {
      return "请输入有效的本地端口。";
    }

    if (!isValidPort(tunnel.remote_port)) {
      return "请输入有效的远端目标端口。";
    }

    if (tunnel.auth_kind === "private_key" && !hasText(tunnel.private_key_path ?? "")) {
      return "请选择私钥文件，或手动填写私钥路径。";
    }

    if (tunnel.auth_kind === "password" && !hasText(payload?.password ?? "")) {
      return "请输入密码。";
    }

    return null;
  }

  function buildCommandCenterView(tunnel, hero, cards, timeline) {
    const hasTunnel = Boolean(tunnel);
    const statusTone = hero?.statusTone ?? "idle";
    const isError = hasTunnel && statusTone === "error";

    return {
      title: hero?.title ?? "选择一个隧道",
      subtitle: hero?.subtitle ?? "从左侧选择现有隧道，或创建一个新的本地转发配置。",
      statusLabel: "连接状态",
      statusTone,
      statusText: hero?.statusText ?? "未选择",
      forwardLabel: cards?.forwardLabel ?? "本地转发",
      forwardText: cards?.forwardValue ?? "选择隧道后显示转发目标",
      forwardMuted: !hasTunnel,
      authLabel: cards?.authLabel ?? "认证方式",
      authText: cards?.authValue ?? "选择隧道后显示认证方式",
      authMuted: !hasTunnel,
      authMeta: cards?.authMeta ?? "",
      healthText: isError ? "连接异常，请处理后重试。" : cards?.healthValue ?? "无数据",
      healthError: isError ? cards?.healthValue ?? "" : "",
      logSummary: timeline?.summaryText ?? "暂无最近日志",
      statusEvents: hasTunnel ? timeline?.statusEvents ?? [] : [],
      sshOutput: hasTunnel ? timeline?.sshOutput ?? [] : [],
      emptyStatusText: hasTunnel
        ? timeline?.emptyStatusText ?? "暂无状态事件"
        : "从左侧选择一条隧道后显示状态事件。",
      emptySshText: hasTunnel
        ? timeline?.emptySshText ?? "暂无 SSH 输出"
        : "从左侧选择一条隧道后显示 SSH 输出。",
    };
  }

  function createTunnelListNode(document, itemView, onSelect) {
    const item = document.createElement("button");
    item.type = "button";
    item.className = `tunnel-item${itemView.isActive ? " active" : ""}`;

    const title = document.createElement("h3");
    title.textContent = itemView.title;
    item.appendChild(title);

    const subtitle = document.createElement("p");
    subtitle.textContent = itemView.subtitle;
    item.appendChild(subtitle);

    const forward = document.createElement("p");
    forward.textContent = itemView.forwardText;
    item.appendChild(forward);

    const badge = document.createElement("span");
    badge.className = `badge ${itemView.badgeTone}`;
    badge.textContent = itemView.badgeText;
    item.appendChild(badge);

    item.addEventListener("click", onSelect);
    return item;
  }

  function buildTunnelListSignature(tunnels, selectedId, describeTunnelListItem) {
    return JSON.stringify(
      (tunnels ?? []).map((tunnel) => {
        const itemView = describeTunnelListItem(tunnel, selectedId);
        return {
          id: tunnel?.definition?.id ?? "",
          title: itemView.title,
          subtitle: itemView.subtitle,
          forwardText: itemView.forwardText,
          badgeTone: itemView.badgeTone,
          badgeText: itemView.badgeText,
          isActive: itemView.isActive,
        };
      }),
    );
  }

  function renderTunnelList(
    list,
    document,
    tunnels,
    selectedId,
    describeTunnelListItem,
    onSelectById,
    previousSignature,
  ) {
    const signature = buildTunnelListSignature(tunnels, selectedId, describeTunnelListItem);
    if (signature === previousSignature) {
      return signature;
    }

    list.innerHTML = "";

    for (const tunnel of tunnels ?? []) {
      const itemView = describeTunnelListItem(tunnel, selectedId);
      const item = createTunnelListNode(document, itemView, () => {
        onSelectById(tunnel.definition.id);
      });
      list.appendChild(item);
    }

    return signature;
  }

  function mapStatusSummaryToCommandCenterCards(summary) {
    const reconnectText = summary?.authMeta || "无数据";
    const errorText = summary?.errorText || "";

    return {
      forwardLabel: summary?.forwardLabel,
      forwardValue: summary?.forwardText,
      authLabel: summary?.authLabel,
      authValue: summary?.authText,
      authMeta: summary?.authMeta,
      healthLabel: "连接健康",
      healthValue: errorText ? `错误 — ${errorText}` : reconnectText,
    };
  }

  function bootstrap(host, document) {
    const bridge = createDesktopBridge(host);
    const invoke = (command, args = {}) => bridge.invoke(command, args);
    const {
      describeTunnelActions,
      describeTunnelListItem,
      describeEditorSheet,
      describeWorkspacePanel,
      describeStatusSummaryCards,
      describeDiagnosticLogPanel,
      describeCommandCenterHero: viewModelDescribeCommandCenterHero,
      describeCommandCenterCards: viewModelDescribeCommandCenterCards,
      describeCommandCenterTimeline: viewModelDescribeCommandCenterTimeline,
      summarizeSnapshotMeta,
    } = host.SshTunnelViewModel;

    const describeCommandCenterHero = viewModelDescribeCommandCenterHero ?? describeWorkspacePanel;
    const describeCommandCenterCards =
      viewModelDescribeCommandCenterCards ??
      ((tunnel) => {
        const summary = describeStatusSummaryCards(tunnel);
        return mapStatusSummaryToCommandCenterCards(summary);
      });
    const describeCommandCenterTimeline =
      viewModelDescribeCommandCenterTimeline ?? describeDiagnosticLogPanel;

    const state = {
      snapshot: null,
      selectedId: null,
      editorOpen: false,
      editingId: null,
      listSignature: null,
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
      editorError: document.getElementById("editor-error"),
      pickPrivateKey: document.getElementById("pick-private-key"),
      privateKeyPath: document.getElementById("private-key-path"),
      authKind: document.getElementById("auth-kind"),
      password: document.getElementById("password"),
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

    function renderWorkspace(tunnel) {
      const hero = describeCommandCenterHero(tunnel);
      const cards = describeCommandCenterCards(tunnel);
      const timeline = describeCommandCenterTimeline(tunnel?.recent_log ?? []);
      const view = buildCommandCenterView(tunnel, hero, cards, timeline);

      refs.workspaceTitle.textContent = view.title;
      refs.workspaceSubtitle.textContent = view.subtitle;
      refs.statusLabel.textContent = view.statusLabel;
      refs.statusCard.className = `status-card ${view.statusTone}`;
      refs.statusCard.textContent = view.statusText;
      refs.statusSubtitle.textContent = view.healthText;
      refs.statusError.textContent = view.healthError;
      refs.statusError.classList.toggle("hidden", !view.healthError);
      refs.forwardLabel.textContent = view.forwardLabel;
      refs.forwardCard.textContent = view.forwardText;
      refs.forwardCard.className = view.forwardMuted ? "summary-value muted" : "summary-value";
      refs.authLabel.textContent = view.authLabel;
      refs.authCard.textContent = view.authText;
      refs.authCard.className = view.authMuted ? "summary-value muted" : "summary-value";
      refs.authMeta.textContent = view.authMeta;
      refs.authMeta.classList.toggle("hidden", !view.authMeta);
      refs.logSummary.textContent = view.logSummary;

      renderLogSection(refs.statusEventsLog, view.statusEvents, view.emptyStatusText);
      renderLogSection(refs.sshOutputLog, view.sshOutput, view.emptySshText);
    }

    function fillForm(tunnel) {
      const editorCopy = describeEditorSheet(tunnel);
      refs.formTitle.textContent = editorCopy.title;
      refs.saveTunnel.textContent = editorCopy.submitText;
      document.getElementById("tunnel-id").value = tunnel?.definition.id ?? "";
      document.getElementById("name").value = tunnel?.definition.name ?? "";
      refs.authKind.value = tunnel?.definition.auth_kind ?? "private_key";
      document.getElementById("ssh-host").value = tunnel?.definition.ssh_host ?? "";
      document.getElementById("ssh-port").value = tunnel?.definition.ssh_port ?? 22;
      document.getElementById("username").value = tunnel?.definition.username ?? "";
      refs.privateKeyPath.value = tunnel?.definition.private_key_path ?? "";
      document.getElementById("local-bind-address").value =
        tunnel?.definition.local_bind_address ?? "127.0.0.1";
      document.getElementById("local-bind-port").value = tunnel?.definition.local_bind_port ?? 15432;
      document.getElementById("remote-host").value = tunnel?.definition.remote_host ?? "";
      document.getElementById("remote-port").value = tunnel?.definition.remote_port ?? 5432;
      refs.password.value = "";
      document.getElementById("auto-connect").checked = tunnel?.definition.auto_connect ?? false;
      document.getElementById("auto-reconnect").checked = tunnel?.definition.auto_reconnect ?? true;
      syncAuthFields();
    }

    function openEditor(tunnel) {
      state.editorOpen = true;
      state.editingId = tunnel?.definition.id ?? null;
      clearEditorError(refs);
      fillForm(tunnel);
      refs.drawerBackdrop.classList.remove("hidden");
      refs.editorDrawer.classList.remove("hidden");
      refs.editorDrawer.setAttribute("aria-hidden", "false");
      refs.drawerBackdrop.setAttribute("aria-hidden", "false");
    }

    function closeEditor() {
      state.editorOpen = false;
      state.editingId = null;
      clearEditorError(refs);
      refs.drawerBackdrop.classList.add("hidden");
      refs.editorDrawer.classList.add("hidden");
      refs.editorDrawer.setAttribute("aria-hidden", "true");
      refs.drawerBackdrop.setAttribute("aria-hidden", "true");
    }

    function renderList() {
      state.listSignature = renderTunnelList(
        refs.list,
        document,
        state.snapshot?.tunnels ?? [],
        state.selectedId,
        describeTunnelListItem,
        (id) => {
          state.selectedId = id;
          render();
        },
        state.listSignature,
      );
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
      }
    }

    async function refresh() {
      try {
        state.snapshot = await invoke("load_state");
        render();
      } catch (error) {
        refs.statusCard.className = "status-card error";
        refs.statusCard.textContent = toErrorMessage(error);
      }
    }

    function formPayload() {
      const authKind = refs.authKind.value;
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
          private_key_path: refs.privateKeyPath.value.trim() || null,
          auto_connect: document.getElementById("auto-connect").checked,
          auto_reconnect: document.getElementById("auto-reconnect").checked,
          password_entry: id ? `profile:${id}` : null,
        },
        password: refs.password.value || null,
      };
    }

    function syncAuthFields() {
      const isPrivateKey = refs.authKind.value === "private_key";
      refs.privateKeyPath.disabled = !isPrivateKey;
      refs.password.disabled = isPrivateKey;
      refs.pickPrivateKey.disabled = !isPrivateKey || !bridge.canPickFile;
    }

    refs.form.addEventListener("submit", async (event) => {
      event.preventDefault();
      clearEditorError(refs);

      try {
        const payload = formPayload();
        const validationError = validateTunnelPayload(payload);

        if (validationError) {
          setEditorError(refs, validationError);
          return;
        }

        state.snapshot = await invoke("save_tunnel", { payload });
        const savedId = payload.tunnel.id || state.snapshot.tunnels.at(-1)?.definition.id;
        state.selectedId =
          state.snapshot.tunnels.find((item) => item.definition.id === savedId)?.definition.id ??
          state.snapshot.tunnels.at(-1)?.definition.id ??
          state.selectedId;
        closeEditor();
        render();
      } catch (error) {
        setEditorError(refs, toErrorMessage(error));
      }
    });

    refs.newTunnel.addEventListener("click", () => {
      openEditor(null);
    });

    refs.autostartToggle.addEventListener("click", async () => {
      state.snapshot = await invoke("set_autostart", {
        enabled: !Boolean(state.snapshot?.autostart_enabled),
      });
      render();
    });

    refs.connectBtn.addEventListener("click", async () => {
      if (!state.selectedId) {
        return;
      }
      state.snapshot = await invoke("connect_tunnel", { id: state.selectedId });
      render();
    });

    refs.disconnectBtn.addEventListener("click", async () => {
      if (!state.selectedId) {
        return;
      }
      state.snapshot = await invoke("disconnect_tunnel", { id: state.selectedId });
      render();
    });

    refs.editBtn.addEventListener("click", () => {
      const tunnel = currentTunnel();
      if (!tunnel) {
        return;
      }
      openEditor(tunnel);
    });

    refs.deleteBtn.addEventListener("click", async () => {
      if (!state.selectedId) {
        return;
      }
      state.snapshot = await invoke("delete_tunnel", { id: state.selectedId });
      state.selectedId = state.snapshot.tunnels[0]?.definition.id ?? null;
      render();
    });

    refs.pickPrivateKey.addEventListener("click", async () => {
      clearEditorError(refs);
      try {
        const selectedPath = await bridge.pickPrivateKeyPath();
        fillPrivateKeyPath(refs.privateKeyPath, selectedPath);
      } catch (error) {
        setEditorError(refs, toErrorMessage(error));
      }
    });

    refs.closeDrawer.addEventListener("click", closeEditor);
    refs.cancelEdit.addEventListener("click", closeEditor);
    refs.drawerBackdrop.addEventListener("click", closeEditor);
    refs.authKind.addEventListener("change", syncAuthFields);

    setInterval(() => {
      if (shouldRefreshSnapshot(state, document.visibilityState)) {
        refresh();
      }
    }, 4000);

    refresh();
  }

  return {
    buildCommandCenterView,
    createTunnelListNode,
    renderTunnelList,
    mapStatusSummaryToCommandCenterCards,
    DESKTOP_ONLY_MESSAGE,
    bootstrap,
    clearEditorError,
    createDesktopBridge,
    fillPrivateKeyPath,
    normalizeDialogSelection,
    shouldRefreshSnapshot,
    setEditorError,
    validateTunnelPayload,
  };
});
