#![feature(bigint_helper_methods)]
#![allow(unused_variables)]
use std::{
  cell::{RefCell, RefMut},
  fmt,
};

use k9::{assert_equal as assert_eq, assert_lesser_than as assert_lt};

const TEXT_START: u32 = 0x00400000;
const EXTERN_START: u32 = 0x10000000;
const TEXT_END: u32 = EXTERN_START - 1;
const HEAP_START: u32 = 0x10040000;
const EXTERN_END: u32 = HEAP_START - 1;
const KTEXT_START: u32 = 0x80000000;
const HEAP_END: u32 = KTEXT_START - 1;
const KDATA_START: u32 = 0x90000000;
const KTEXT_END: u32 = KDATA_START - 1;

struct Memory {
  pub text: Vec<u8>,
  pub data: Vec<u8>,
}

impl Memory {
  pub fn load_word(&mut self, addr: u32) -> u32 {
    assert_eq!(addr / 4, 0, "unaligned memory read");

    match addr {
      TEXT_START..=TEXT_END => {
        // .text
        let relative = (addr - TEXT_START) as usize;
        let bytes = self
          .text
          .get(relative..relative + 4)
          .expect("read into uninitialized memory");
        u32::from_le_bytes(bytes.try_into().unwrap())
      }

      _ => panic!("invalid mem reference"),
    }
  }
}

fn twos_complement_overflowed(n1: u32, n2: u32, r: u32) -> bool {
  // last_bit(n1) SAME AS last_bit(n2) AND last_bit(n1) DIFFERENT last_bit(result)
  // (last_bit(n1) XOR last_bit(n2) == 0) AND (last_bit(n1) XOR last_bit(result) == 1)
  (n1 ^ n2) < (1 << 31) && (n1 ^ r) > u32::MAX >> 1
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

  pub(super) const fn shamt(instr: u32) -> u32 {
    (instr >> 6) & ((1 << 5) - 1)
  }

  pub(super) const fn imm16(instr: u32) -> u16 {
    (instr & ((1 << 16) - 1)) as u16
  }
}

/// Creates a tuple of references to rd, rs, and rt
fn register_triad(instr: u32, reg: &Registers) -> (RefMut<u32>, RefMut<u32>, RefMut<u32>) {
  let rd = isolate::rd(instr);
  let rs = isolate::rs(instr);
  let rt = isolate::rt(instr);

  (reg.r(rd as usize), reg.r(rs as usize), reg.r(rt as usize))
}

/// Handles functs under opcode zero
fn handle_opcode_zero(instr: u32, reg: &mut Registers) {
  let funct = isolate::funct(instr);

  match funct {
    0x0 => {
      // sll rd, rt, shamt
      let (mut rd, _, rt) = register_triad(instr, reg);
      let shamt = isolate::shamt(instr);

      *rd = *rt << shamt;
    }

    0x4 => {
      // sllv rd, rt, rs
      let (mut rd, rs, rt) = register_triad(instr, reg);

      *rd = *rt << *rs;
    }

    0x3 => {
      // sra rd, rt, shamt
      let (mut rd, _, rt) = register_triad(instr, reg);
      let shamt = isolate::shamt(instr);

      *rd = (*rt >> shamt) | (*rt & (1 << 31))
    }

    0x19 => {
      // multu
      let rs = isolate::rs(instr);
      let rt = isolate::rt(instr);

      let (lo, hi) = u32::widening_mul(rs, rt);
      reg.hi = hi;
      reg.lo = lo;
    }
    0x20 => {
      // add
      let (mut rd, rs, rt) = register_triad(instr, reg);
      let result = u32::wrapping_add(*rs, *rt);

      if twos_complement_overflowed(*rs, *rt, result) {
        panic!("trap: overflow");
      }

      *rd = result;
    }

    0x21 => {
      // addu
      let (mut rd, rs, rt) = register_triad(instr, reg);
      *rd = u32::wrapping_add(*rs, *rt);
    }

    0x22 => {
      // sub
      let (mut rd, rs, rt) = register_triad(instr, reg);
      let result = u32::wrapping_sub(*rs, *rt);

      if twos_complement_overflowed(*rs, *rt, result) {
        panic!("trap: overflow");
      }

      *rd = result;
    }

    0x23 => {
      // subu
      let (mut rd, rs, rt) = register_triad(instr, reg);
      *rd = u32::wrapping_sub(*rs, *rt);
    }

    0x24 => {
      // and
      let (mut rd, rs, rt) = register_triad(instr, reg);
      *rd = *rs & *rt;
    }

    0x25 => {
      // or
      let (mut rd, rs, rt) = register_triad(instr, reg);
      *rd = *rs | *rt;
    }

    0x26 => {
      // xor
      let (mut rd, rs, rt) = register_triad(instr, reg);
      *rd = *rs ^ *rt;
    }

    0x27 => {
      // nor
      let (mut rd, rs, rt) = register_triad(instr, reg);
      *rd = !(*rs | *rt);
    }

    _ => todo!(),
  }
}

#[derive(Debug, Default)]
struct Registers {
  regular: [RefCell<u32>; 32],
  pub pc: u32,
  pub hi: u32,
  pub lo: u32,
}

impl Registers {
  pub fn r(&self, n: usize) -> RefMut<u32> {
    assert_lt!(n, 32, "internal VM fault: register idx out of range");
    self.regular[n].borrow_mut()
  }
}

/// MIPS bytecote interpreter which runs one program, then dies.
pub struct Cpu {
  memory: Memory,
  registers: Registers,
}

impl Cpu {
  pub fn new(program: Vec<u8>) -> Cpu {
    let registers = Registers::default();
    *registers.r(8) = 1;
    *registers.r(9) = 2;

    Cpu {
      memory: Memory {
        text: program,
        data: Vec::new(),
      },
      registers,
    }
  }

  pub fn next(&mut self) {
    let instr = self.memory.load_word(TEXT_START + self.registers.pc);

    // instruction flow: according to this documentation
    // https://www.math.unipd.it/~sperduti/ARCHITETTURE-1/mips32.pdf

    let opcode = isolate::opcode(instr);

    match opcode {
      0 => handle_opcode_zero(instr, &mut self.registers),
      _ => todo!(),
    }
  }
}

impl fmt::Debug for Cpu {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "PC: {}", self.registers.pc)?;

    for (i, value_cell) in self.registers.regular.iter().enumerate() {
      writeln!(f, "r{i}: {:#04x}", *value_cell.borrow())?
    }

    write!(f, "")
  }
}
