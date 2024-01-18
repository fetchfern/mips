#![allow(non_snake_case)]
use dioxus::prelude::*;
use mips_cpu::Cpu;
use mips_program::ProgramData;
use std::rc::Rc;

fn main() {
  dioxus_desktop::launch_cfg(
    App,
    dioxus_desktop::Config::new()
      .with_custom_head(r#"<link rel="stylesheet" href="public/tailwind.css">"#.to_owned()),
  )
}

fn App(cx: Scope) -> Element {
  let text = 0b0000_0001_0000_1001_0101_0000_0010_0001_u32
    .to_le_bytes()
    .to_vec();

  let program = ProgramData::builder().text(text).build();

  let mut cpu = Cpu::new(Rc::new(program));
  cpu.cycle();

  let registers = format!("{cpu:#?}");
  let lines = registers.lines();

  cx.render(rsx!(
    div {
      for line in lines {
        p {
          class: "text-xl",
          "{line}"
        }
      }
    }
  ))
}
