{
  description = "A blog engine written in rust!";
  inputs = {
    nixpkgs.url = "github:NixOs/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        rust-overlay.follows = "rust-overlay";
        flake-utils.follows = "flake-utils";
      };
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
          src = craneLib.cleanCargoSource ./.;
          nativeBuildInputs = with pkgs;
            [ rustToolchain cargo-watch pkg-config sqlx-cli ];
          buildInputs = with pkgs; [ openssl sqlite ];
          commonArgs = {
            inherit src buildInputs nativeBuildInputs;
          };
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          bin = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;

            nativeBuildInputs = (commonArgs.nativeBuildInputs or [ ]) ++ [
              pkgs.sqlx-cli
            ];

            preBuild = ''
              export DATABASE_URL=sqlite:./db/articles.sqlite3
              sqlx database create
              sqlx migrate run
            '';
          });

        in
        with pkgs;
        {
          packages = {
            inherit bin;
            default = bin;
          };
          devShells.default = craneLib.devShell {
            inputsFrom = [ bin ];

            shellHook = ''
              export DATABASE_URL=sqlite:./db/articles.sqlite3

              echo "Hey! <3"
              echo "this is development shell for my blog!"
            '';
          };
        }
      );
}
