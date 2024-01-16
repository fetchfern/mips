use crate::exception::Exception;
use std::rc::Rc;

pub const TEXT_START: u32 = 0x00400000;
pub const EXTERN_START: u32 = 0x10000000;
pub const TEXT_END: u32 = EXTERN_START - 1;
// const HEAP_START: u32 = 0x10040000;
// const EXTERN_END: u32 = HEAP_START - 1;
// const KTEXT_START: u32 = 0x80000000;
// const HEAP_END: u32 = KTEXT_START - 1;
// const KDATA_START: u32 = 0x90000000;
// const KTEXT_END: u32 = KDATA_START - 1;

pub struct MemoryMap {
  source_object: Rc<mips_object::Object>,
}

impl MemoryMap {
  pub fn from_object(source_object: Rc<mips_object::Object>) -> MemoryMap {
    MemoryMap { source_object }
  }

  pub fn load_word(&mut self, addr: u32) -> Result<u32, Exception> {
    match addr {
      TEXT_START..=TEXT_END => {
        // .text (program instructions)
        let relative = (addr - TEXT_START) as usize;

        if let Some(bytes) = self.source_object.text.raw_data.get(relative..relative + 4) {
          // prior .get(n..n+4) ensuring `bytes` being 4 bytes long
          #[allow(clippy::unwrap_used)]
          Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
        } else {
          Err(Exception::AddrLoadFetch)
        }
      }

      addr => todo!("mem fetch @ {addr:#10x}"),
    }
  }
}
