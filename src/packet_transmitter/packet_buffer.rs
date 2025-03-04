use nullnet_libwallguard::Packet;
use std::ops::RangeTo;

pub(crate) struct PacketBuffer {
    buffer: Vec<Packet>,
    size: usize,
}

impl PacketBuffer {
    pub(crate) fn new(size: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(size),
            size,
        }
    }

    pub(crate) fn push(&mut self, packet: Packet) {
        self.buffer.push(packet);
    }

    pub(crate) fn take(&mut self) -> Vec<Packet> {
        std::mem::take(&mut self.buffer)
    }

    pub(crate) fn get(&mut self, range: RangeTo<usize>) -> Vec<Packet> {
        self.buffer.get(range).unwrap_or_default().to_vec()
    }

    pub(crate) fn extend(&mut self, packets: Vec<Packet>) {
        self.buffer.extend(packets);
    }

    pub(crate) fn drain(&mut self, range: RangeTo<usize>) {
        self.buffer.drain(range);
    }

    pub(crate) fn is_full(&self) -> bool {
        self.buffer.len() >= self.size
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub(crate) fn len(&self) -> usize {
        self.buffer.len()
    }
}
