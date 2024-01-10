#[derive(Debug, Default)]
struct RegisterMem {
  area: [u8; 32],
}

impl RegisterMem {
  pub fn r(&mut self, n: usize) -> &mut u8 {
    &mut self.area[n]
  }
}

/// MIPS interpreter which runs one program, then dies.
pub struct Cpu<'a> {
  program: &'a [u8],
  ptr: usize,
  reg: RegisterMem,
}

impl Cpu<'_> {
  pub fn new(program: &[u8]) -> Cpu<'_> {
    Cpu {
      program,
      ptr: 0,
      reg: Default::default(),
    }
  }

  pub fn next(&mut self) {
    let bytes = &self.program[self.ptr..self.ptr + 4];
    let _instr = u32::from_le_bytes(bytes.try_into().unwrap());
  }
}
