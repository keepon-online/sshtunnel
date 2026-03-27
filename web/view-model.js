(function (root, factory) {
  const api = factory();

  if (typeof module !== "undefined" && module.exports) {
    module.exports = api;
  }

  root.SshTunnelViewModel = api;
})(typeof globalThis !== "undefined" ? globalThis : this, () => {
  function describeAutostart(enabled) {
    return enabled
      ? {
          statusText: "enabled",
          actionText: "关闭开机自启",
        }
      : {
          statusText: "disabled",
          actionText: "开启开机自启",
        };
  }

  function describeTunnelStatus(status) {
    switch (status) {
      case "idle":
        return { tone: "idle", text: "空闲" };
      case "connected":
        return { tone: "connected", text: "已连接" };
      case "error":
        return { tone: "error", text: "错误" };
      default:
        return {
          tone: status || "idle",
          text: status || "unknown",
        };
    }
  }

  function describeTunnelActions(tunnel) {
    if (!tunnel) {
      return {
        connectText: "连接",
        connectDisabled: true,
        disconnectText: "断开",
        disconnectDisabled: true,
      };
    }

    switch (tunnel.status) {
      case "connected":
        return {
          connectText: "已连接",
          connectDisabled: true,
          disconnectText: "断开",
          disconnectDisabled: false,
        };
      case "error":
        return {
          connectText: "重新连接",
          connectDisabled: false,
          disconnectText: "断开",
          disconnectDisabled: true,
        };
      case "idle":
      default:
        return {
          connectText: "连接",
          connectDisabled: false,
          disconnectText: "断开",
          disconnectDisabled: true,
        };
    }
  }

  function describeTunnelListItem(tunnel, selectedId) {
    const statusCopy = describeTunnelStatus(tunnel?.status);
    const definition = tunnel?.definition ?? {};

    return {
      title: definition.name ?? "",
      subtitle: `${definition.username ?? ""}@${definition.ssh_host ?? ""}`,
      forwardText: `${definition.local_bind_port ?? ""} -> ${definition.remote_host ?? ""}:${definition.remote_port ?? ""}`,
      badgeTone: statusCopy.tone,
      badgeText: statusCopy.text,
      isActive: definition.id === selectedId,
    };
  }

  function summarizeSnapshotMeta(snapshot) {
    const autostart = describeAutostart(Boolean(snapshot?.autostart_enabled));

    return {
      sshText: snapshot?.ssh_available ? "available" : "missing",
      autostartText: autostart.statusText,
      autostartAction: autostart.actionText,
      configPath: snapshot?.config_path ?? "-",
    };
  }

  return {
    describeAutostart,
    describeTunnelActions,
    describeTunnelListItem,
    describeTunnelStatus,
    summarizeSnapshotMeta,
  };
});
