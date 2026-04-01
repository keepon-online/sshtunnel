const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const {
  buildCommandCenterView,
  describeConnectivityResult,
  mapStatusSummaryToCommandCenterCards,
  DESKTOP_ONLY_MESSAGE,
  createTunnelListNode,
  filterTunnels,
  renderTunnelList,
  renderLogSection,
  createDesktopBridge,
  fillPrivateKeyPath,
  mergeTimelineLogs,
  shouldRefreshSnapshot,
  shouldAutoFollowLogPanel,
  scrollLogPanelToLatest,
  setEditorError,
  validateTunnelPayload,
} = require("../app.js");
const { describeDiagnosticLogPanel } = require("../view-model.js");

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

function fakeElement(tagName) {
  return {
    tagName,
    type: "",
    className: "",
    textContent: "",
    children: [],
    listeners: {},
    addEventListener(name, handler) {
      this.listeners[name] = handler;
    },
    appendChild(child) {
      this.children.push(child);
      return child;
    },
  };
}

function fakeDocument() {
  return {
    createElement(tagName) {
      return fakeElement(tagName);
    },
  };
}

function fakeList() {
  return {
    children: [],
    resetCount: 0,
    appendChild(child) {
      this.children.push(child);
      return child;
    },
    set innerHTML(value) {
      this.resetCount += 1;
      this.children = [];
      this._innerHTML = value;
    },
    get innerHTML() {
      return this._innerHTML || "";
    },
  };
}

