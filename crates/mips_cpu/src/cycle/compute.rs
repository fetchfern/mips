use crate::cycle::{data, Next};
use crate::exception::Exception;
use crate::mem::MemoryMap;
use crate::register::Registers;
use std::cell::RefMut;

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

/// Parses instructions in format `i rs, rt`
fn parse_trap_r(instr: u32, reg: &Registers) -> (RefMut<u32>, RefMut<u32>) {
  // data::isolate_r* cannot return values higher or equal to 32
  #[allow(clippy::unwrap_used)]
  let rs = reg.r(data::isolate_rs(instr) as usize).unwrap();
  #[allow(clippy::unwrap_used)]
  let rt = reg.r(data::isolate_rt(instr) as usize).unwrap();

  (rs, rt)
}

/// Perform the next cycle (as pointed by the current program counter). This
/// function does NOT write to the program counter, the caller is responsible
/// for updating the PC depending on the cycle result.
pub fn perform_cycle(memory: &mut MemoryMap, registers: &mut Registers) -> Next {
  let instr = match memory.load_word(registers.pc) {
    Ok(v) => v,
    Err(e) => return Next::Exception(e),
  };

  // instruction flow: according to this documentation
  // https://www.math.unipd.it/~sperduti/ARCHITETTURE-1/mips32.pdf

  let opcode = data::isolate_opcode(instr);

  match opcode {
    0x0 => handle_opcode_zero(instr, memory, registers),
    0x1 => handle_opcode_one(instr, memory, registers),

    0x2 => {
      // j target
      let target = data::isolate_target_26(instr);
      Next::Branch(target)
    }

    0x3 => {
      // jal target
      let target = data::isolate_target_26(instr);

      // unwrap is OK the value is a known constant
      #[allow(clippy::unwrap_used)]
      registers.link(31).unwrap();

      Next::Branch(target)
    }

    0x4 => {
      // beq rs, rt, offset
      let (rt, rs, offset) = parse_arithm_i(instr, registers);
      let addr = data::add_ihalf_to_uword(registers.pc, offset);

      if *rt == *rs {
        Next::Branch(addr)
      } else {
        Next::Forward
      }
    }

    0x5 => {
      // bne rs, rt, offset
      let (rt, rs, offset) = parse_arithm_i(instr, registers);
      let addr = data::add_ihalf_to_uword(registers.pc, offset);

      if *rt != *rs {
        Next::Branch(addr)
      } else {
        Next::Forward
      }
    }

    0x6 => {
      // blez rs, offset

      let (_, rs, offset) = parse_arithm_i(instr, registers);
      let addr = data::add_ihalf_to_uword(registers.pc, offset);

      // lez signed comparison
      if *rs == 0 || *rs >= (1 << 31) {
        Next::Branch(addr)
      } else {
        Next::Forward
      }
    }

    0x7 => {
      // bgtz rs, offset

      let (_, rs, offset) = parse_arithm_i(instr, registers);
      let addr = data::add_ihalf_to_uword(registers.pc, offset);

      // gtz signed comparison
      if (1..1 << 31).contains(&*rs) {
        Next::Branch(addr)
      } else {
        Next::Forward
      }
    }

    0x8 => {
      // addi rt, rs, imm16
      let (mut rt, rs, imm16) = parse_arithm_i(instr, registers);

      let addend0 = *rs;
      let addend1 = data::sign_extend(16, imm16 as u32);
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

      *rt = *rs + data::sign_extend(16, imm16 as u32);
      Next::Forward
    }

    0xa => {
      // slti rt, rs, imm16
      let (mut rt, rs, imm16) = parse_arithm_i(instr, registers);

      *rt = (*rs - data::sign_extend(16, imm16 as u32)) >> 31;
      Next::Forward
    }

    0xb => {
      // sltiu rt, rs, imm16
      let (mut rt, rs, imm16) = parse_arithm_i(instr, registers);

      *rt = (*rs < data::sign_extend(16, imm16 as u32)) as u32;
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

    0x20 => {
      // lb rt, offset(rs)
      let (mut rt, rs, offset) = parse_arithm_i(instr, registers);
      let addr = data::add_ihalf_to_uword(*rs, offset);

      match memory.load_byte(addr) {
        Ok(b) => {
          *rt = data::sign_extend(8, b as u32);
          Next::Forward
        }
        Err(e) => Next::Exception(e),
      }
    }

    0x21 => {
      // lh rt, offset(rs)
      let (mut rt, rs, offset) = parse_arithm_i(instr, registers);
      let addr = data::add_ihalf_to_uword(*rs, offset);

      match memory.load_halfword(addr) {
        Ok(h) => {
          *rt = data::sign_extend(16, h as u32);
          Next::Forward
        }
        Err(e) => Next::Exception(e),
      }
    }

    0x23 => {
      // lw rt, offset(rs)
      let (mut rt, rs, offset) = parse_arithm_i(instr, registers);
      let addr = data::add_ihalf_to_uword(*rs, offset);

      match memory.load_word(addr) {
        Ok(w) => {
          *rt = w;
          Next::Forward
        }
        Err(e) => Next::Exception(e),
      }
    }

    0x24 => {
      // lbu rt, offset(rs)
      let (mut rt, rs, offset) = parse_arithm_i(instr, registers);
      let addr = data::add_ihalf_to_uword(*rs, offset);

      match memory.load_byte(addr) {
        Ok(b) => {
          *rt = b as u32;
          Next::Forward
        }
        Err(e) => Next::Exception(e),
      }
    }

    0x25 => {
      // lhu rt, offset(rs)
      let (mut rt, rs, offset) = parse_arithm_i(instr, registers);
      let addr = data::add_ihalf_to_uword(*rs, offset);

      match memory.load_halfword(addr) {
        Ok(h) => {
          *rt = h as u32;
          Next::Forward
        }
        Err(e) => Next::Exception(e),
      }
    }

    _ => unimplemented!(),
  }
}

fn handle_opcode_zero(instr: u32, _memory: &mut MemoryMap, registers: &mut Registers) -> Next {
  let funct = data::isolate_funct(instr);

  match funct {
    0x0 => {
      // sll rd, rt, shamt
      let (mut rd, _, rt) = parse_arithm_r(instr, registers);
      let shamt = data::isolate_shamt(instr);

      *rd = *rt << shamt;
    }

    0x3 => {
      // sra rd, rt, shamt
      let (mut rd, _, rt) = parse_arithm_r(instr, registers);
      let shamt = data::isolate_shamt(instr);

      *rd = (*rt >> shamt) | (*rt & (1 << 31))
    }

    0x4 => {
      // sllv rd, rt, rs
      let (mut rd, rs, rt) = parse_arithm_r(instr, registers);

      *rd = *rt << *rs;
    }

    0x8 => {
      // jr rs
      #[allow(clippy::unwrap_used)]
      let rs = registers.r(data::isolate_rs(instr) as usize).unwrap();

      return Next::Branch(*rs);
    }

    0x9 => {
      // jalr rs, rd
      #[allow(clippy::unwrap_used)]
      let rs = registers.r(data::isolate_rs(instr) as usize).unwrap();

      #[allow(clippy::unwrap_used)]
      registers.link(data::isolate_rd(instr) as usize).unwrap();

      return Next::Branch(*rs);
    }

    0xa => {
      // movz rd, rs, rt
      let (mut rd, rs, rt) = parse_arithm_r(instr, registers);

      if *rt == 0 {
        *rd = *rs;
      }
    }

    0xb => {
      // movz rd, rs, rt
      let (mut rd, rs, rt) = parse_arithm_r(instr, registers);

      if *rt != 0 {
        *rd = *rs;
      }
    }

    0x10 => {
      // mfhi rd
      #[allow(clippy::unwrap_used)]
      let mut rd = registers.r(data::isolate_rd(instr) as usize).unwrap();

      *rd = registers.hi;
    }

    0x11 => {
      // mthi rs
      #[allow(clippy::unwrap_used)]
      let rs_value = *registers.r(data::isolate_rs(instr) as usize).unwrap();
      registers.hi = rs_value;
    }

    0x12 => {
      // mflo rd
      #[allow(clippy::unwrap_used)]
      let mut rd = registers.r(data::isolate_rd(instr) as usize).unwrap();

      *rd = registers.lo;
    }

    0x13 => {
      // mtlo rs
      #[allow(clippy::unwrap_used)]
      let rs_value = *registers.r(data::isolate_rs(instr) as usize).unwrap();
      registers.lo = rs_value;
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

    0x31 => {
      // tgeu rs, rt
      let (rs, rt) = parse_trap_r(instr, registers);

      if *rs >= *rt {
        return Next::Exception(Exception::Trap);
      } else {
        return Next::Forward;
      }
    }

    0x33 => {
      // tltu rs, rt
      let (rs, rt) = parse_trap_r(instr, registers);

      if *rs < *rt {
        return Next::Exception(Exception::Trap);
      } else {
        return Next::Forward;
      }
    }

    0x34 => {
      // teq rs, rt
      let (rs, rt) = parse_trap_r(instr, registers);

      if *rs == *rt {
        return Next::Exception(Exception::Trap);
      } else {
        return Next::Forward;
      }
    }

    0x36 => {
      // tneq rs, rt
      let (rs, rt) = parse_trap_r(instr, registers);

      if *rs != *rt {
        return Next::Exception(Exception::Trap);
      } else {
        return Next::Forward;
      }
    }

    _ => todo!(),
  }

  Next::Forward
}

fn handle_opcode_one(instr: u32, _memory: &mut MemoryMap, registers: &mut Registers) -> Next {
  let (rt, rs, imm16) = parse_arithm_i(instr, registers);

  match *rt {
    0x0 => {
      // bltz rs, offset

      // signed  ltz comparison
      if *rs >= (1 << 31) {
        Next::Branch(data::add_ihalf_to_uword(registers.pc, imm16))
      } else {
        Next::Forward
      }
    }

    0x1 => {
      // bgez rs, offset

      // signed comparison
      if *rs < (1 << 31) {
        Next::Branch(data::add_ihalf_to_uword(registers.pc, imm16))
      } else {
        Next::Forward
      }
    }

    0x10 => {
      // bltzal rs, offset

      // signed  ltz comparison
      if *rs >= (1 << 31) {
        #[allow(clippy::unwrap_used)]
        registers.link(31).unwrap();

        Next::Branch(data::add_ihalf_to_uword(registers.pc, imm16))
      } else {
        Next::Forward
      }
    }

    0x11 => {
      // bgezal rs, offset

      // signed comparison
      if *rs < (1 << 31) {
        #[allow(clippy::unwrap_used)]
        registers.link(31).unwrap();

        Next::Branch(data::add_ihalf_to_uword(registers.pc, imm16))
      } else {
        Next::Forward
      }
    }

    _ => unimplemented!(),
  }
}
