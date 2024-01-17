pub fn isolate_opcode(instr: u32) -> u32 {
  (instr >> 26) & ((1 << 6) - 1)
}

pub fn isolate_funct(instr: u32) -> u32 {
  instr & ((1 << 6) - 1)
}

pub fn isolate_rs(instr: u32) -> u32 {
  (instr >> 21) & ((1 << 5) - 1)
}

pub fn isolate_rt(instr: u32) -> u32 {
  (instr >> 16) & ((1 << 5) - 1)
}

pub fn isolate_rd(instr: u32) -> u32 {
  (instr >> 11) & ((1 << 5) - 1)
}

pub fn isolate_shamt(instr: u32) -> u32 {
  (instr >> 6) & ((1 << 5) - 1)
}

pub fn isolate_imm16(instr: u32) -> u16 {
  (instr & ((1 << 16) - 1)) as u16
}

pub fn isolate_target_26(instr: u32) -> u32 {
  instr & ((1 << 26) - 1)
}

pub fn twos_complement_overflowed(n1: u32, n2: u32, r: u32) -> bool {
  // last_bit(n1) SAME AS last_bit(n2) AND last_bit(n1) DIFFERENT last_bit(result)
  // (last_bit(n1) XOR last_bit(n2) == 0) AND (last_bit(n1) XOR last_bit(result) == 1)
  (n1 ^ n2) < 0x80000000 && (n1 ^ r) > 0x7fffffff
}

pub fn sign_extend(n: u16) -> u32 {
  let n = n as u32;
  let ext = 0xffffu32 * (n >> 15);
  n | ext << 16
}
