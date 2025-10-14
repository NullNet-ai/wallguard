use nullnet_liberror::{Error, ErrorHandler, Location, location};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize)]
pub struct TimestampedPacket {
    pub duration: Duration,
    pub data: Vec<u8>,
}

impl TimestampedPacket {
    pub fn new(duration: Duration, data: Vec<u8>) -> Self {
        Self { duration, data }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.duration.as_millis().to_le_bytes());
        bytes.extend_from_slice(&(self.data.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.data);

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < 20 {
            return Err("Not enough bytes").handle_err(location!());
        }

        let duration_millis = u128::from_le_bytes(bytes[0..16].try_into().handle_err(location!())?);

        let data_len =
            u32::from_le_bytes(bytes[16..20].try_into().handle_err(location!())?) as usize;

        if bytes.len() < 20 + data_len {
            return Err("Incomplete data").handle_err(location!());
        }

        let data = bytes[20..20 + data_len].to_vec();

        Ok(TimestampedPacket {
            duration: Duration::from_millis(duration_millis as u64),
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialization() {
        let packet = TimestampedPacket::new(Duration::from_millis(42), vec![1, 2, 3, 4, 5]);

        let bytes = packet.to_bytes();
        let decoded = TimestampedPacket::from_bytes(&bytes).unwrap();

        assert_eq!(packet.duration, decoded.duration);
        assert_eq!(packet.data, decoded.data);
    }
}
