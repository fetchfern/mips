use super::segmented_store::SegmentedStore;
use std::ops::Range;

#[derive(Debug)]
struct ContinuousRegion {
  index: usize,
  data: Vec<u8>,
}

impl ContinuousRegion {
  pub fn range(&self) -> Range<usize> {
    self.index..self.index + self.data.len()
  }
}

#[derive(Debug, Default)]
pub struct HybridStore {
  regions: Vec<ContinuousRegion>,
  fallback: SegmentedStore,
}

impl HybridStore {
  pub fn new() -> Self {
    Self {
      regions: Vec::new(),
      fallback: SegmentedStore::new(),
    }
  }

  /// Insert a continuous chunk of memory at a certain memory index.  
  pub fn insert_continuous(&mut self, index: usize, data: Vec<u8>) {
    self.regions.push(ContinuousRegion { index, data });
  }

  pub fn read(&self, index: usize) -> Option<&[u8]> {
    self
      .try_read_continuous(index)
      .or_else(|| self.fallback.read_continuous(index))
  }

  pub fn read_byte(&self, index: usize) -> Option<u8> {
    self.read(index).and_then(|sl| sl.first().copied())
  }

  pub fn read_halfword(&self, index: usize) -> Option<u16> {
    // This is starting to get annoying
    //
    // The two bytes could be in a continuous region. Easy!
    //
    // or...
    //
    // One of them could be in a continuous region and another in the
    // fallback store. Or not exist. And it gets worse in `read_word`.
    //
    // Getting to use `read_byte` for each byte isn't all bad thankfully,
    // although maybe a bit unefficient.

    Some(u16::from_le_bytes([
      self.read_byte(index)?,
      self.read_byte(index + 1)?,
    ]))
  }

  pub fn read_word(&self, index: usize) -> Option<u32> {
    Some(u32::from_le_bytes([
      self.read_byte(index)?,
      self.read_byte(index + 1)?,
      self.read_byte(index + 2)?,
      self.read_byte(index + 3)?,
    ]))
  }

  fn try_read_continuous(&self, index: usize) -> Option<&[u8]> {
    self
      .regions
      .iter()
      .find(|r| r.range().contains(&index))
      .map(|r| &r.data[index - r.index..])
  }
}
