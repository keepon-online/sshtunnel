use sshtunnel_core::models::{AuthKind, TunnelDefinition};
use sshtunnel_core::ssh_args::{build_ssh_args, build_ssh_probe_args};

fn valid_key_tunnel() -> TunnelDefinition {
    TunnelDefinition {
        id: "db".into(),
        name: "Database".into(),
        ssh_host: "bastion.example.com".into(),
        ssh_port: 22,
        username: "deploy".into(),
        local_bind_address: "127.0.0.1".into(),
        local_bind_port: 15432,
        remote_host: "10.0.0.12".into(),
        remote_port: 5432,
        auth_kind: AuthKind::PrivateKey,
        private_key_path: Some("/home/top/.ssh/id_ed25519".into()),
        auto_connect: false,
        auto_reconnect: true,
        password_entry: None,
    }
}

#[test]
fn rejects_password_tunnel_without_credential_reference() {
    let mut tunnel = valid_key_tunnel();
    tunnel.auth_kind = AuthKind::Password;
    tunnel.private_key_path = None;

    let result = tunnel.validate();

    assert!(result.is_err());
    assert!(
        result.err().unwrap().contains("password_entry"),
        "unexpected error message"
    );
}

#[test]
fn rejects_empty_hosts_and_ports() {
    let mut tunnel = valid_key_tunnel();
    tunnel.ssh_host.clear();
    tunnel.remote_host.clear();
    tunnel.local_bind_port = 0;

    let result = tunnel.validate();

    assert!(result.is_err());
}

#[test]
fn builds_private_key_ssh_command() {
    let tunnel = valid_key_tunnel();

    let args = build_ssh_args(&tunnel);

    assert_eq!(args[0], "-N");
    assert!(args.contains(&"-T".to_string()));
    assert!(args.contains(&"-p".to_string()));
    assert!(args.contains(&"22".to_string()));
    assert!(args.contains(&"-i".to_string()));
    assert!(args.contains(&"/home/top/.ssh/id_ed25519".to_string()));
    assert!(args.contains(&"-L".to_string()));
    assert!(
        args.contains(&"127.0.0.1:15432:10.0.0.12:5432".to_string()),
        "missing local forwarding spec"
    );
    assert_eq!(args.last().unwrap(), "deploy@bastion.example.com");
}

#[test]
fn builds_password_command_without_private_key_flag() {
    let mut tunnel = valid_key_tunnel();
    tunnel.auth_kind = AuthKind::Password;
    tunnel.private_key_path = None;
    tunnel.password_entry = Some("sshtunnel/db".into());

    let args = build_ssh_args(&tunnel);

    assert!(args.contains(&"-o".to_string()));
    assert!(args.contains(&"PreferredAuthentications=password".to_string()));
    assert!(
        !args.contains(&"-i".to_string()),
        "password auth should not inject key args"
    );
}

#[test]
fn builds_probe_command_without_forwarding_flags() {
    let tunnel = valid_key_tunnel();

    let args = build_ssh_probe_args(&tunnel, "printf '__probe__\\n' >&2");

    assert_eq!(args[0], "-T");
    assert!(!args.contains(&"-N".to_string()));
    assert!(!args.contains(&"-L".to_string()));
    assert_eq!(args[args.len() - 2], "deploy@bastion.example.com");
    assert_eq!(args.last().unwrap(), "printf '__probe__\\n' >&2");
}
