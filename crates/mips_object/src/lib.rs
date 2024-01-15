/// Block of data with labels at certain byte positions
pub struct LabeledBlock {
  pub raw_data: Vec<u8>,
}

/// Object defition of a MIPS32 assembly source
pub struct Object {
  /// .text block, contains user program code.
  pub text: LabeledBlock,
}
