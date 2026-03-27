use crate::{
    models::{AuthKind, TunnelDefinition},
    ssh_args::build_ssh_args,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchPlan {
    Native(CommandSpec),
    PromptedPassword {
        command: CommandSpec,
        password: String,
        prompt: String,
    },
}

pub fn build_launch_plan(
    tunnel: &TunnelDefinition,
    password: Option<&str>,
) -> Result<LaunchPlan, String> {
    tunnel.validate()?;

    let command = CommandSpec {
        program: "ssh".to_string(),
        args: build_ssh_args(tunnel),
    };

    match tunnel.auth_kind {
        AuthKind::PrivateKey => Ok(LaunchPlan::Native(command)),
        AuthKind::Password => {
            let password = password
                .filter(|value| !value.is_empty())
                .ok_or_else(|| "password auth requires a password value".to_string())?;

            Ok(LaunchPlan::PromptedPassword {
                command,
                password: password.to_string(),
                prompt: "assword:".to_string(),
            })
        }
    }
}
