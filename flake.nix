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
          inherit (pkgs) lib;

          rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

          sqlFilter = path: _type: null != builtins.match ".*sql$" path;
          sqlOrCargo = path: type: (sqlFilter path type) || (craneLib.filterCargoSources path type);

          src = lib.cleanSourceWith {
            src = craneLib.path ./.; # The original, unfiltered source
            filter = sqlOrCargo;
          };
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

          dockerImage = pkgs.dockerTools.buildImage {
            name="uc_blog";
            tag="latest";
            copyToRoot = [ bin ];
            config = {
              Cmd = [ "${bin}/bin/uc_blog"];
              Volumes = {"./db/articles.sqlite3" = {};};
            };
          };

        in
        with pkgs;
        {
          packages = {
            inherit bin dockerImage;
            default = bin;
          };
          defaultPackage = dockerImage;
          devShells.default = craneLib.devShell {
            inputsFrom = [ bin ];
            packages = [
                pkgs.sqlx-cli
            ];


            shellHook = ''
              export DATABASE_URL=sqlite:./db/articles.sqlite3
              sqlx database create
              sqlx migrate run

              echo "Hey! <3"
              echo "this is development shell for my blog!"
            '';
          };
        }
      );
}
