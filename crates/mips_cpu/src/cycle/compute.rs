use std::cell::RefMut;
use crate::register::Registers;
use crate::mem::MemoryMap;
use crate::exception::Exception;
use crate::cycle::{data, Next};

/// Parses instructions in format `i rd, rs, rt`
fn parse_arithm_r(instr: u32, reg: &Registers) -> (RefMut<u32>, RefMut<u32>, RefMut<u32>) {
  // data::isolate_r* cannot return values higher or equal to 32
  #[allow(clippy::unwrap_used)]
  let rd = reg.r(data::isolate_rd(instr) as usize).unwrap();
  #[allow(clippy::unwrap_used)]
  let rs = reg.r(data::isolate_rs(instr) as usize).unwrap();
  #[allow(clippy::unwrap_used)]
  let rt = reg.r(data::isolate_rt(instr) as usize).unwrap();

  (rd, rs, rt)
}

/// Parses instructions in format `i rt, rs, imm16`
fn parse_arithm_i(instr: u32, reg: &Registers) -> (RefMut<u32>, RefMut<u32>, u16) {
  // data::isolate_r* cannot return values higher or equal to 32
  #[allow(clippy::unwrap_used)]
  let rt = reg.r(data::isolate_rt(instr) as usize).unwrap();
  #[allow(clippy::unwrap_used)]
  let rs = reg.r(data::isolate_rs(instr) as usize).unwrap();
  let imm16 = data::isolate_imm16(instr);

  (rt, rs, imm16)
}

/// Perform the next cycle (as pointed by the current program counter). This
/// function does NOT write to the program counter, the caller is responsible
/// for updating the PC depending on the cycle result.
pub fn perform_cycle(memory: &mut MemoryMap, registers: &mut Registers) -> Next {
  let instr = expect!(memory.load_word(registers.pc));

  // instruction flow: according to this documentation
  // https://www.math.unipd.it/~sperduti/ARCHITETTURE-1/mips32.pdf

  let opcode = data::isolate_opcode(instr);

  match opcode {
    0 => handle_zero_opcode(instr, memory, registers),

    0x8 => {
      // addi rt, rs, imm16
      let (mut rt, rs, imm16) = parse_arithm_i(instr, registers);

      let addend0 = *rs;
      let addend1 = data::sign_extend(imm16);
      let sum = addend0 + addend1;

      if data::twos_complement_overflowed(addend0, addend1, sum) {
        return Next::Exception(Exception::Overflow);
      }

      *rt = sum;

      Next::Forward
    }

    0x9 => {
      // addiu rt, rs, imm16
      let (mut rt, rs, imm16) = parse_arithm_i(instr, registers);

      *rt = *rs + data::sign_extend(imm16);
      Next::Forward
    }

    0xa => {
      // slti rt, rs, imm16
      let (mut rt, rs, imm16) = parse_arithm_i(instr, registers);

      *rt = (*rs - data::sign_extend(imm16)) >> 31;
      Next::Forward
    }

    0xb => {
      // sltiu rt, rs, imm16
      let (mut rt, rs, imm16) = parse_arithm_i(instr, registers);

      *rt = (*rs < data::sign_extend(imm16)) as u32;
      Next::Forward
    }

    0xc => {
      // andi rt, rs, imm16
      let (mut rt, rs, imm16) = parse_arithm_i(instr, registers);

      *rt = *rs & imm16 as u32;
      Next::Forward
    }

    0xd => {
      // ori rt, rs, imm16
      let (mut rt, rs, imm16) = parse_arithm_i(instr, registers);

      *rt = *rs | imm16 as u32;
      Next::Forward
    }

    0xe => {
      // xori rt, rs, imm16
      let (mut rt, rs, imm16) = parse_arithm_i(instr, registers);

      *rt = *rs ^ imm16 as u32;
      Next::Forward
    }

    0xf => {
      // lui rt, imm16
      let hword = data::isolate_imm16(instr);
      // we know data::isolate_rt cannot return >= 32
      #[allow(clippy::unwrap_used)]
      let mut rt = registers.r(data::isolate_rt(instr) as usize).unwrap();
      *rt = (hword as u32) << 16;
      Next::Forward
    }

    _ => unimplemented!(),
  }
}

fn handle_zero_opcode(
  instr: u32,
  _memory: &mut MemoryMap,
  registers: &mut Registers,
) -> Next {
  let funct = data::isolate_funct(instr);

  match funct {
    0x0 => {
      // sll rd, rt, shamt
      let (mut rd, _, rt) = parse_arithm_r(instr, registers);
      let shamt = data::isolate_shamt(instr);

      *rd = *rt << shamt;
    }

    0x4 => {
      // sllv rd, rt, rs
      let (mut rd, rs, rt) = parse_arithm_r(instr, registers);

      *rd = *rt << *rs;
    }

    0x3 => {
      // sra rd, rt, shamt
      let (mut rd, _, rt) = parse_arithm_r(instr, registers);
      let shamt = data::isolate_shamt(instr);

      *rd = (*rt >> shamt) | (*rt & (1 << 31))
    }

    0x19 => {
      // multu
      let (_, rs, rt) = parse_arithm_r(instr, registers);

      let (lo, hi) = u32::widening_mul(*rs, *rt);
      drop((rs, rt));
      registers.hi = hi;
      registers.lo = lo;
    }
    0x20 => {
      // add
      let (mut rd, rs, rt) = parse_arithm_r(instr, registers);
      let result = u32::wrapping_add(*rs, *rt);

      if data::twos_complement_overflowed(*rs, *rt, result) {
        return Next::Exception(Exception::Overflow);
      }

      *rd = result;
    }

    0x21 => {
      // addu
      let (mut rd, rs, rt) = parse_arithm_r(instr, registers);
      *rd = u32::wrapping_add(*rs, *rt);
    }

    0x22 => {
      // sub
      let (mut rd, rs, rt) = parse_arithm_r(instr, registers);
      let result = u32::wrapping_sub(*rs, *rt);

      if data::twos_complement_overflowed(*rs, *rt, result) {
        return Next::Exception(Exception::Overflow);
      }

      *rd = result;
    }

    0x23 => {
      // subu
      let (mut rd, rs, rt) = parse_arithm_r(instr, registers);
      *rd = u32::wrapping_sub(*rs, *rt);
    }

    0x24 => {
      // and
      let (mut rd, rs, rt) = parse_arithm_r(instr, registers);
      *rd = *rs & *rt;
    }

    0x25 => {
      // or
      let (mut rd, rs, rt) = parse_arithm_r(instr, registers);
      *rd = *rs | *rt;
    }

    0x26 => {
      // xor
      let (mut rd, rs, rt) = parse_arithm_r(instr, registers);
      *rd = *rs ^ *rt;
    }

    0x27 => {
      // nor
      let (mut rd, rs, rt) = parse_arithm_r(instr, registers);
      *rd = !(*rs | *rt);
    }

    _ => todo!(),
  }

  Next::Forward
}
