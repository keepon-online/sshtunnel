use sshtunnel_core::{
    models::{AuthKind, TunnelDefinition},
    ssh_launch::{build_launch_plan, LaunchPlan},
};

fn sample_tunnel() -> TunnelDefinition {
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
fn private_key_auth_uses_native_process_launch() {
    let tunnel = sample_tunnel();

    let plan = build_launch_plan(&tunnel, None).expect("launch plan");

    match plan {
        LaunchPlan::Native(command) => {
            assert_eq!(command.program, "ssh");
            assert!(command.args.contains(&"-i".to_string()));
        }
        _ => panic!("expected native launch plan"),
    }
}

#[test]
fn password_auth_requires_runtime_password_value() {
    let mut tunnel = sample_tunnel();
    tunnel.auth_kind = AuthKind::Password;
    tunnel.private_key_path = None;
    tunnel.password_entry = Some("profile:db".into());

    let error = build_launch_plan(&tunnel, None).expect_err("missing password should fail");

    assert!(error.contains("password"));
}

#[test]
fn password_auth_uses_prompted_password_launch() {
    let mut tunnel = sample_tunnel();
    tunnel.auth_kind = AuthKind::Password;
    tunnel.private_key_path = None;
    tunnel.password_entry = Some("profile:db".into());

    let plan = build_launch_plan(&tunnel, Some("s3cr3t")).expect("launch plan");

    match plan {
        LaunchPlan::PromptedPassword {
            command,
            password,
            prompt,
        } => {
            assert_eq!(command.program, "ssh");
            assert_eq!(password, "s3cr3t");
            assert_eq!(prompt, "assword:");
            assert!(command
                .args
                .contains(&"PreferredAuthentications=password".to_string()));
        }
        _ => panic!("expected prompted password launch plan"),
    }
}
