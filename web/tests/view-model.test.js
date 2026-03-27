const test = require("node:test");
const assert = require("node:assert/strict");

const {
  describeAutostart,
  describeTunnelActions,
  describeTunnelListItem,
  describeTunnelStatus,
  summarizeSnapshotMeta,
} = require("../view-model.js");

function sampleTunnel(status = "idle") {
  return {
    status,
    definition: {
      id: "db",
      name: "Database",
      username: "deploy",
      ssh_host: "bastion.example.com",
      local_bind_port: 15432,
      remote_host: "10.0.0.12",
      remote_port: 5432,
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
