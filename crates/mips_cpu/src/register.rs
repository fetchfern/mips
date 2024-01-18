use crate::exception::{Exception, Unstable};
use crate::mem::TEXT_START;
use std::cell::{BorrowError, Ref, RefCell, RefMut};

/// A collection of registers present in a MIPS32 CPU. Contains the regular
/// 32 registers, the PC and the HI/LO registers.  
///
/// All registers span 4 bytes (a word).  
///
/// For ease of use, regular registers are protected in a `RefCell`. This might
/// change in the future.
#[derive(Debug)]
pub struct Registers {
  regular: [RefCell<u32>; 32],
  pub pc: u32,
  pub hi: u32,
  pub lo: u32,
}

impl Registers {
  /// Initialize the registers with sensible initial values. Subject to change.
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

  /// Tries to get mutable access to a regular register. Returns `Result::Err` if
  /// the given register is not in range `0..32` or if the register is already mutably
  /// borrowed.
  pub fn r(&self, n: usize) -> Result<RefMut<u32>, Unstable<Exception>> {
    self
      .regular
      .get(n)
      .ok_or_else(|| Unstable::VmError("requested register out of range".to_owned()))
      .and_then(|r| {
        r.try_borrow_mut()
          .map_err(|_| Unstable::VmError("race condition while borrowing register".to_owned()))
      })
  }

  /// Load the next instruction address (PC+4) into the given register. Typical operation of
  /// jump-and-link or branch-and-link instructions.
  pub fn link(&self, n: usize) -> Result<(), Unstable<Exception>> {
    let mut r = self.r(n)?;
    *r = self.pc + 4;
    Ok(())
  }

  /// Tries to borrow every value in regular registers, for debugging purposes.
  pub fn regular_values(&self) -> [Result<Ref<u32>, BorrowError>; 32] {
    // collected vector cannot not be length 32
    #[allow(clippy::unwrap_used)]
    self
      .regular
      .iter()
      .map(RefCell::try_borrow)
      .collect::<Vec<_>>()
      .try_into()
      .unwrap()
  }
}
