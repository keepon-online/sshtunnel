#[cfg(test)]
mod tests {
    use sshtunnel_core::models::{AuthKind, TunnelDefinition};

    use crate::tray_model::{
        order_recent_tunnels, recent_tray_items, tray_action_id, tray_action_label,
        TrayTunnelAction,
    };

    fn tunnel(id: &str, name: &str) -> TunnelDefinition {
        TunnelDefinition {
            id: id.into(),
            name: name.into(),
            ssh_host: format!("{id}.example.com"),
            ssh_port: 22,
            username: "deploy".into(),
            local_bind_address: "127.0.0.1".into(),
            local_bind_port: 15000,
            remote_host: "10.0.0.10".into(),
            remote_port: 5432,
            auth_kind: AuthKind::PrivateKey,
            private_key_path: Some("~/.ssh/id_ed25519".into()),
            auto_connect: false,
            auto_reconnect: true,
            password_entry: None,
        }
    }

    #[test]
    fn keeps_only_first_three_recent_tray_items() {
        let tunnels = vec![
            tunnel("a", "Alpha"),
            tunnel("b", "Beta"),
            tunnel("c", "Gamma"),
            tunnel("d", "Delta"),
        ];

        let items = recent_tray_items(tunnels.iter(), |_| false);

        assert_eq!(items.len(), 3);
        assert_eq!(items[0].title, "Alpha");
        assert_eq!(items[1].title, "Beta");
        assert_eq!(items[2].title, "Gamma");
    }

    #[test]
    fn orders_recent_tunnels_by_last_touched_ids() {
        let tunnels = vec![
            tunnel("a", "Alpha"),
            tunnel("b", "Beta"),
            tunnel("c", "Gamma"),
            tunnel("d", "Delta"),
        ];
        let recent_ids = vec!["c".to_string(), "a".to_string()];

        let ordered = order_recent_tunnels(&tunnels, &recent_ids);

        assert_eq!(ordered[0].id, "c");
        assert_eq!(ordered[1].id, "a");
        assert_eq!(ordered[2].id, "b");
        assert_eq!(ordered[3].id, "d");
    }

    #[test]
    fn uses_disconnect_action_for_connected_tunnel() {
        let tunnels = vec![tunnel("db", "Database")];

        let items = recent_tray_items(tunnels.iter(), |id| id == "db");

        assert_eq!(items[0].action, TrayTunnelAction::Disconnect);
        assert_eq!(tray_action_id(items[0].action, &items[0].tunnel_id), "disconnect:db");
    }

    #[test]
    fn uses_connect_action_for_idle_tunnel() {
        let tunnels = vec![tunnel("cache", "Cache")];

        let items = recent_tray_items(tunnels.iter(), |_| false);

        assert_eq!(items[0].action, TrayTunnelAction::Connect);
        assert_eq!(
            tray_action_id(items[0].action, &items[0].tunnel_id),
            "connect:cache"
        );
    }

    #[test]
    fn renders_recent_tray_item_label_with_action_and_target() {
        let tunnels = vec![tunnel("db", "Database")];

        let connected_items = recent_tray_items(tunnels.iter(), |id| id == "db");
        let idle_items = recent_tray_items(tunnels.iter(), |_| false);

        assert_eq!(
            tray_action_label(&connected_items[0]),
            "Disconnect: Database (deploy@db.example.com)"
        );
        assert_eq!(
            tray_action_label(&idle_items[0]),
            "Connect: Database (deploy@db.example.com)"
        );
    }
}
