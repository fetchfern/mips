use super::{Continuous, HybridStore, SegmentedStore};

/// Interface with encapsulates read operations with different storage
/// solutions.
pub enum IoInterface<'a> {
  Continuous(&'a Continuous),
  Hybrid(&'a HybridStore),
  Segmented(&'a SegmentedStore),
}

impl IoInterface<'_> {
  pub fn read_byte(&self, index: usize) -> Option<u8> {
    use IoInterface::*;

    match self {
      Continuous(c) => c.read_byte(index),
      Hybrid(h) => h.read_byte(index),
      Segmented(s) => s.read_byte(index),
    }
  }

  pub fn read_halfword(&self, index: usize) -> Option<u16> {
    use IoInterface::*;

    match self {
      Continuous(c) => c.read_halfword(index),
      Hybrid(h) => h.read_halfword(index),
      Segmented(s) => s.read_halfword(index),
    }
  }

  pub fn read_word(&self, index: usize) -> Option<u32> {
    use IoInterface::*;

    match self {
      Continuous(c) => c.read_word(index),
      Hybrid(h) => h.read_word(index),
      Segmented(s) => s.read_word(index),
    }
  }
}
