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
    in rec {
      devShells.default = pkgs.mkShell {
        buildInputs = with pkgs; [cargo cargo-watch rustfmt clippy ];
      };

      # packages.lss = pkgs.rustPlatform.buildRustPackage {
      #   pname = "lss";
      #   version = "1.2.1";
      #   src = ./.;
      #   cargoLock = {
      #     lockFile = ./Cargo.lock;
      #     allowBuiltinFetchGit = true;
      #   };
      # };

      # defaultPackage = packages.lss;

      formatter = pkgs.nixpkgs-fmt;
    });
}
