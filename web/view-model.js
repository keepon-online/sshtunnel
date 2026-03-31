(function (root, factory) {
  const api = factory();

  if (typeof module !== "undefined" && module.exports) {
    module.exports = api;
  }

  root.SshTunnelViewModel = api;
})(typeof globalThis !== "undefined" ? globalThis : this, () => {
  function formatIp(ip) {
    if (!ip) return "";
    const ipv4Match = ip.match(/^(\d{1,3})\.\d{1,3}\.\d{1,3}\.(\d{1,3})$/);
    if (ipv4Match) {
      return `${ipv4Match[1]}.***.***.${ipv4Match[2]}`;
    }
    if (ip.includes('.')) {
      const parts = ip.split('.');
      if (parts.length >= 2) {
        return `${parts[0]}.***.${parts[parts.length - 1]}`;
      }
    }
    return "****";
  }

  function describeAutostart(enabled) {
    return enabled
      ? {
          statusText: "已开启",
          actionText: "关闭开机自启",
        }
      : {
          statusText: "已关闭",
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
      id: definition.id,
      title: definition.name ?? "",
      subtitleRaw: `${definition.username ?? ""}@${definition.ssh_host ?? ""}`,
      subtitle: `${definition.username ?? ""}@${formatIp(definition.ssh_host)}`,
      forwardRaw: `${definition.local_bind_port ?? ""} -> ${definition.remote_host ?? ""}:${definition.remote_port ?? ""}`,
      forwardText: `${definition.local_bind_port ?? ""} -> ${formatIp(definition.remote_host)}:${definition.remote_port ?? ""}`,
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
      subtitleRaw: `${tunnel.definition?.username ?? ""}@${tunnel.definition?.ssh_host ?? ""}`,
      subtitle: `${tunnel.definition?.username ?? ""}@${formatIp(tunnel.definition?.ssh_host)}`,
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
        primaryLabel: "连接状态",
        primaryTone: "idle",
        primaryText: "未选择",
        primarySubtitle: "从左侧选择现有隧道，或创建一个新的本地转发配置。",
        errorText: "",
        forwardLabel: "本地转发",
        forwardText: "选择隧道后显示转发目标",
        authLabel: "认证方式",
        authText: "选择隧道后显示认证方式",
        authMeta: "",
      };
    }

    const statusCopy = describeTunnelStatus(tunnel.status);
    const definition = tunnel.definition ?? {};
    return {
      primaryLabel: "连接状态",
      primaryTone: statusCopy.tone,
      primaryText: statusCopy.text,
      primarySubtitleRaw: `${definition.username ?? ""}@${definition.ssh_host ?? ""}`,
      primarySubtitle: `${definition.username ?? ""}@${formatIp(definition.ssh_host)}`,
      errorText: tunnel.last_error ?? "",
      forwardLabel: "本地转发",
      forwardRaw: `${definition.local_bind_address ?? ""}:${definition.local_bind_port ?? ""} -> ${definition.remote_host ?? ""}:${definition.remote_port ?? ""}`,
      forwardText: `${definition.local_bind_address ?? ""}:${definition.local_bind_port ?? ""} -> ${formatIp(definition.remote_host)}:${definition.remote_port ?? ""}`,
      authLabel: "认证方式",
      authText: definition.auth_kind === "password" ? "密码认证" : "密钥认证",
      authMeta: `自动重连: ${definition.auto_reconnect ? "开启" : "关闭"}`,
    };
  }

  function describeDiagnosticLogPanel(lines) {
    const logLines = Array.isArray(lines) ? lines : [];
    const statusPatterns = [
      "[测试状态]",
      "spawned ssh process",
      "stopped ssh process",
      "ssh process exited",
      "failed to query process status",
      "missing password credential",
      "password sent",
      "ssh exited with status",
    ];
    const errorPatterns = [
      "失败",
      "不可达",
      "超时",
      "缺少",
      "拒绝",
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
      const lower = line.toLowerCase();
      if (line.startsWith("[测试状态]") || statusPatterns.some((pattern) => lower.includes(pattern))) {
        statusEvents.push(toEntry(line));
      } else if (line.startsWith("[测试输出]")) {
        sshOutput.push(toEntry(line));
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

  function describeCommandCenterHero(tunnel) {
    return describeWorkspacePanel(tunnel);
  }

  function describeCommandCenterCards(tunnel) {
    if (!tunnel) {
      return {
        forwardLabel: "本地转发",
        forwardValue: "选择隧道后显示转发目标",
        authLabel: "认证方式",
        authValue: "选择隧道后显示认证方式",
        healthLabel: "连接健康",
        healthValue: "无数据",
      };
    }

    const definition = tunnel.definition ?? {};
    const forwardValue = `${definition.local_bind_address ?? ""}:${definition.local_bind_port ?? ""} -> ${definition.remote_host ?? ""}:${definition.remote_port ?? ""}`;

    const statusCopy = describeTunnelStatus(tunnel.status);
    const healthValue =
      tunnel.status === "error"
        ? `错误 — ${tunnel.last_error ?? statusCopy.text}`
        : `自动重连: ${definition.auto_reconnect ? "开启" : "关闭"}`;

    return {
      forwardLabel: "本地转发",
      forwardValue,
      authLabel: "认证方式",
      authValue: definition.auth_kind === "password" ? "密码认证" : "密钥认证",
      healthLabel: "连接健康",
      healthValue,
    };
  }

  function describeCommandCenterTimeline(lines) {
    return describeDiagnosticLogPanel(lines);
  }

  function summarizeSnapshotMeta(snapshot) {
    const autostart = describeAutostart(Boolean(snapshot?.autostart_enabled));

    return {
      sshText: snapshot?.ssh_available ? "可用" : "缺失",
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
    describeCommandCenterHero,
    describeCommandCenterCards,
    describeCommandCenterTimeline,
  };
});