function fakeScrollableList() {
  return {
    children: [],
    resetCount: 0,
    scrollTop: 0,
    scrollHeight: 0,
    clientHeight: 120,
    appendChild(child) {
      this.children.push(child);
      this.scrollHeight = this.children.length * 48;
      return child;
    },
    set innerHTML(value) {
      this.resetCount += 1;
      this.children = [];
      this.scrollHeight = 0;
      this._innerHTML = value;
    },
    get innerHTML() {
      return this._innerHTML || "";
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

test("createTunnelListNode renders tunnel fields as text nodes instead of HTML", () => {
  const item = createTunnelListNode(
    fakeDocument(),
    {
      title: '<img src=x onerror="alert(1)">',
      subtitle: "deploy@bastion.example.com",
      forwardText: "127.0.0.1:15432 -> db.internal:5432",
      badgeTone: "connected",
      badgeText: "<b>已连接</b>",
      isActive: true,
    },
    () => {},
  );

  assert.equal(item.tagName, "button");
  assert.equal(item.className, "tunnel-item active");
  assert.equal(item.children[0].tagName, "h3");
  assert.equal(item.children[0].textContent, '<img src=x onerror="alert(1)">');
  assert.equal(item.children[3].tagName, "span");
  assert.equal(item.children[3].textContent, "<b>已连接</b>");
});

test("renderTunnelList skips DOM rebuild when tunnel list signature is unchanged", () => {
  const list = fakeList();
  const document = fakeDocument();
  const tunnels = [
    {
      definition: {
        id: "db",
      },
      status: "idle",
    },
  ];
  const describeTunnelListItem = (tunnel, selectedId) => ({
    title: `title-${tunnel.definition.id}`,
    subtitle: "deploy@bastion.example.com",
    forwardText: "127.0.0.1:15432 -> db.internal:5432",
    badgeTone: tunnel.status,
    badgeText: tunnel.status,
    isActive: tunnel.definition.id === selectedId,
  });

  const firstSignature = renderTunnelList(
    list,
    document,
    tunnels,
    "db",
    describeTunnelListItem,
    () => {},
    null,
    "",
    "all",
  );

  assert.equal(list.resetCount, 1);
  assert.equal(list.children.length, 1);

  const secondSignature = renderTunnelList(
    list,
    document,
    tunnels,
    "db",
    describeTunnelListItem,
    () => {},
    firstSignature,
    "",
    "all",
  );

  assert.equal(secondSignature, firstSignature);
  assert.equal(list.resetCount, 1);
  assert.equal(list.children.length, 1);
});

test("scrollLogPanelToLatest keeps the newest log line in view", () => {
  const container = {
    scrollTop: 0,
    scrollHeight: 420,
    clientHeight: 120,
  };

  scrollLogPanelToLatest(container);

  assert.equal(container.scrollTop, 300);
});

test("shouldAutoFollowLogPanel stays attached when the reader is near the tail", () => {
  assert.equal(
    shouldAutoFollowLogPanel({
      scrollTop: 458,
      scrollHeight: 600,
      clientHeight: 120,
    }),
    true,
  );
});

test("shouldAutoFollowLogPanel preserves manual scroll position away from the tail", () => {
  assert.equal(
    shouldAutoFollowLogPanel({
      scrollTop: 120,
      scrollHeight: 600,
      clientHeight: 120,
    }),
    false,
  );
});

test("renderLogSection appends log entries and follows the tail", () => {
  const container = fakeScrollableList();
  container.scrollTop = 300;
  container.scrollHeight = 420;

  renderLogSection(
    container,
    [
      { text: "spawned ssh process pid=123", tone: "default" },
      { text: "password sent to interactive ssh session", tone: "default" },
      { text: "channel 0: open failed: Connection refused", tone: "error" },
      { text: "ssh exited with status code 255", tone: "error" },
    ],
    "暂无 SSH 输出",
    fakeDocument(),
  );

  assert.equal(container.children.length, 4);
  assert.equal(container.children[3].className, "log-line error");
  assert.equal(container.children[3].textContent, "ssh exited with status code 255");
  assert.equal(container.scrollTop, 72);
});

test("renderLogSection does not steal scroll position when reading history", () => {
  const container = fakeScrollableList();
  container.scrollTop = 96;
  container.scrollHeight = 720;

  renderLogSection(
    container,
    [
      { text: "line 1", tone: "default" },
      { text: "line 2", tone: "default" },
      { text: "line 3", tone: "default" },
      { text: "line 4", tone: "default" },
      { text: "line 5", tone: "default" },
    ],
    "暂无 SSH 输出",
    fakeDocument(),
  );

  assert.equal(container.scrollTop, 96);
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
      statusEvents: [],
      sshOutput: [],
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

test("buildCommandCenterView keeps timeline visible for app-level test logs without a selected tunnel", () => {
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
      summaryText: "最近 1 条状态事件，1 条 SSH 输出",
      statusEvents: [{ text: "[测试状态] SSH 登录成功", tone: "default" }],
      sshOutput: [{ text: "[测试输出] Connection refused", tone: "error" }],
      emptyStatusText: "暂无状态事件",
      emptySshText: "暂无 SSH 输出",
    },
  );

  assert.equal(view.statusEvents.length, 1);
  assert.equal(view.sshOutput.length, 1);
  assert.equal(view.emptyStatusText, "暂无状态事件");
  assert.equal(view.emptySshText, "暂无 SSH 输出");
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

test("mergeTimelineLogs appends app-level test logs after tunnel logs", () => {
  assert.deepEqual(mergeTimelineLogs(["spawned ssh process"], ["[测试状态] SSH 登录成功"]), [
    "spawned ssh process",
    "[测试状态] SSH 登录成功",
  ]);
});

test("describeConnectivityResult formats drawer copy for a failed probe", () => {
  assert.deepEqual(
    describeConnectivityResult({
      ssh_ok: true,
      target_ok: false,
      summary: "SSH 登录成功，但远端目标不可达：Connection refused",
      ssh_summary: "已完成 SSH 握手",
      target_summary: "Connection refused",
    }),
    {
      tone: "error",
      title: "SSH 登录成功，但远端目标不可达：Connection refused",
      details: ["SSH 登录：已完成 SSH 握手", "目标检查：Connection refused"],
    },
  );
});

test("describeDiagnosticLogPanel routes connectivity test status and output lines into separate lanes", () => {
  const panel = describeDiagnosticLogPanel([
    "[测试状态] SSH 登录成功",
    "[测试输出] Connection refused",
  ]);

  assert.equal(panel.statusEvents.length, 1);
  assert.equal(panel.statusEvents[0].text, "[测试状态] SSH 登录成功");
  assert.equal(panel.sshOutput.length, 1);
  assert.equal(panel.sshOutput[0].tone, "error");
});

test("styles keep select controls readable in dark theme", () => {
  const css = fs.readFileSync(path.join(__dirname, "../styles.css"), "utf8");

  assert.match(css, /:root,\s*:root\[data-theme="dark"\]\s*\{[\s\S]*--select-bg-color:\s*#111827;/);
  assert.match(css, /select\s*\{[\s\S]*appearance:\s*none;/);
  assert.match(css, /select\s*\{[\s\S]*-webkit-appearance:\s*none;/);
  assert.match(css, /input,\s*[\r\n\s]*select\s*\{[\s\S]*background:\s*var\(--surface-layer\)/);
  assert.match(css, /input,\s*[\r\n\s]*select\s*\{[\s\S]*color:\s*var\(--ink\)/);
  assert.match(css, /select\s*\{[\s\S]*var\(--surface-layer\);/);
  assert.match(css, /select option\s*\{[\s\S]*background:\s*var\(--select-bg-color\)/);
  assert.match(css, /select option\s*\{[\s\S]*color:\s*var\(--ink\)/);
  assert.match(css, /input:disabled,\s*[\r\n\s]*select:disabled\s*\{/);
  assert.match(css, /input:disabled,\s*[\r\n\s]*select:disabled\s*\{[\s\S]*color:\s*var\(--muted\)/);
  assert.match(css, /input:disabled,\s*[\r\n\s]*select:disabled\s*\{[\s\S]*background:\s*var\(--surface-layer\)/);
  assert.match(css, /label span\s*\{[\s\S]*color:\s*var\(--ink\)/);
  assert.match(css, /input::placeholder,\s*[\r\n\s]*select::placeholder\s*\{[\s\S]*color:\s*var\(--muted\)/);
});

test("styles keep log panels scrollable and tall enough for recent output", () => {
  const css = fs.readFileSync(path.join(__dirname, "../styles.css"), "utf8");

  assert.match(css, /\.log-panel\s*\{[\s\S]*overflow-y:\s*auto;/);
  assert.match(css, /\.log-panel\s*\{[\s\S]*overflow-x:\s*hidden;/);
  assert.match(css, /\.log-panel\s*\{[\s\S]*min-height:\s*120px;/);
});

test("shell locks viewport with fixed height and prevents page-level scroll", () => {
  const css = fs.readFileSync(path.join(__dirname, "../styles.css"), "utf8");
  const shellBlock = css.match(/\.shell\s*\{[^}]*\}/)?.[0] ?? "";

  assert.match(shellBlock, /height:\s*100vh;/);
  assert.match(shellBlock, /overflow:\s*hidden;/);
  assert.match(shellBlock, /align-items:\s*stretch;/);
});

test("tauri config centers the main window and uses lighter height constraints", () => {
  const config = fs.readFileSync(path.join(__dirname, "../../src-tauri/tauri.conf.json"), "utf8");

  assert.match(config, /"center":\s*true/);
  assert.match(config, /"width":\s*1280/);
  assert.match(config, /"height":\s*760/);
  assert.match(config, /"minWidth":\s*940/);
  assert.match(config, /"minHeight":\s*620/);
});

test("filterTunnels returns all tunnels when no filters are active", () => {
  const tunnels = [
    { definition: { id: "a", name: "prod-db" }, status: "connected" },
    { definition: { id: "b", name: "dev-redis" }, status: "idle" },
  ];

  assert.deepEqual(filterTunnels(tunnels, "", "all"), tunnels);
});

test("filterTunnels filters by status", () => {
  const tunnels = [
    { definition: { id: "a", name: "prod-db" }, status: "connected" },
    { definition: { id: "b", name: "dev-redis" }, status: "idle" },
    { definition: { id: "c", name: "staging" }, status: "error" },
  ];

  assert.deepEqual(filterTunnels(tunnels, "", "connected"), [tunnels[0]]);
  assert.deepEqual(filterTunnels(tunnels, "", "idle"), [tunnels[1]]);
  assert.deepEqual(filterTunnels(tunnels, "", "error"), [tunnels[2]]);
});

test("filterTunnels filters by text matching name, host, username, and ports", () => {
  const tunnels = [
    { definition: { id: "a", name: "prod-db", ssh_host: "bastion.example.com", username: "admin", ssh_port: 22, local_bind_port: 15432, remote_host: "10.0.0.12", remote_port: 5432 }, status: "idle" },
    { definition: { id: "b", name: "dev-redis", ssh_host: "dev.local", username: "deploy", ssh_port: 22, local_bind_port: 6379, remote_host: "redis.internal", remote_port: 6379 }, status: "idle" },
  ];

  assert.deepEqual(filterTunnels(tunnels, "prod", "all"), [tunnels[0]]);
  assert.deepEqual(filterTunnels(tunnels, "redis", "all"), [tunnels[1]]);
  assert.deepEqual(filterTunnels(tunnels, "5432", "all"), [tunnels[0]]);
  assert.deepEqual(filterTunnels(tunnels, "admin", "all"), [tunnels[0]]);
  assert.deepEqual(filterTunnels(tunnels, "deploy", "all"), [tunnels[1]]);
});

test("filterTunnels combines text and status filters", () => {
  const tunnels = [
    { definition: { id: "a", name: "prod-db", ssh_host: "bastion", username: "admin", ssh_port: 22, local_bind_port: 15432, remote_host: "10.0.0.12", remote_port: 5432 }, status: "connected" },
    { definition: { id: "b", name: "prod-cache", ssh_host: "cache", username: "admin", ssh_port: 22, local_bind_port: 6379, remote_host: "redis", remote_port: 6379 }, status: "idle" },
  ];

  assert.deepEqual(filterTunnels(tunnels, "prod", "connected"), [tunnels[0]]);
  assert.deepEqual(filterTunnels(tunnels, "prod", "idle"), [tunnels[1]]);
  assert.deepEqual(filterTunnels(tunnels, "prod", "error"), []);
});

test("filterTunnels returns empty array for no matches", () => {
  const tunnels = [
    { definition: { id: "a", name: "prod-db", ssh_host: "bastion", username: "admin", ssh_port: 22, local_bind_port: 15432, remote_host: "10.0.0.12", remote_port: 5432 }, status: "idle" },
  ];

  assert.deepEqual(filterTunnels(tunnels, "nonexistent", "all"), []);
});

test("filterTunnels handles null input", () => {
  assert.deepEqual(filterTunnels(null, "", "all"), []);
});
