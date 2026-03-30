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

  function describeWorkspacePanel(tunnel) {
    if (!tunnel) {
      return {
        title: "选择一个隧道",
        subtitle: "从左侧选择现有隧道，或创建一个新的本地转发配置。",
        statusText: "未选择",
        statusTone: "idle",
      };
    }

    const statusCopy = describeTunnelStatus(tunnel.status);
    return {
      title: tunnel.definition?.name ?? "",
      subtitle: `${tunnel.definition?.username ?? ""}@${tunnel.definition?.ssh_host ?? ""}`,
      statusText: statusCopy.text,
      statusTone: statusCopy.tone,
    };
  }

  function describeEditorSheet(tunnel) {
    if (!tunnel) {
      return {
        title: "新建隧道",
        submitText: "保存配置",
      };
    }

    return {
      title: "编辑隧道",
      submitText: "保存修改",
    };
  }

  function describeStatusSummaryCards(tunnel) {
    if (!tunnel) {
      return {
        primaryLabel: "Connection",
        primaryTone: "idle",
        primaryText: "未选择",
        primarySubtitle: "从左侧选择现有隧道，或创建一个新的本地转发配置。",
        errorText: "",
        forwardLabel: "本地转发",
        forwardText: "选择隧道后显示转发目标",
        authLabel: "Authentication",
        authText: "选择隧道后显示认证方式",
        authMeta: "",
      };
    }

    const statusCopy = describeTunnelStatus(tunnel.status);
    const definition = tunnel.definition ?? {};
    return {
      primaryLabel: "Connection",
      primaryTone: statusCopy.tone,
      primaryText: statusCopy.text,
      primarySubtitle: `${definition.username ?? ""}@${definition.ssh_host ?? ""}`,
      errorText: tunnel.last_error ?? "",
      forwardLabel: "本地转发",
      forwardText: `${definition.local_bind_address ?? ""}:${definition.local_bind_port ?? ""} -> ${definition.remote_host ?? ""}:${definition.remote_port ?? ""}`,
      authLabel: "Authentication",
      authText: definition.auth_kind === "password" ? "密码认证" : "密钥认证",
      authMeta: `自动重连: ${definition.auto_reconnect ? "开启" : "关闭"}`,
    };
  }

  function describeDiagnosticLogPanel(lines) {
    const logLines = Array.isArray(lines) ? lines : [];
    const statusPatterns = [
      "spawned ssh process",
      "stopped ssh process",
      "ssh process exited",
      "failed to query process status",
      "missing password credential",
      "password sent",
      "ssh exited with status",
    ];
    const errorPatterns = [
      "error",
      "failed",
      "missing",
      "denied",
      "refused",
      "exit status",
      "permission",
    ];

    const toEntry = (text) => ({
      text,
      tone: errorPatterns.some((pattern) => text.toLowerCase().includes(pattern))
        ? "error"
        : "default",
    });

    const statusEvents = [];
    const sshOutput = [];

    for (const line of logLines) {
      if (statusPatterns.some((pattern) => line.toLowerCase().includes(pattern))) {
        statusEvents.push(toEntry(line));
      } else {
        sshOutput.push(toEntry(line));
      }
    }

    const eventCount = statusEvents.length;
    const sshCount = sshOutput.length;

    return {
      summaryText:
        eventCount === 0 && sshCount === 0
          ? "暂无最近日志"
          : `最近 ${eventCount} 条状态事件，${sshCount} 条 SSH 输出`,
      statusEvents,
      sshOutput,
      emptyStatusText: "暂无状态事件",
      emptySshText: "暂无 SSH 输出",
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
    describeEditorSheet,
    describeDiagnosticLogPanel,
    describeStatusSummaryCards,
    describeWorkspacePanel,
    summarizeSnapshotMeta,
  };
});
