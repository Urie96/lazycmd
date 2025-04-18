{
  nixpkgs ? <nixpkgs>,
  rust-overlay ? (
    import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz")
  ),
}:
let
  pkgs = import nixpkgs {
    config = { };
    overlays = [
      rust-overlay
      (final: prev: {
        rustToolchain =
          let
            rust = prev.rust-bin;
          in
          if builtins.pathExists ./rust-toolchain.toml then
            rust.fromRustupToolchainFile ./rust-toolchain.toml
          else if builtins.pathExists ./rust-toolchain then
            rust.fromRustupToolchainFile ./rust-toolchain
          else
            rust.stable.latest.default.override {
              extensions = [
                "rust-src"
                "rustfmt"
              ];
            };
      })
    ];
  };
in
pkgs.mkShell {
  packages = with pkgs; [
    rustToolchain
    openssl
    pkg-config
    cargo-deny
    cargo-edit
    cargo-watch
    rust-analyzer
    lua5_4_compat
  ];

  env = {
    # Required by rust-analyzer
    RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
  };
}
