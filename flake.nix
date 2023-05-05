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
        in
        rec
        {
          packages = rec {
            default = swayws;
            swayws = craneLib.buildPackage {
              name = "swayws";
              src = craneLib.cleanCargoSource ./.;
              doCheck = false;
              cargoVendorDir = craneLib.vendorCargoDeps { cargoLock = ./Cargo.lock; };
            };
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
