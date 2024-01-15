use crate::cycle::{Trigger, CycleResult};
use crate::mem::TEXT_START;
use std::cell::{RefCell, RefMut, BorrowError, Ref};

#[derive(Debug)]
pub struct Registers {
  regular: [RefCell<u32>; 32],
  pub pc: u32,
  pub hi: u32,
  pub lo: u32,
}

impl Registers {
  pub fn init() -> Registers {
    let regular = <[RefCell<u32>; 32]>::default();
    *regular[8].borrow_mut() = 3;
    *regular[9].borrow_mut() = 4;

    Registers {
      regular,
      pc: TEXT_START,
      hi: 0,
      lo: 0,
    }
  }

  pub fn r(&self, n: usize) -> CycleResult<RefMut<u32>> {
    if n >= 32 {
      return Err(Trigger::VmError(format!("register index out of range (expected < 32, got {n})")));
    }

    // prior check for n < 32
    #[allow(clippy::unwrap_used)]
    Ok(self.regular.get(n).unwrap().borrow_mut())
  }

  pub fn regular_values(&self) -> [Result<Ref<u32>, BorrowError>; 32] {
    // collected vector cannot not be length 32
    #[allow(clippy::unwrap_used)]
    self.regular
      .iter()
      .map(RefCell::try_borrow)
      .collect::<Vec<_>>()
      .try_into()
      .unwrap()
  }
}
