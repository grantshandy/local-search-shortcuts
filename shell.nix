{pkgs ? import <nixpkgs> {}, ...}: let
  curl = pkgs.lib.getExe pkgs.curl;
  update-res = pkgs.writeShellScriptBin "update-res" "${curl} https://duckduckgo.com/bang.js -o ./res/bang.json";
  download-htmx = pkgs.writeShellScriptBin "download-htmx" "${curl} https://unpkg.com/htmx.org@2.0.4/dist/htmx.min.js -o ./res/htmx.min.js";
in
  pkgs.mkShell {
    RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

    buildInputs = with pkgs; [
      cargo
      cargo-watch
      rustfmt
      clippy
      update-res
      download-htmx
    ];
  }
