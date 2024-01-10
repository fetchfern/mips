{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    rust-overlay.url = "github:oxalica/rust-overlay";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    utils,
    rust-overlay,
    treefmt-nix,
    ...
  }:
    utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system overlays;};
      overlays = [(import rust-overlay)];
    in
      with pkgs; {
        name = "mips-stuff";

        formatter.${system} = treefmt-nix.lib.mkWrapper pkgs {
          projectRootFile = "flake.nix";

          programs = {
            alejandra.enable = true;
            deadnix.enable = true;
            nixd.enable = true;
            rustfmt.enable = true;
            prettier.enable = true;
          };
        };

        devShell = mkShell {
          buildInputs =
            [
              bun
              gdk-pixbuf
              glib
              gobject-introspection
              gtk3
              gtk4
              nixd
              openssl
              pango
              webkitgtk_4_1
              (rust-bin.stable.latest.default.override {
                targets = ["wasm32-unknown-unknown"];
              })
            ]
            ++ lib.optionals pkgs.stdenv.isDarwin (with darwin; [
              apple_sdk.frameworks.AppKit
              apple_sdk.frameworks.Carbon
              apple_sdk.frameworks.Cocoa
              apple_sdk.frameworks.CoreFoundation
              apple_sdk.frameworks.IOKit
              apple_sdk.frameworks.WebKit
              apple_sdk.frameworks.Security
              apple_sdk.frameworks.DisplayServices
            ]);
          nativeBuildInputs = [pkg-config];
        };
      });
}
