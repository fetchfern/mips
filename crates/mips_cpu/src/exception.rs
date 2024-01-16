use crate::cycle;

#[repr(u8)]
#[derive(Debug)]
/// An unexpected change in control flow.
pub enum Exception {
  /// Address error caused by a load or an instruction fetch. Happens when reading
  /// uninitialized or unauthorized memory.
  AddrLoadFetch = 0x4,
  /// Address error by a store. Happens when triggering a store operation on an address
  /// which does not allow writes.
  AddrStore = 0x5,
  /// Exception raised by a system call.
  Syscall = 0x8,
  /// Arithmetic overflow error.
  Overflow = 0xb,
  /// Traps are synchronous exceptions caused by instructions constructed for this purpose,
  /// such as `teq`, `tne`, `tlt`, and more.
  Trap = 0xc,
}

#[derive(Debug)]
/// Error which can either be the of error type `T` or a VM internal error.
pub enum Unstable<T> {
  Normal(T),
  VmError(String),
}

impl<T> From<Unstable<T>> for cycle::Next
where
  T: Into<cycle::Next>,
{
  fn from(value: Unstable<T>) -> Self {
    match value {
      Unstable::VmError(reason) => cycle::Next::VmError(reason),
      Unstable::Normal(e) => e.into(),
    }
  }
}
