pub mod serde_ipaddr_option {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::net::IpAddr;

    pub fn serialize<S>(ip: &Option<IpAddr>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(ip) = ip {
            (1u8, ip.to_string()).serialize(serializer)
        } else {
            (0u8, "".to_string()).serialize(serializer)
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<IpAddr>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (flag, ip_str): (u8, String) = Deserialize::deserialize(deserializer)?;
        match flag {
            1 => ip_str.parse().map(Some).map_err(serde::de::Error::custom),
            _ => Ok(None),
        }
    }
}

pub mod serde_ipaddr_vec {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::net::IpAddr;

    #[allow(clippy::ptr_arg)]
    pub fn serialize<S>(ips: &Vec<IpAddr>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let ip_strings: Vec<String> = ips.iter().map(|ip| ip.to_string()).collect();
        serializer.serialize_str(&ip_strings.join(","))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<IpAddr>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ip_string = String::deserialize(deserializer)?;
        let ips: Vec<IpAddr> = ip_string
            .split(',')
            .filter_map(|s| s.parse().ok())
            .collect();
        Ok(ips)
    }
}
