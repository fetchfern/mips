use crate::mem::{MemoryMap, TEXT_START};
use crate::register::Registers;
use std::cell::RefMut;

fn isolate_opcode(instr: u32) -> u32 {
  (instr >> 26) & ((1 << 6) - 1)
}

fn isolate_funct(instr: u32) -> u32 {
  instr & ((1 << 6) - 1)
}

fn isolate_rs(instr: u32) -> u32 {
  (instr >> 21) & ((1 << 5) - 1)
}

fn isolate_rt(instr: u32) -> u32 {
  (instr >> 16) & ((1 << 5) - 1)
}

fn isolate_rd(instr: u32) -> u32 {
  (instr >> 11) & ((1 << 5) - 1)
}

fn isolate_shamt(instr: u32) -> u32 {
  (instr >> 6) & ((1 << 5) - 1)
}

fn isolate_imm16(instr: u32) -> u16 {
  (instr & ((1 << 16) - 1)) as u16
}

fn twos_complement_overflowed(n1: u32, n2: u32, r: u32) -> bool {
  // last_bit(n1) SAME AS last_bit(n2) AND last_bit(n1) DIFFERENT last_bit(result)
  // (last_bit(n1) XOR last_bit(n2) == 0) AND (last_bit(n1) XOR last_bit(result) == 1)
  (n1 ^ n2) < 0x80000000 && (n1 ^ r) > 0x7fffffff
}

fn sign_extend(n: u16) -> u32 {
  let n = n as u32;
  let ext = 0xffffu32 * (n >> 15);
  n | ext << 16
}

/// Creates a tuple of references to rd, rs, and rt
fn register_triad(instr: u32, reg: &Registers) -> (RefMut<u32>, RefMut<u32>, RefMut<u32>) {
  // we know isolate_r* cannot return values higher or equal to 32, so it's okay
  #[allow(clippy::unwrap_used)]
  {
    let rd = reg.r(isolate_rd(instr) as usize).unwrap();
    let rs = reg.r(isolate_rs(instr) as usize).unwrap();
    let rt = reg.r(isolate_rt(instr) as usize).unwrap();

    (rd, rs, rt)
  }
}

pub type CycleResult<T> = std::result::Result<T, Trigger>;

#[derive(Debug)]
pub enum Fault {
  UninitRead,
  UnalignedAccess,
}

/// Cause for a deviation in normal execution.
#[derive(Debug)]
pub enum Trigger {
  /// Successful branching occured. In case of branch delay enabled, treat this
  /// as a `Next` variant, and apply the branch change if the next cycle yields
  /// a `Next` variant.
  Branch(u32),
  /// Trap occured
  Trap,
  /// Fault occured during execution
  Fault(Fault),
  /// Internal virtual machine fault
  VmError(String),
}

