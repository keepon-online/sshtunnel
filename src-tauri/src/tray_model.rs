use sshtunnel_core::models::TunnelDefinition;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayTunnelAction {
    Connect,
    Disconnect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayTunnelItem {
    pub tunnel_id: String,
    pub title: String,
    pub detail: String,
    pub action: TrayTunnelAction,
}

pub fn order_recent_tunnels<'a>(
    tunnels: &'a [TunnelDefinition],
    recent_ids: &[String],
) -> Vec<&'a TunnelDefinition> {
    let mut ordered = Vec::new();

    for recent_id in recent_ids {
        if let Some(tunnel) = tunnels.iter().find(|item| &item.id == recent_id) {
            ordered.push(tunnel);
        }
    }

    for tunnel in tunnels {
        if !ordered.iter().any(|item| item.id == tunnel.id) {
            ordered.push(tunnel);
        }
    }

    ordered
}

pub fn recent_tray_items<'a, I>(
    tunnels: I,
    status_for_id: impl Fn(&str) -> bool,
) -> Vec<TrayTunnelItem>
where
    I: IntoIterator<Item = &'a TunnelDefinition>,
{
    tunnels
        .into_iter()
        .take(3)
        .map(|tunnel| TrayTunnelItem {
            tunnel_id: tunnel.id.clone(),
            title: tunnel.name.clone(),
            detail: format!("{}@{}", tunnel.username, tunnel.ssh_host),
            action: if status_for_id(&tunnel.id) {
                TrayTunnelAction::Disconnect
            } else {
                TrayTunnelAction::Connect
            },
        })
        .collect()
}

pub fn tray_action_id(action: TrayTunnelAction, tunnel_id: &str) -> String {
    let prefix = match action {
        TrayTunnelAction::Connect => "connect",
        TrayTunnelAction::Disconnect => "disconnect",
    };

    format!("{prefix}:{tunnel_id}")
}

pub fn tray_action_label(item: &TrayTunnelItem) -> String {
    let action_label = match item.action {
        TrayTunnelAction::Connect => "连接",
        TrayTunnelAction::Disconnect => "断开",
    };

    format!("{action_label}：{} ({})", item.title, item.detail)
}
