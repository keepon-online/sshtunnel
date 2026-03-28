const test = require("node:test");
const assert = require("node:assert/strict");

const {
  describeAutostart,
  describeTunnelActions,
  describeTunnelListItem,
  describeTunnelStatus,
  describeEditorSheet,
  describeDiagnosticLogPanel,
  describeStatusSummaryCards,
  describeWorkspacePanel,
  summarizeSnapshotMeta,
} = require("../view-model.js");

function sampleTunnel(status = "idle") {
  return {
    status,
    last_error: status === "error" ? "ssh exited with status code 7" : null,
    definition: {
      id: "db",
      name: "Database",
      username: "deploy",
      ssh_host: "bastion.example.com",
      local_bind_address: "127.0.0.1",
      local_bind_port: 15432,
      remote_host: "10.0.0.12",
      remote_port: 5432,
      auth_kind: "private_key",
      auto_reconnect: true,
    },
  };
}

test("describeAutostart returns enabled copy", () => {
  assert.deepEqual(describeAutostart(true), {
    statusText: "enabled",
    actionText: "关闭开机自启",
  });
});

test("describeAutostart returns disabled copy", () => {
  assert.deepEqual(describeAutostart(false), {
    statusText: "disabled",
    actionText: "开启开机自启",
  });
});

test("summarizeSnapshotMeta includes autostart and ssh labels", () => {
  assert.deepEqual(
    summarizeSnapshotMeta({
      ssh_available: true,
      autostart_enabled: false,
      config_path: "/tmp/sshtunnel/config.json",
    }),
    {
      sshText: "available",
      autostartText: "disabled",
      autostartAction: "开启开机自启",
      configPath: "/tmp/sshtunnel/config.json",
    },
  );
});

test("describeTunnelStatus returns localized idle copy", () => {
  assert.deepEqual(describeTunnelStatus("idle"), {
    tone: "idle",
    text: "空闲",
  });
});

test("describeTunnelStatus returns localized connected copy", () => {
  assert.deepEqual(describeTunnelStatus("connected"), {
    tone: "connected",
    text: "已连接",
  });
});

test("describeTunnelActions disables both actions when no tunnel is selected", () => {
  assert.deepEqual(describeTunnelActions(null), {
    connectText: "连接",
    connectDisabled: true,
    disconnectText: "断开",
    disconnectDisabled: true,
  });
});

test("describeTunnelActions enables connect for idle state", () => {
  assert.deepEqual(describeTunnelActions({ status: "idle" }), {
    connectText: "连接",
    connectDisabled: false,
    disconnectText: "断开",
    disconnectDisabled: true,
  });
});

test("describeTunnelActions enables disconnect for connected state", () => {
  assert.deepEqual(describeTunnelActions({ status: "connected" }), {
    connectText: "已连接",
    connectDisabled: true,
    disconnectText: "断开",
    disconnectDisabled: false,
  });
});

test("describeTunnelActions enables reconnect for error state", () => {
  assert.deepEqual(describeTunnelActions({ status: "error" }), {
    connectText: "重新连接",
    connectDisabled: false,
    disconnectText: "断开",
    disconnectDisabled: true,
  });
});

test("describeTunnelListItem returns localized copy and active state for selected tunnel", () => {
  assert.deepEqual(describeTunnelListItem(sampleTunnel("connected"), "db"), {
    title: "Database",
    subtitle: "deploy@bastion.example.com",
    forwardText: "15432 -> 10.0.0.12:5432",
    badgeTone: "connected",
    badgeText: "已连接",
    isActive: true,
  });
});

test("describeTunnelListItem marks non-selected tunnel as inactive", () => {
  assert.equal(describeTunnelListItem(sampleTunnel("idle"), "cache").isActive, false);
});

