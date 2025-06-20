use std::fmt::{Display, Formatter, Result};

pub enum DBTable {
    Devices,
    SSHKeys,
    RemoteAccessSessions,
    Accounts,
    AccountOrganizations,
    IpInfos,
    Connections,
    SystemResources,
    DeviceConfigurations,
    DeviceRules,
    DeviceAliases,
    DeviceInterfaces,
    DeviceInterfaceAddresses,
}

impl Display for DBTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let table_name = match self {
            DBTable::Devices => "devices",
            DBTable::SSHKeys => "device_ssh_keys",
            DBTable::RemoteAccessSessions => "device_remote_access_sessions",
            DBTable::Accounts => "accounts",
            DBTable::AccountOrganizations => "account_organizations",
            DBTable::IpInfos => "ip_infos",
            DBTable::Connections => "connections",
            DBTable::SystemResources => "system_resources",
            DBTable::DeviceConfigurations => "device_configurations",
            DBTable::DeviceRules => "device_rules",
            DBTable::DeviceAliases => "device_aliases",
            DBTable::DeviceInterfaces => "device_interfaces",
            DBTable::DeviceInterfaceAddresses => "device_interface_addresses",
        };
        write!(f, "{}", table_name)
    }
}

impl From<DBTable> for String {
    fn from(value: DBTable) -> Self {
        value.to_string()
    }
}
