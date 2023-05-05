{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, fenix, ... }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };

          fenixChannel = fenix.packages.${system}.stable;
          fenixToolchain = (fenixChannel.withComponents [
            "rustc"
            "cargo"
            "rustfmt"
            "clippy"
            "rust-analysis"
            "rust-src"
            "llvm-tools-preview"
          ]);
          craneLib = crane.lib.${system}.overrideToolchain fenixToolchain;

          commonArgs = {
            src = craneLib.cleanCargoSource ./.;
            doCheck = false;
            cargoVendorDir = craneLib.vendorCargoDeps { cargoLock = ./Cargo.lock; };
          };
          cargoArtifacts = craneLib.buildDepsOnly (commonArgs);
          swayws = craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; });
        in
        rec
        {
          packages = {
            default = swayws;
            swayws = swayws;
          };

          devShells.default = pkgs.mkShell {
            nativeBuildInputs = (with packages.swayws; nativeBuildInputs ++ buildInputs) ++ [ fenixToolchain ];
            RUST_SRC_PATH = "${fenixChannel.rust-src}/lib/rustlib/src/rust/library";
          };
        }) // {
      overlays.default = (final: prev: {
        inherit (self.packages.${prev.system})
          swayws;
      });
    };
}
