use crate::models::{AuthKind, TunnelDefinition};

pub fn build_ssh_args(tunnel: &TunnelDefinition) -> Vec<String> {
    let mut args = vec![
        "-N".to_string(),
        "-T".to_string(),
        "-p".to_string(),
        tunnel.ssh_port.to_string(),
        "-o".to_string(),
        "ExitOnForwardFailure=yes".to_string(),
        "-o".to_string(),
        "ServerAliveInterval=30".to_string(),
        "-o".to_string(),
        "ServerAliveCountMax=3".to_string(),
    ];

    match tunnel.auth_kind {
        AuthKind::PrivateKey => {
            if let Some(key_path) = &tunnel.private_key_path {
                args.push("-i".to_string());
                args.push(key_path.clone());
                args.push("-o".to_string());
                args.push("IdentitiesOnly=yes".to_string());
            }
        }
        AuthKind::Password => {
            args.push("-o".to_string());
            args.push("PreferredAuthentications=password".to_string());
            args.push("-o".to_string());
            args.push("PubkeyAuthentication=no".to_string());
        }
    }

    args.push("-L".to_string());
    args.push(format!(
        "{}:{}:{}:{}",
        tunnel.local_bind_address, tunnel.local_bind_port, tunnel.remote_host, tunnel.remote_port
    ));
    args.push(format!("{}@{}", tunnel.username, tunnel.ssh_host));

    args
}
