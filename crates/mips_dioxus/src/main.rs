#![allow(non_snake_case)]
use dioxus::prelude::*;
use mips_cpu::Cpu;

fn main() {
  // launch the dioxus app in a webview
  dioxus_desktop::launch(App);
}

// define a component that renders a div with the text "Hello, world!"
fn App(cx: Scope) -> Element {
  let prog = 0b000000010000100101010100001u32.to_le_bytes();

  let mut cpu = Cpu::new(&prog);

  cpu.next();

  cx.render(rsx!(
    div {
      h1 { "{cpu:?}" }
    }
  ))
}
