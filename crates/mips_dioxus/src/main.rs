#![allow(non_snake_case)]
use dioxus::prelude::*;
use mips_cpu::Cpu;

fn main() {
  // launch the dioxus app in a webview
  dioxus_desktop::launch_cfg(
    App,
    dioxus_desktop::Config::new()
      .with_custom_head(r#"<link rel="stylesheet" href="public/tailwind.css">"#.to_string()),
  )
}

// define a component that renders a div with the text "Hello, world!"
fn App(cx: Scope) -> Element {
  let prog = 0b0000_0001_0000_1001_0101_0000_0010_0001_u32.to_le_bytes();

  let mut cpu = Cpu::new(&prog);

  cpu.next();

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
