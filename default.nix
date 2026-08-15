{ rustPlatform, ... }:

let
  # Parse Cargo.toml so we can grab values from it
  cargo-toml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
in
rustPlatform.buildRustPackage rec {
  pname = cargo-toml.package.name;
  version = cargo-toml.package.version;
  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;
}
