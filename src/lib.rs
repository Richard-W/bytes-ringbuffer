//! This crate provides a simple ringbuffer that implements the [Buf](../bytes/trait.Buf.html) and
//! [BufMut](../bytes/trait.BufMut.html) traits from the [bytes](../bytes/index.html) crate.
//!
//! ```
//! # extern crate bytes_ringbuffer;
//! # use bytes_ringbuffer::*;
//! # fn main() {
//! let mut buf = RingBuffer::new(4);
//! buf.put_u16(1234);
//! buf.put_u16(5671);
//! assert_eq!(buf.get_u16(), 1234);
//! assert_eq!(buf.get_u16(), 5671);
//! # }
//! ```
extern crate bytes;

use std::mem::MaybeUninit;

pub use bytes::{Buf, BufMut};

/// Fixed-capacity buffer
#[derive(Debug, Clone)]
pub struct RingBuffer {
    buffer: Vec<MaybeUninit<u8>>,
    begin: usize,
    len: usize,
}

impl RingBuffer {
    /// Create a ringbuffer with the given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![MaybeUninit::uninit(); capacity],
            begin: 0,
            len: 0,
        }
    }

    /// Capacity of the ringbuffer
    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }
}

impl Buf for RingBuffer {
    fn remaining(&self) -> usize {
        self.len
    }

    fn bytes(&self) -> &[u8] {
        let end = (self.begin + self.len).min(self.capacity());
        let slice = &self.buffer[self.begin..end];
        // Safe because `slice` is a subset of the bytes that have been declared
        // initialized by the unsafe `BufMut::advance_mut` function.
        unsafe { &*(slice as *const [MaybeUninit<u8>] as *const [u8]) }
    }

    fn advance(&mut self, cnt: usize) {
        assert!(cnt <= self.len);
        self.begin += cnt;
        self.begin %= self.capacity();
        self.len -= cnt;
    }
}

impl BufMut for RingBuffer {
    fn remaining_mut(&self) -> usize {
        self.capacity() - self.remaining()
    }

    fn bytes_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        let begin = (self.begin + self.len) % self.capacity();
        let end = (begin + self.remaining_mut()).min(self.capacity());
        &mut self.buffer[begin..end]
    }

    unsafe fn advance_mut(&mut self, cnt: usize) {
        assert!(cnt <= self.remaining_mut());
        self.len += cnt;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ringbuffer_init() {
        let buf = RingBuffer::new(16);
        assert_eq!(buf.capacity(), 16);
        assert_eq!(buf.remaining(), 0);
        assert_eq!(buf.remaining_mut(), 16);
    }

    #[test]
    fn ringbuffer_read_write() {
        let mut buf = RingBuffer::new(16);

        // Write 1..6 to buffer
        for i in 0..6 {
            buf.put_u8(i);
        }
        assert_eq!(buf.remaining(), 6);
        assert_eq!(buf.remaining_mut(), 10);
        // Read 1..6 from buffer
        for i in 0..6 {
            assert_eq!(buf.get_u8(), i);
        }

        // Write over the end
        for i in 0..16 {
            buf.put_u8(i);
        }
        assert_eq!(buf.remaining(), 16);
        assert_eq!(buf.remaining_mut(), 0);
        // bytes() should be a slice of length 10
        assert_eq!(buf.bytes().len(), 10);
        for i in 0..10 {
            assert_eq!(buf.get_u8(), i);
        }
        // Now bytes() should be a slice of length 6
        assert_eq!(buf.bytes().len(), 6);
        // Empty the buffer
        for i in 10..16 {
            assert_eq!(buf.get_u8(), i);
        }
    }

    #[test]
    #[should_panic]
    fn ringbuffer_read_over_remaining() {
        let mut buf = RingBuffer::new(16);
        for i in 0..15 {
            buf.put_u8(i);
        }
        for _ in 0..16 {
            buf.get_u8();
        }
    }

    #[test]
    #[should_panic]
    fn ringbuffer_write_over_capacity() {
        let mut buf = RingBuffer::new(16);
        for i in 0..17 {
            buf.put_u8(i);
        }
    }
}
