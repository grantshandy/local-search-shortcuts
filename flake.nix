{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    utils,
    ...
  }:
    utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      update-bang = pkgs.writeShellScriptBin "update-resources" ''
        ${pkgs.curl}/bin/curl https://duckduckgo.com/bang.js -o ./res/bang.json
      '';
      download-htmx = pkgs.writeShellScriptBin "download-htmx" ''
        ${pkgs.curl}/bin/curl https://unpkg.com/htmx.org@2.0.4/dist/htmx.min.js -o ./res/htmx.min.js
      '';
    in {
      devShells.default = pkgs.mkShell {
        RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

        buildInputs = with pkgs; [
          cargo
          cargo-watch
          rust-analyzer
          rustfmt
          clippy
          update-bang
          download-htmx
        ];
      };
    });
}
