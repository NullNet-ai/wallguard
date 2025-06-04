use std::fmt::{Display, Formatter, Result};

pub enum DBTable {
    Devices,
    SSHKeys,
    RemoteAccessSessions,
    OgranizationAccounts,
}

impl Display for DBTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let table_name = match self {
            DBTable::Devices => "devices",
            DBTable::SSHKeys => "device_ssh_keys",
            DBTable::RemoteAccessSessions => "device_remote_access_sessions",
            DBTable::OgranizationAccounts => "organization_accounts",
        };
        write!(f, "{}", table_name)
    }
}

impl Into<String> for DBTable {
    fn into(self) -> String {
        self.to_string()
    }
}