/// Perform the next cycle (as pointed by the current program counter). This
/// function does NOT write to the program counter, the caller is responsible
/// for updating the PC depending on the cycle result.
pub fn perform_cycle(memory: &mut MemoryMap, registers: &mut Registers) -> CycleResult<()> {
  let instr = memory.load_word(registers.pc)?;

  // instruction flow: according to this documentation
  // https://www.math.unipd.it/~sperduti/ARCHITETTURE-1/mips32.pdf

  let opcode = isolate_opcode(instr);

  match opcode {
    0 => handle_zero_opcode(instr, memory, registers),

    0x8 => {
      // addi rt, rs, imm16
      let mut rt = registers.r(isolate_rt(instr) as usize)?;
      let rs = registers.r(isolate_rs(instr) as usize)?;
      let imm16 = isolate_imm16(instr);

      let addend0 = *rs;
      let addend1 = sign_extend(imm16);
      let sum = addend0 + addend1;

      if twos_complement_overflowed(addend0, addend1, sum) {
        return Err(Trigger::Trap);
      }

      *rt = sum;

      Ok(())
    }

    0x9 => {
      // addiu rt, rs, imm16
      let mut rt = registers.r(isolate_rt(instr) as usize)?;
      let rs = registers.r(isolate_rs(instr) as usize)?;
      let imm16 = isolate_imm16(instr);

      *rt = *rs + sign_extend(imm16);
      Ok(())
    }

    0xa => {
      // slti rt, rs, imm16
      let mut rt = registers.r(isolate_rt(instr) as usize)?;
      let rs = registers.r(isolate_rs(instr) as usize)?;
      let imm16 = isolate_imm16(instr);

      *rt = (*rs - sign_extend(imm16)) >> 31;
      Ok(())
    }

    0xb => {
      // sltiu rt, rs, imm16
      let mut rt = registers.r(isolate_rt(instr) as usize)?;
      let rs = registers.r(isolate_rs(instr) as usize)?;
      let imm16 = isolate_imm16(instr);

      *rt = (*rs < sign_extend(imm16)) as u32;
      Ok(())
    }

    0xc => {
      // andi rt, rs, imm16
      let mut rt = registers.r(isolate_rt(instr) as usize)?;
      let rs = registers.r(isolate_rs(instr) as usize)?;
      let imm16 = isolate_imm16(instr);

      *rt = *rs & imm16 as u32;
      Ok(())
    }

    0xd => {
      // ori rt, rs, imm16
      let mut rt = registers.r(isolate_rt(instr) as usize)?;
      let rs = registers.r(isolate_rs(instr) as usize)?;
      let imm16 = isolate_imm16(instr);

      *rt = *rs | imm16 as u32;
      Ok(())
    }

    0xe => {
      // xori rt, rs, imm16
      let mut rt = registers.r(isolate_rt(instr) as usize)?;
      let rs = registers.r(isolate_rs(instr) as usize)?;
      let imm16 = isolate_imm16(instr);

      *rt = *rs ^ imm16 as u32;
      Ok(())
    }

    0xf => {
      // lui rt, imm16
      let hword = isolate_imm16(instr);
      let mut rt = registers.r(isolate_rt(instr) as usize)?;
      *rt = (hword as u32) << 16;
      Ok(())
    }

    _ => unimplemented!(),
  }
}

fn handle_zero_opcode(
  instr: u32,
  memory: &mut MemoryMap,
  registers: &mut Registers,
) -> CycleResult<()> {
  let funct = isolate_funct(instr);

  match funct {
    0x0 => {
      // sll rd, rt, shamt
      let (mut rd, _, rt) = register_triad(instr, registers);
      let shamt = isolate_shamt(instr);

      *rd = *rt << shamt;
    }

    0x4 => {
      // sllv rd, rt, rs
      let (mut rd, rs, rt) = register_triad(instr, registers);

      *rd = *rt << *rs;
    }

    0x3 => {
      // sra rd, rt, shamt
      let (mut rd, _, rt) = register_triad(instr, registers);
      let shamt = isolate_shamt(instr);

      *rd = (*rt >> shamt) | (*rt & (1 << 31))
    }

    0x19 => {
      // multu
      let (_, rs, rt) = register_triad(instr, registers);

      let (lo, hi) = u32::widening_mul(*rs, *rt);
      drop((rs, rt));
      registers.hi = hi;
      registers.lo = lo;
    }
    0x20 => {
      // add
      let (mut rd, rs, rt) = register_triad(instr, registers);
      let result = u32::wrapping_add(*rs, *rt);

      if twos_complement_overflowed(*rs, *rt, result) {
        return Err(Trigger::Trap);
      }

      *rd = result;
    }

    0x21 => {
      // addu
      let (mut rd, rs, rt) = register_triad(instr, registers);
      *rd = u32::wrapping_add(*rs, *rt);
    }

    0x22 => {
      // sub
      let (mut rd, rs, rt) = register_triad(instr, registers);
      let result = u32::wrapping_sub(*rs, *rt);

      if twos_complement_overflowed(*rs, *rt, result) {
        return Err(Trigger::Trap);
      }

      *rd = result;
    }

    0x23 => {
      // subu
      let (mut rd, rs, rt) = register_triad(instr, registers);
      *rd = u32::wrapping_sub(*rs, *rt);
    }

    0x24 => {
      // and
      let (mut rd, rs, rt) = register_triad(instr, registers);
      *rd = *rs & *rt;
    }

    0x25 => {
      // or
      let (mut rd, rs, rt) = register_triad(instr, registers);
      *rd = *rs | *rt;
    }

    0x26 => {
      // xor
      let (mut rd, rs, rt) = register_triad(instr, registers);
      *rd = *rs ^ *rt;
    }

    0x27 => {
      // nor
      let (mut rd, rs, rt) = register_triad(instr, registers);
      *rd = !(*rs | *rt);
    }

    _ => todo!(),
  }

  Ok(())
}
