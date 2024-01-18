use crate::exception::Exception;

/// Specifies the resolution of a cycle.
pub enum Next {
  /// Loaded instruction performed with no issues, the next instruction can
  /// safely be loaded.
  Forward,
  /// Branch/Jump needs to be performed immediately
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

/// Actual code performing each instruction.
mod compute;
/// Operations on instructions.
mod data;
