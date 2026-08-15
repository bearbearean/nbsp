{ pkgs, ... }:

let
  # Use Rustup to get our wanted Rust version from the toolchain file
  rustup-toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
in
pkgs.mkShell rec {
  packages = with pkgs; [
    rustup-toolchain
    cargo-make
    release-plz
    gh
  ];
}
