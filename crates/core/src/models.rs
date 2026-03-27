use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthKind {
    PrivateKey,
    Password,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TunnelDefinition {
    pub id: String,
    pub name: String,
    pub ssh_host: String,
    pub ssh_port: u16,
    pub username: String,
    pub local_bind_address: String,
    pub local_bind_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
    pub auth_kind: AuthKind,
    pub private_key_path: Option<String>,
    pub auto_connect: bool,
    pub auto_reconnect: bool,
    pub password_entry: Option<String>,
}

impl TunnelDefinition {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("id is required".into());
        }

        if self.name.trim().is_empty() {
            return Err("name is required".into());
        }

        if self.ssh_host.trim().is_empty() {
            return Err("ssh_host is required".into());
        }

        if self.username.trim().is_empty() {
            return Err("username is required".into());
        }

        if self.local_bind_address.trim().is_empty() {
            return Err("local_bind_address is required".into());
        }

        if self.remote_host.trim().is_empty() {
            return Err("remote_host is required".into());
        }

        if self.ssh_port == 0 {
            return Err("ssh_port must be greater than 0".into());
        }

        if self.local_bind_port == 0 {
            return Err("local_bind_port must be greater than 0".into());
        }

        if self.remote_port == 0 {
            return Err("remote_port must be greater than 0".into());
        }

        match self.auth_kind {
            AuthKind::PrivateKey => {
                let key_path = self.private_key_path.as_deref().unwrap_or_default().trim();
                if key_path.is_empty() {
                    return Err("private_key_path is required for private_key auth".into());
                }
            }
            AuthKind::Password => {
                let credential = self.password_entry.as_deref().unwrap_or_default().trim();
                if credential.is_empty() {
                    return Err("password_entry is required for password auth".into());
                }
            }
        }

        Ok(())
    }
}
