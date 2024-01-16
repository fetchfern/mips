use crate::exception::Exception;

macro_rules! expect {
  ($e:expr) => {
    match ($e) {
      ::std::result::Result::Ok(v) => v,
      ::std::result::Result::Err(v) => {
        return ::std::convert::Into::<$crate::cycle::Next>::into(v);
      }
    }
  }
}

/// Specifies the resolution of a cycle.
pub enum Next {
  /// Loaded instruction performed with no issues, the next instruction can
  /// safely be loaded.
  Forward,
  /// Branch/Jump needs to be performed
  Branch(u32),
  /// Issue an exception. Depending on the exception configuration on coproc0,
  /// branch execution to exception handler.
  Exception(Exception),
  /// Virtual machine internal error.
  VmError(String),
}

impl From<Exception> for Next {
  fn from(value: Exception) -> Self {
    Next::Exception(value)
  }
}

pub use compute::perform_cycle;

mod data;
mod compute;
