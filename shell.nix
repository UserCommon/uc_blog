{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustup
    cargo
    clippy
    rust-analyzer
    sqlx-cli
    docker
    lld
    openssl
  ];

  shellHook = ''
    echo "<3"
    set -a
    source ./articles/.env
    source ./proxy/.env
    set +a
    export RUST_LOG=debug

    alias run="cargo watch -x check -x test -x run"
  '';
}
