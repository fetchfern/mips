/// A size-bound continuous data store. It's nothing more than a
/// wrapper around a `Vec`.
#[derive(Debug)]
pub struct Continuous {
  data: Vec<u8>,
  max_size: usize,
}

impl Continuous {
  /// Create a new `Continuous` data store with the specified size limit.
  pub fn init(max_size: usize) -> Self {
    Self {
      // skip a good 8 small relocations
      data: Vec::with_capacity(512),
      max_size,
    }
  }

  /// Read a single byte out the data store.
  pub fn read_byte(&self, index: usize) -> Option<u8> {
    debug_assert!(index < self.max_size, "index over theoretical limit");

    self.data.get(index).copied()
  }

  /// Read a half word (2 bytes) out of the data store.
  pub fn read_halfword(&self, index: usize) -> Option<u16> {
    debug_assert!(index + 1 < self.max_size, "index over theoretical limit");
    let bytes = [
      self.data.get(index).copied()?,
      self.data.get(index + 1).copied()?,
    ];

    Some(u16::from_le_bytes(bytes))
  }

  /// Read a whole word (4 bytes) out of the data store.
  pub fn read_word(&self, index: usize) -> Option<u32> {
    debug_assert!(index + 3 < self.max_size, "index over theoretical limit");
    let mut bytes = [0, 0, 0, 0];

    self
      .data
      .get(index..index + 4)?
      .iter()
      .enumerate()
      .for_each(|(i, v)| bytes[i] = *v);

    Some(u32::from_le_bytes(bytes))
  }
}
