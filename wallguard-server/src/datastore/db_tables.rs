use std::fmt::{Display, Formatter, Result};

pub enum DBTable {
    Devices,
    RemoteAccessSessions,
}

impl Display for DBTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let table_name = match self {
            DBTable::Devices => "devices",
            DBTable::RemoteAccessSessions => "device_remote_access_sessions",
        };
        write!(f, "{}", table_name)
    }
}

impl Into<String> for DBTable {
    fn into(self) -> String {
        self.to_string()
    }
}
