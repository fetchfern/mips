use std::{
  cell::{Cell, UnsafeCell},
  ops::{Deref, DerefMut},
};

const fn isolate_opcode(instr: u32) -> u32 {
  (instr >> 26) & ((1 << 6) - 1)
}

const fn isolate_funct(instr: u32) -> u32 {
  instr & ((1 << 6) - 1)
}

const fn isolate_rs(instr: u32) -> u32 {
  (instr >> 21) & ((1 << 5) - 1)
}

const fn isolate_rt(instr: u32) -> u32 {
  (instr >> 16) & ((1 << 5) - 1)
}

const fn isolate_rd(instr: u32) -> u32 {
  (instr >> 11) & ((1 << 5) - 1)
}

fn register_triad(instr: u32, reg: &RegisterMem) -> (ReleaseGuard, ReleaseGuard, ReleaseGuard) {
  let rd = isolate_rd(instr);
  let rs = isolate_rs(instr);
  let rt = isolate_rt(instr);

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

pub struct ReleaseGuard<'a> {
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
pub struct RegisterMem {
  pub area: [UnsafeCell<u8>; 32],
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
#[derive(Debug)]
pub struct Cpu<'a> {
  program: &'a [u8],
  ptr: usize,
  pub reg: RegisterMem,
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
    let instr = u32::from_le_bytes(bytes.try_into().unwrap());

    // instruction flow: according to this documentation
    // https://www.math.unipd.it/~sperduti/ARCHITETTURE-1/mips32.pdf

    let opcode = isolate_opcode(instr);

    match opcode {
      0 => opcode_zero_hdl(instr, &self.reg),
      _ => todo!(),
    }
  }
}
