use std::collections::VecDeque;
use std::io::Read;

const SIZE: usize = 2048;

#[derive(Debug)]
pub struct Segment {
  index: usize,
  data: Box<[u8; SIZE]>,
}

impl Segment {
  pub fn zeroed(index: usize) -> Self {
    Self {
      index,
      data: Box::new([0; SIZE]),
    }
  }

  pub fn start(&self) -> usize {
    self.index
  }

  pub fn end(&self) -> usize {
    self.index + SIZE
  }

  /// If a specific index is located in this `Segment`
  pub fn contains(&self, index: usize) -> bool {
    index
      .checked_sub(self.index)
      .filter(|dist| *dist < SIZE)
      .is_some()
  }

  pub fn to_relative_index(&self, absolute: usize) -> Option<usize> {
    absolute.checked_sub(self.index)
  }

  pub fn bytes_at(&self, index: usize) -> &[u8] {
    debug_assert!(index < SIZE);
    &self.data[index..]
  }

  pub fn bytes_at_mut(&mut self, index: usize) -> &mut [u8] {
    debug_assert!(index < SIZE);
    &mut self.data[index..]
  }
}

/// Segmented storage, in blocks of 2048 bytes.
#[derive(Debug, Default)]
pub struct SegmentedStore {
  segments: VecDeque<Segment>,
}

impl SegmentedStore {
  pub fn new() -> Self {
    Self {
      segments: VecDeque::new(),
    }
  }

  /// Returns the segment which contains `index`, if it exists.
  pub fn find_segment(&self, index: usize) -> Option<&Segment> {
    let floor = (index / SIZE) * SIZE;

    self
      .segments
      .binary_search_by_key(&floor, |s| s.index)
      .ok()
      .and_then(|i| self.segments.get(i))
  }

  /// Returns the segment containing `index`, creating it if absent.
  pub fn find_segment_or_insert(&mut self, index: usize) -> &mut Segment {
    let floor = (index / SIZE) * SIZE;

    match self.segments.binary_search_by_key(&floor, |s| s.index) {
      Ok(i) => &mut self.segments[i],
      Err(i) => {
        // the segment definitely doesn't exist
        self.segments.insert(i, Segment::zeroed(floor));
        &mut self.segments[i]
      }
    }
  }

  /// Returns a slice of the continuous memory chunk which starts at `index`.
  pub fn read_continuous(&self, index: usize) -> Option<&[u8]> {
    self
      .find_segment(index)
      .and_then(|s| s.to_relative_index(index).map(|i| s.bytes_at(i)))
  }

  pub fn read_byte(&self, index: usize) -> Option<u8> {
    self
      .read_continuous(index)
      .and_then(|sl| sl.first().copied())
  }

  pub fn read_halfword(&self, index: usize) -> Option<u16> {
    Some(u16::from_le_bytes([
      self.read_byte(index)?,
      self.read_byte(index + 1)?,
    ]))
  }

  /// Read a word which might cross segment boundaries.
  #[inline]
  pub fn read_word(&self, index: usize) -> Option<u32> {
    if index.div_ceil(SIZE) == (index + 4).div_ceil(SIZE) {
      // does NOT cross segment boundaries
      self
        .read_continuous(index)
        .and_then(|sl| sl.get(0..4))
        .map(|s| u32::from_le_bytes(s.try_into().unwrap()))
    } else {
      let low = self.read_continuous(index)?;
      let missing_bytes = 4 - low.len();
      let high = self.read_continuous(index + low.len())?;

      let mut bytes = [0, 0, 0, 0];
      bytes[..low.len()].copy_from_slice(low);
      bytes[low.len()..].copy_from_slice(high.get(..missing_bytes)?);

      Some(u32::from_le_bytes(bytes))
    }
  }

  pub fn write(&mut self, index: usize, mut data: &[u8]) {
    let mut start = index - (index / SIZE) * SIZE;
    let mut blocks_traversed = 0;

    while !data.is_empty() {
      let segment = self.find_segment_or_insert(index + SIZE * blocks_traversed);
      let result = data.read(segment.bytes_at_mut(start));
      debug_assert!(result.is_ok());

      blocks_traversed += 1;
      start = 0;
    }
  }
}
