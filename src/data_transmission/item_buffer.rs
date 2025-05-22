use std::ops::RangeTo;

pub(crate) struct ItemBuffer<T> {
    buffer: Vec<T>,
    size: usize,
}

impl<T: Clone> ItemBuffer<T> {
    pub(crate) fn new(size: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(size),
            size,
        }
    }

    pub(crate) fn push(&mut self, item: T) {
        self.buffer.push(item);
    }

    pub(crate) fn take(&mut self) -> Vec<T> {
        std::mem::take(&mut self.buffer)
    }

    pub(crate) fn get(&mut self, range: RangeTo<usize>) -> Vec<T> {
        self.buffer.get(range).unwrap_or_default().to_vec()
    }

    pub(crate) fn extend(&mut self, items: Vec<T>) {
        self.buffer.extend(items);
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
