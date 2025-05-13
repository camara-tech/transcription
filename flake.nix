{
  description = "Rust transcription project with reproducible dev environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
      let
        system = "x86_64-linux";
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" ];
        };
      in {
        devShell."${system}" = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            pkgs.pkg-config
          ];
          shellHook = ''
          export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library";
          '';
        };
      };
}
