const test = require("node:test");
const assert = require("node:assert/strict");

const {
  buildCommandCenterView,
  mapStatusSummaryToCommandCenterCards,
  DESKTOP_ONLY_MESSAGE,
  createDesktopBridge,
  fillPrivateKeyPath,
  shouldRefreshSnapshot,
  setEditorError,
  validateTunnelPayload,
} = require("../app.js");

function fakeMessageNode() {
  return {
    textContent: "",
    hidden: true,
    classList: {
      toggle(_className, force) {
        this.owner.hidden = force;
      },
      owner: null,
    },
  };
}

test("createDesktopBridge tolerates missing tauri globals and returns desktop-only errors", async () => {
  const bridge = createDesktopBridge({});

  assert.equal(bridge.isDesktop, false);
  await assert.rejects(() => bridge.invoke("load_state"), new Error(DESKTOP_ONLY_MESSAGE));
  await assert.rejects(() => bridge.pickPrivateKeyPath(), new Error(DESKTOP_ONLY_MESSAGE));
});

test("createDesktopBridge uses tauri dialog open and returns the selected key path", async () => {
  const bridge = createDesktopBridge({
    __TAURI__: {
      core: {
        invoke: async () => ({ ok: true }),
      },
      dialog: {
        open: async () => "/home/top/.ssh/id_ed25519",
      },
    },
  });

  assert.equal(await bridge.pickPrivateKeyPath(), "/home/top/.ssh/id_ed25519");
});

test("setEditorError renders drawer errors inline", () => {
  const editorError = fakeMessageNode();
  editorError.classList.owner = editorError;

  setEditorError({ editorError }, "保存失败：private_key_path is required");

  assert.equal(editorError.textContent, "保存失败：private_key_path is required");
  assert.equal(editorError.hidden, false);
});

test("fillPrivateKeyPath writes the selected path back into the input", () => {
  const input = { value: "" };

  fillPrivateKeyPath(input, "/home/top/.ssh/id_ed25519");

  assert.equal(input.value, "/home/top/.ssh/id_ed25519");
});

test("shouldRefreshSnapshot pauses auto refresh while the editor drawer is open", () => {
  assert.equal(shouldRefreshSnapshot({ editorOpen: true }, "visible"), false);
  assert.equal(shouldRefreshSnapshot({ editorOpen: false }, "visible"), true);
  assert.equal(shouldRefreshSnapshot({ editorOpen: false }, "hidden"), false);
});

test("validateTunnelPayload returns the first Chinese error for an empty tunnel form", () => {
  const message = validateTunnelPayload({
    tunnel: {
      name: "",
      ssh_host: "",
      ssh_port: 22,
      username: "",
      local_bind_address: "",
      local_bind_port: 15432,
      remote_host: "",
      remote_port: 5432,
      auth_kind: "private_key",
      private_key_path: null,
    },
    password: null,
  });

  assert.equal(message, "请输入名称。");
});

test("buildCommandCenterView returns empty-state render model without selected tunnel", () => {
  const view = buildCommandCenterView(
    null,
    {
      title: "选择一个隧道",
      subtitle: "从左侧选择现有隧道，或创建一个新的本地转发配置。",
      statusTone: "idle",
      statusText: "未选择",
    },
    {
      forwardLabel: "本地转发",
      forwardValue: "选择隧道后显示转发目标",
      authLabel: "认证方式",
      authValue: "选择隧道后显示认证方式",
      healthValue: "无数据",
    },
    {
      summaryText: "暂无最近日志",
      statusEvents: [{ text: "ignored", tone: "default" }],
      sshOutput: [{ text: "ignored", tone: "default" }],
      emptyStatusText: "暂无状态事件",
      emptySshText: "暂无 SSH 输出",
    },
  );

  assert.deepEqual(view, {
    title: "选择一个隧道",
    subtitle: "从左侧选择现有隧道，或创建一个新的本地转发配置。",
    statusLabel: "连接状态",
    statusTone: "idle",
    statusText: "未选择",
    forwardLabel: "本地转发",
    forwardText: "选择隧道后显示转发目标",
    forwardMuted: true,
    authLabel: "认证方式",
    authText: "选择隧道后显示认证方式",
    authMuted: true,
    authMeta: "",
    healthText: "无数据",
    healthError: "",
    logSummary: "暂无最近日志",
    statusEvents: [],
    sshOutput: [],
    emptyStatusText: "从左侧选择一条隧道后显示状态事件。",
    emptySshText: "从左侧选择一条隧道后显示 SSH 输出。",
  });
});

test("buildCommandCenterView keeps reconnect detail visible in error state", () => {
  const view = buildCommandCenterView(
    { definition: { id: "db" } },
    {
      title: "Database",
      subtitle: "deploy@bastion.example.com",
      statusTone: "error",
      statusText: "错误",
    },
    {
      forwardLabel: "本地转发",
      forwardValue: "127.0.0.1:15432 -> 10.0.0.12:5432",
      authLabel: "认证方式",
      authValue: "密钥认证",
      healthValue: "错误 — ssh exited with status code 7",
    },
    {
      summaryText: "最近 2 条状态事件，1 条 SSH 输出",
      statusEvents: [{ text: "ssh exited with status 7", tone: "error" }],
      sshOutput: [{ text: "channel 0: open failed", tone: "error" }],
      emptyStatusText: "暂无状态事件",
      emptySshText: "暂无 SSH 输出",
    },
  );

  assert.equal(view.healthText, "连接异常，请处理后重试。");
  assert.equal(view.healthError, "错误 — ssh exited with status code 7");
  assert.equal(view.forwardMuted, false);
  assert.equal(view.authMuted, false);
  assert.equal(view.statusEvents.length, 1);
  assert.equal(view.sshOutput.length, 1);
});

test("buildCommandCenterView keeps health text for healthy connected state", () => {
  const view = buildCommandCenterView(
    { definition: { id: "db" } },
    {
      title: "Database",
      subtitle: "deploy@bastion.example.com",
      statusTone: "connected",
      statusText: "已连接",
    },
    {
      forwardLabel: "本地转发",
      forwardValue: "127.0.0.1:15432 -> 10.0.0.12:5432",
      authLabel: "认证方式",
      authValue: "密钥认证",
      healthValue: "自动重连: 开启",
    },
    {
      summaryText: "最近 1 条状态事件，0 条 SSH 输出",
      statusEvents: [{ text: "spawned ssh process pid=123", tone: "default" }],
      sshOutput: [],
      emptyStatusText: "暂无状态事件",
      emptySshText: "暂无 SSH 输出",
    },
  );

  assert.equal(view.healthText, "自动重连: 开启");
  assert.equal(view.healthError, "");
  assert.equal(view.statusTone, "connected");
});

test("mapStatusSummaryToCommandCenterCards keeps reconnect detail for healthy fallback mapping", () => {
  const mapped = mapStatusSummaryToCommandCenterCards({
    forwardLabel: "本地转发",
    forwardText: "127.0.0.1:15432 -> 10.0.0.12:5432",
    authLabel: "认证方式",
    authText: "密钥认证",
    authMeta: "自动重连: 开启",
    errorText: "",
    primarySubtitle: "deploy@bastion.example.com",
  });

  assert.deepEqual(mapped, {
    forwardLabel: "本地转发",
    forwardValue: "127.0.0.1:15432 -> 10.0.0.12:5432",
    authLabel: "认证方式",
    authValue: "密钥认证",
    authMeta: "自动重连: 开启",
    healthLabel: "连接健康",
    healthValue: "自动重连: 开启",
  });
});
