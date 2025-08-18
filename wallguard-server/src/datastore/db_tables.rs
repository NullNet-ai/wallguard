use std::fmt::{Display, Formatter, Result};

pub enum DBTable {
    Devices,
    SSHKeys,
    RemoteAccessSessions,
    Accounts,
    IpInfos,
    Connections,
    SystemResources,
    DeviceConfigurations,
    DeviceFilterRules,
    DeviceNatRules,
    DeviceAliases,
    IpAliases,
    PortAliases,
    DeviceInterfaces,
    DeviceInterfaceAddresses,
    DeviceCredentials,
    InstallationCodes,
    DeviceInstances,
}

impl Display for DBTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let table_name = match self {
            DBTable::Devices => "devices",
            DBTable::SSHKeys => "device_ssh_keys",
            DBTable::RemoteAccessSessions => "device_remote_access_sessions",
            DBTable::Accounts => "accounts",
            DBTable::IpInfos => "ip_infos",
            DBTable::Connections => "connections",
            DBTable::SystemResources => "system_resources",
            DBTable::DeviceConfigurations => "device_configurations",
            DBTable::DeviceFilterRules => "device_filter_rules",
            DBTable::DeviceNatRules => "device_nat_rules",
            DBTable::DeviceAliases => "aliases",
            DBTable::IpAliases => "ip_aliases",
            DBTable::PortAliases => "port_aliases",
            DBTable::DeviceInterfaces => "device_interfaces",
            DBTable::DeviceInterfaceAddresses => "device_interface_addresses",
            DBTable::DeviceCredentials => "device_credentials",
            DBTable::InstallationCodes => "installation_codes",
            DBTable::DeviceInstances => "device_instances",
        };
        write!(f, "{table_name}")
    }
}

impl From<DBTable> for String {
    fn from(value: DBTable) -> Self {
        value.to_string()
    }
}
