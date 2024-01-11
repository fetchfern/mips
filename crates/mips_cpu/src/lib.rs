use std::cell::{Cell, UnsafeCell};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::fmt;

// opcode | rs    | rt    | rd    | funct
// 000000 | 01000 | 01001 | 01010 | 100001

const fn isolate_opcode(instr: u32) -> u32 {
  (instr >> 26) & ((1 << 6) - 1)
}

const fn isolate_funct(instr: u32) -> u32 {
  instr & ((1 << 6) - 1)
}

const fn isolate_rs(instr: u32) -> u32 {
  (instr >> 16) & ((1 << 5) - 1)
}

const fn isolate_rt(instr: u32) -> u32 {
  (instr >> 11) & ((1 << 5) - 1)
}

const fn isolate_rd(instr: u32) -> u32 {
  (instr >> 6) & ((1 << 5) - 1)
}

fn register_triad(instr: u32, reg: &RegisterMem) -> (ReleaseGuard, ReleaseGuard, ReleaseGuard) {
  let rd = isolate_rd(instr);
  let rs = isolate_rs(instr);
  let rt = isolate_rt(instr);

  println!("{rd}, {rs}, {rt}");

  (reg.r(rd as usize), reg.r(rs as usize), reg.r(rt as usize))
}

fn opcode_zero_hdl(instr: u32, reg: &RegisterMem) {
  let funct = isolate_funct(instr);

  match funct {
    0x21 => {
      let (mut rd, rs, rt) = register_triad(instr, reg);

      let (value, _) = u8::overflowing_add(*rs, *rt);

      *rd = value;
    }
    _ => todo!(),
  }
}

struct ReleaseGuard<'a> {
  ptr: &'a mut u8,
  parent: &'a RegisterMem,
  idx: usize,
}

impl<'a> Drop for ReleaseGuard<'a> {
  fn drop(&mut self) {
    self.parent.mut_borrow_mask[self.idx].set(false);
  }
}

impl<'a> Deref for ReleaseGuard<'a> {
  type Target = u8;

  fn deref(&self) -> &Self::Target {
    &*self.ptr
  }
}

impl<'a> DerefMut for ReleaseGuard<'a> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.ptr
  }
}

#[derive(Debug, Default)]
struct RegisterMem {
  area: [UnsafeCell<u8>; 32],
  mut_borrow_mask: [Cell<bool>; 32],
}

impl RegisterMem {
  pub fn r(&self, n: usize) -> ReleaseGuard {
    assert!(n <= 32, "register idx out of range");

    if self.mut_borrow_mask[n].get() {
      panic!("race condition on register {n}");
    }

    self.mut_borrow_mask[n].set(true);

    ReleaseGuard {
      ptr: unsafe { &mut *self.area[n].get() },
      parent: self,
      idx: n,
    }
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

    let opcode = isolate_opcode(instr);

    match opcode {
      0 => opcode_zero_hdl(instr, &self.registers),
      _ => todo!(),
    }
  }
}

impl Debug for Cpu<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for value in self.registers.mut_borrow_mask.iter() {
            if value.get() {
                panic!("race condition: debug fmt while register borrowed");
            }
        }

        writeln!(f, "PC: {}", self.pc)?;

        for (i, value_cell) in self.registers.area.iter().enumerate() {
            writeln!(f, "r{i}: {:#04x}", unsafe { *value_cell.get() })?
        } 

        write!(f, "")
    }
}
