use std::collections::VecDeque;
use std::ops::RangeTo;

// Backed by a VecDeque (not a Vec) so that repeatedly draining a batch off
// the front — the access pattern every caller uses — costs O(batch), not
// O(remaining length): a Vec::drain(..batch) has to shift the whole
// remaining tail down on every call, which turns catching up a large backlog
// into an O(n^2) sequence of memmoves.
pub(crate) struct ItemBuffer<T> {
    buffer: VecDeque<T>,
    size: usize,
}

impl<T: Clone> ItemBuffer<T> {
    pub(crate) fn new(size: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(size),
            size,
        }
    }

    pub(crate) fn push(&mut self, item: T) {
        self.buffer.push_back(item);
    }

    pub(crate) fn take(&mut self) -> Vec<T> {
        Vec::from(std::mem::take(&mut self.buffer))
    }

    pub(crate) fn get(&mut self, range: RangeTo<usize>) -> Vec<T> {
        self.buffer.iter().take(range.end).cloned().collect()
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
