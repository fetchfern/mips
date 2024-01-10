#![allow(non_snake_case)]
use dioxus::prelude::*;

fn main() {
  // launch the dioxus app in a webview
  dioxus_desktop::launch(App);
}

// define a component that renders a div with the text "Hello, world!"
fn App(cx: Scope) -> Element {
  let mut count = use_state(cx, || 0);

  cx.render(rsx!(
    h1 {
      text_align: "center",
      "High-Five counter: {count}"
    },
    button {
      onclick: move |_| count += 1,
      "Up high!"
    },
    button {
      onclick: move |_| count -= 1,
      "Down low!"
    },
  ))
}
