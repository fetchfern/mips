use crate::exception::Exception;
use mips_program::{Context, ProgramData, Section};
use std::rc::Rc;

/// Start of `.text`.
///
/// The `.text` section contains user code to be executed.
pub const TEXT_START: u32 = 0x00400000;
/// End of `.text`, inclusive.
pub const TEXT_END: u32 = EXTERN_START - 1;
/// Start of `.extern`.
///
/// The `.extern` section is reserved for global variables. It starts at
/// memory address `0x10000000`, which is normally the start of the `.data`
/// section. However, like the MARS simulator, the first `2 ^ 16` bytes of
/// said section are reserved for global variables, so the `.data` section
/// starts at `0x10010000` by default.
pub const EXTERN_START: u32 = 0x10000000;
/// End of `.extern`, inclusive.
pub const EXTERN_END: u32 = DATA_START - 1;
/// Start of `.data`.  
///
/// The `.data` section stores static program data. Note that it isn't
/// immutable, think of it like a `static mut` like Rust.
pub const DATA_START: u32 = 0x10010000;
/// End of `.data`, inclusive.
pub const DATA_END: u32 = HEAP_START - 1;
/// Start of `.heap`.
///
/// The `.heap` section contains data allocated though the `sbrk` syscall.
pub const HEAP_START: u32 = 0x10040000;
/// End of `.heap`, inclusive.
pub const HEAP_END: u32 = KTEXT_START - 1;
/// Start of `.ktext`.
///
/// The `.ktext` section contains kernel code, like the exception handler.
pub const KTEXT_START: u32 = 0x80000000;
/// End of `.ktext`, inclusive.
pub const KTEXT_END: u32 = KDATA_START - 1;
/// Start of `.kdata`.
///
/// The `.kdata` section contains kernel static data.
pub const KDATA_START: u32 = 0x90000000;

pub struct MemoryMap {
  program: Rc<ProgramData>,
}

impl MemoryMap {
  pub fn from_program(program: Rc<ProgramData>) -> MemoryMap {
    MemoryMap { program }
  }

  pub fn load_word(&mut self, addr: u32) -> Result<u32, Exception> {
    let data_maybe = match addr {
      TEXT_START..=TEXT_END => {
        // .text (program instructions)
        self
          .program
          .read(Section::Text, Context::User)
          .ok_or(Exception::AddrLoadFetch)?
          .read_word((addr - TEXT_START) as usize)
      }

      EXTERN_START..=EXTERN_END => self
        .program
        .read(Section::Extern, Context::User)
        .ok_or(Exception::AddrLoadFetch)?
        .read_word((addr - EXTERN_START) as usize),

      DATA_START..=DATA_END => self
        .program
        .read(Section::Data, Context::User)
        .ok_or(Exception::AddrLoadFetch)?
        .read_word((addr - DATA_START) as usize),

      addr => todo!("mem fetch @ {addr:#10x}"),
    };

    data_maybe.ok_or(Exception::AddrLoadFetch)
  }
}
