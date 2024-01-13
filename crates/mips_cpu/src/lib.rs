use k9::assert_lesser_than as assert_lt;
use std::cell::{RefCell, RefMut};
use std::fmt;

struct Memory {
  pub text: Vec<u8>,
  pub data: Vec<u8>,
}

impl Memory {
  pub fn load_word(&mut self, addr: u32) -> u32 {
    
  }
}

// addu t2, t0, t1
// opcode | rs    | rt    | rd    | shamt | funct
// 000000 | 01000 | 01001 | 01010 | 00000 | 100001

/// Functions for isolating the bits of an instruction
mod isolate {
  pub(super) const fn opcode(instr: u32) -> u32 {
    (instr >> 26) & ((1 << 6) - 1)
  }

  pub(super) const fn funct(instr: u32) -> u32 {
    instr & ((1 << 6) - 1)
  }

  pub(super) const fn rs(instr: u32) -> u32 {
    (instr >> 21) & ((1 << 5) - 1)
  }

  pub(super) const fn rt(instr: u32) -> u32 {
    (instr >> 16) & ((1 << 5) - 1)
  }

  pub(super) const fn rd(instr: u32) -> u32 {
    (instr >> 11) & ((1 << 5) - 1)
  }

  pub(super) const fn imm16(instr: u32) -> u16 {
    (instr & ((1 << 16) - 1)) as u16
  }
}

/// Creates a tuple of references to rd, rs, and rt
fn register_triad(instr: u32, reg: &RegisterMem) -> (RefMut<u32>, RefMut<u32>, RefMut<u32>) {
  let rd = isolate::rd(instr);
  let rs = isolate::rs(instr);
  let rt = isolate::rt(instr);

  (reg.r(rd as usize), reg.r(rs as usize), reg.r(rt as usize))
}

/// Handles functs under opcode zero
fn handle_opcode_zero(instr: u32, reg: &RegisterMem) {
  let funct = isolate::funct(instr);

  match funct {
    0x21 => {
      // addu
      let (mut rd, rs, rt) = register_triad(instr, reg);

      let (value, _) = u32::overflowing_add(*rs, *rt);

      *rd = value
    }

    0x23 => {
      // subu
      let (mut rd, rs, rt) = register_triad(instr, reg);

      let (value, _) = u32::overflowing_sub(*rs, *rt);

      *rd = value
    }

    _ => todo!(),
  }
}

#[derive(Debug, Default)]
struct RegisterMem {
  area: [RefCell<u32>; 32],
}

impl RegisterMem {
  pub fn r(&self, n: usize) -> RefMut<u32> {
    assert_lt!(n, 32, "internal VM fault: register idx out of range");
    self.area[n].borrow_mut()
  }
}

/// MIPS bytecote interpreter which runs one program, then dies.
pub struct Cpu<'a> {
  program: &'a [u8],
  pc: usize,
  registers: RegisterMem,
}

impl Cpu<'_> {
  pub fn new(program: &[u8]) -> Cpu<'_> {
    let registers = RegisterMem::default();
    *registers.r(8) = 1;
    *registers.r(9) = 2;

    Cpu {
      program,
      pc: 0,
      registers,
    }
  }

  pub fn next(&mut self) {
    let bytes = &self.program[self.pc..self.pc + 4];
    let instr = u32::from_le_bytes(bytes.try_into().unwrap());

    // instruction flow: according to this documentation
    // https://www.math.unipd.it/~sperduti/ARCHITETTURE-1/mips32.pdf

    let opcode = isolate::opcode(instr);

    match opcode {
      0 => handle_opcode_zero(instr, &self.registers),
      _ => todo!(),
    }
  }
}

impl fmt::Debug for Cpu<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "PC: {}", self.pc)?;

    for (i, value_cell) in self.registers.area.iter().enumerate() {
      writeln!(f, "r{i}: {:#04x}", *value_cell.borrow())?
    }

    write!(f, "")
  }
}
