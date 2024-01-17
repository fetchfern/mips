#![feature(is_sorted)]

use derive_more::Deref;
use interface::IoInterface;
use storage::continuous::Continuous;
use storage::hybrid_store::HybridStore;
use storage::segmented_store::SegmentedStore;

#[derive(Debug)]
/// A label, like `msg`, `main` and `loop` in:
///
/// ```asm
/// .data
/// msg: .asciiz "hello world"
/// .text
/// main:
///   addiu $t0, $z, 5
/// loop:
///   subiu $t0, $t0, 1
///   bnez $t0, loop
///
///   li $v0, 10
///   syscall
/// ```
pub struct Label {
  /// The position of the label, relative to the block. In the example, the `position` of
  /// both the labels `msg` and `main` would be 0, and the `position` of label `loop` would
  /// be 4.
  pub position: usize,
  /// The name of the label.
  pub name: String,
}

#[derive(Debug, Deref)]
pub struct Labeled<S> {
  #[deref]
  storage: S,
  labels: Vec<Label>,
}

impl<S> Labeled<S> {
  pub fn with_no_labels(storage: S) -> Self {
    Self {
      storage,
      labels: Vec::new(),
    }
  }
}

#[derive(Debug)]
/// Contains all data needed to run a MIPS program. Implements read/write restrictions.
pub struct ProgramData {
  /// `.text` block, contains user program code.  
  ///
  /// In most cases, this can be stored in a continuous block of memory
  /// with no need for fancy memory handling. But! `.text` is a rather large
  /// section of the memory map and if we allow self-modifying code, as well
  /// as fragmenting the code in regions distant from each-other, that would
  /// consume a lot of memory. The `.text` section thereby needs a sort of
  /// hybrid aproach, where the data is stored in blocks of continuous memory,
  /// depending on the user's source, and r/w operations issued on uninitialized
  /// `.text` data should fallback on a segmented store. Hence the `HybridStore`
  /// was created.
  text: Labeled<HybridStore>,
  /// `.extern block`, contains globals.
  ///
  /// This block is pretty short, we don't really need fancy memory storage.
  /// `Continuous` does the trick.
  r#extern: Labeled<Continuous>,
  /// `.data` block, contains static data.
  ///
  /// This one is also pretty short (if we don't count what we reserve for
  /// the heap), so `Continuous` storage will go.
  data: Labeled<Continuous>,
  /// Heap region.
  ///
  /// This one can get real big. A `SegmentedStore` is definitely needed.
  ///
  /// A pro of having all these complicated storage solutions is we can actually use
  /// all the address space available, while MARS is limited to `0x103fffff`.
  /// Trying to issue a write to an address above that (but below the stack)
  /// will result in an error! That's a pretty wide unusable range.
  ///
  /// A con is a that this level of flexibility is completely useless to almost
  /// anyone.
  #[allow(dead_code)]
  heap: Labeled<SegmentedStore>,
  /// `.ktext` block, contains kernel code
  ///
  /// The kernel text is the same story as `.text`.
  #[allow(dead_code)]
  ktext: Labeled<HybridStore>,
  /// `.kdata` block, contains kernel static data.
  ///
  /// Same story as the heap.
  #[allow(dead_code)]
  kdata: Labeled<SegmentedStore>,
}

impl ProgramData {
  pub fn builder() -> ProgramDataBuilder {
    ProgramDataBuilder::new()
  }

  pub fn labels(&self, section: Section) -> &[Label] {
    use Section::*;
    match section {
      Text => &self.text.labels,
      Extern => &self.r#extern.labels,
      Data => &self.data.labels,
    }
  }

  /// Request to read into a memory section.  
  ///
  /// Returns `None` if reading is unauthorized considering the `Context`.  
  /// Returns `Some(interface)` if reading is authorized.  
  pub fn read(&self, section: Section, _context: Context) -> Option<IoInterface<'_>> {
    use Section::*;

    match section {
      Text => {
        // whatever context is allowed to read .text
        Some(IoInterface::Hybrid(&self.text.storage))
      }

      Extern => {
        // whatever context is allowed to read .extern
        Some(IoInterface::Continuous(&self.r#extern.storage))
      }

      Data => {
        // whatever context is allowed to read .extern
        Some(IoInterface::Continuous(&self.data.storage))
      }
    }
  }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Section {
  Text,
  Extern,
  Data,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Context {
  User,
  Kernel,
  External,
}

/// Builder for `ProgramData`.
///
/// Definitely not complete
#[derive(Debug, Default)]
pub struct ProgramDataBuilder {
  text: Option<Vec<u8>>,
}
impl ProgramDataBuilder {
  pub fn new() -> Self {
    ProgramDataBuilder { text: None }
  }

  pub fn text(mut self, text: Vec<u8>) -> Self {
    self.text = Some(text);
    self
  }

  pub fn build(self) -> ProgramData {
    let mut text_store = HybridStore::new();
    if let Some(text) = self.text {
      text_store.insert_continuous(0, text);
    }

    ProgramData {
      text: Labeled::with_no_labels(text_store),
      r#extern: Labeled::with_no_labels(Continuous::init(0)),
      data: Labeled::with_no_labels(Continuous::init(0)),
      heap: Labeled::with_no_labels(SegmentedStore::new()),
      ktext: Labeled::with_no_labels(HybridStore::new()),
      kdata: Labeled::with_no_labels(SegmentedStore::new()),
    }
  }
}

pub mod interface;
mod storage;