test("describeWorkspacePanel returns empty-state copy when no tunnel is selected", () => {
  assert.deepEqual(describeWorkspacePanel(null), {
    title: "选择一个隧道",
    subtitle: "从左侧选择现有隧道，或创建一个新的本地转发配置。",
    statusText: "未选择",
    statusTone: "idle",
  });
});

test("describeWorkspacePanel returns title and subtitle for selected tunnel", () => {
  assert.deepEqual(describeWorkspacePanel(sampleTunnel("error")), {
    title: "Database",
    subtitle: "deploy@bastion.example.com",
    statusText: "错误",
    statusTone: "error",
  });
});

test("describeEditorSheet returns create-mode copy", () => {
  assert.deepEqual(describeEditorSheet(null), {
    title: "新建隧道",
    submitText: "保存配置",
  });
});

test("describeEditorSheet returns edit-mode copy", () => {
  assert.deepEqual(describeEditorSheet(sampleTunnel()), {
    title: "编辑隧道",
    submitText: "保存修改",
  });
});

test("describeStatusSummaryCards returns empty-state summary", () => {
  assert.deepEqual(describeStatusSummaryCards(null), {
    primaryLabel: "Connection",
    primaryTone: "idle",
    primaryText: "未选择",
    primarySubtitle: "从左侧选择现有隧道，或创建一个新的本地转发配置。",
    errorText: "",
    forwardLabel: "Forwarding",
    forwardText: "选择隧道后显示转发目标",
    authLabel: "Authentication",
    authText: "选择隧道后显示认证方式",
    authMeta: "",
  });
});

test("describeStatusSummaryCards returns connected summary", () => {
  assert.deepEqual(describeStatusSummaryCards(sampleTunnel("connected")), {
    primaryLabel: "Connection",
    primaryTone: "connected",
    primaryText: "已连接",
    primarySubtitle: "deploy@bastion.example.com",
    errorText: "",
    forwardLabel: "Forwarding",
    forwardText: "127.0.0.1:15432 -> 10.0.0.12:5432",
    authLabel: "Authentication",
    authText: "密钥认证",
    authMeta: "自动重连: 开启",
  });
});

test("describeStatusSummaryCards surfaces error summary and auth details", () => {
  const tunnel = sampleTunnel("error");
  tunnel.definition.auth_kind = "password";
  tunnel.definition.auto_reconnect = false;

  assert.deepEqual(describeStatusSummaryCards(tunnel), {
    primaryLabel: "Connection",
    primaryTone: "error",
    primaryText: "错误",
    primarySubtitle: "deploy@bastion.example.com",
    errorText: "ssh exited with status code 7",
    forwardLabel: "Forwarding",
    forwardText: "127.0.0.1:15432 -> 10.0.0.12:5432",
    authLabel: "Authentication",
    authText: "密码认证",
    authMeta: "自动重连: 关闭",
  });
});

test("describeDiagnosticLogPanel groups status events and ssh output", () => {
  assert.deepEqual(
    describeDiagnosticLogPanel([
      "spawned ssh process pid=123",
      "channel 0: open failed: connect failed: Connection refused",
      "password sent to interactive ssh session",
    ]),
    {
      summaryText: "最近 2 条状态事件，1 条 SSH 输出",
      statusEvents: [
        { text: "spawned ssh process pid=123", tone: "default" },
        { text: "password sent to interactive ssh session", tone: "default" },
      ],
      sshOutput: [
        {
          text: "channel 0: open failed: connect failed: Connection refused",
          tone: "error",
        },
      ],
      emptyStatusText: "暂无状态事件",
      emptySshText: "暂无 SSH 输出",
    },
  );
});

test("describeDiagnosticLogPanel returns empty-state summary when no logs exist", () => {
  assert.deepEqual(describeDiagnosticLogPanel([]), {
    summaryText: "暂无最近日志",
    statusEvents: [],
    sshOutput: [],
    emptyStatusText: "暂无状态事件",
    emptySshText: "暂无 SSH 输出",
  });
});
