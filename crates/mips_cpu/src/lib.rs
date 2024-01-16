#![feature(bigint_helper_methods)]

use cycle::Next;
use std::fmt;
use std::rc::Rc;

/// MIPS bytecote interpreter which runs one program, then dies.
pub struct Cpu {
  memory: mem::MemoryMap,
  registers: register::Registers,
  _source_object: Rc<mips_object::Object>,
}

impl Cpu {
  /// Prepare a runnable program instance, map data onto CPU memory
  pub fn new(obj: Rc<mips_object::Object>) -> Cpu {
    let registers = register::Registers::init();

    Cpu {
      memory: mem::MemoryMap::from_object(Rc::clone(&obj)),
      _source_object: obj,
      registers,
    }
  }

  /// Run one CPU cycle
  pub fn cycle(&mut self) {
    let result = cycle::perform_cycle(&mut self.memory, &mut self.registers);

    match result {
      Next::Forward => {
        self.registers.pc += 4;
      }

      Next::Branch(value) => {
        self.registers.pc = value;
      }

      Next::Exception(_excpt) => {
        todo!("exception handling");
      }

      Next::VmError(reason) => {
        panic!("internal VM error: {reason}");
      }
    }
  }
}

impl fmt::Debug for Cpu {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "PC: {:#010x} ({})", self.registers.pc, self.registers.pc)?;
    writeln!(f, "HI: {:#010x} ({})", self.registers.hi, self.registers.hi)?;
    writeln!(f, "LO: {:#010x} ({})", self.registers.lo, self.registers.lo)?;

    for (i, value_result) in self.registers.regular_values().iter().enumerate() {
      let value = match value_result {
        Ok(value_ref) => format!("{:#010x}", **value_ref),
        Err(_) => "*unsynced*".to_owned(),
      };

      writeln!(f, "r{i}: {value}")?
    }

    write!(f, "")
  }
}

pub mod cycle;
pub mod mem;
pub mod register;
pub mod exception;
