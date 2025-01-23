use wallguard_server::Packet;

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

    pub(crate) fn get_clone(&mut self) -> Vec<Packet> {
        self.buffer.clone()
    }

    pub(crate) fn extend(&mut self, packets: Vec<Packet>) {
        self.buffer.extend(packets);
    }

    pub(crate) fn clear(&mut self) {
        self.buffer.clear();
    }

    pub(crate) fn is_full(&self) -> bool {
        self.buffer.len() >= self.size
    }
}
