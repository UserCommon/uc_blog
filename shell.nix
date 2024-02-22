{ pkgs ? import <nixpkgs> {} }:

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
        Hello!
    '';
}
