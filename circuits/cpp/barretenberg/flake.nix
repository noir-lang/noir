{
  description =
    "Barretenberg: C++ cryptographic library, BN254 elliptic curve library, and PLONK SNARK prover";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    let
      barretenbergOverlay = final: prev: {
        barretenberg = prev.callPackage ./barretenberg.nix { };
        barretenberg-wasm = prev.callPackage ./barretenberg-wasm.nix { };
        barretenberg-transcript00 = prev.fetchurl {
          url = "http://aztec-ignition.s3.amazonaws.com/MAIN%20IGNITION/monomial/transcript00.dat";
          sha256 = "sha256-D5SzlCb1pX0aF3QmJPfTFwoy4Z1sXhbyAigUOdvkhpU=";
        };
      };
    in
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ barretenbergOverlay ];
          };

          optional = pkgs.lib.lists.optional;

          crossTargets = builtins.listToAttrs
            (
              [ ] ++ optional (pkgs.hostPlatform.isx86_64 && pkgs.hostPlatform.isLinux) {
                name = "cross-aarch64";
                value = pkgs.pkgsCross.aarch64-multiplatform-musl.pkgsLLVM.barretenberg;
              } ++ optional (pkgs.hostPlatform.isx86_64 && pkgs.hostPlatform.isDarwin) {
                name = "cross-aarch64";
                value = pkgs.pkgsCross.aarch64-darwin.barretenberg;
              }
            );

          shellDefaults = {
            nativeBuildInputs = [
              pkgs.starship
              pkgs.llvmPackages_15.llvm
            ];

            shellHook = ''
              eval "$(starship init bash)"
            '';
          };
        in
        rec {
          packages = {
            llvm15 = pkgs.barretenberg.override {
              llvmPackages = pkgs.llvmPackages_15;
            };
            llvm16 = pkgs.barretenberg.override {
              llvmPackages = pkgs.llvmPackages_16;
            };
            wasm32 = pkgs.barretenberg-wasm;

            default = packages.llvm15;
          } // crossTargets;

          # Provide legacyPackages with our overlay so we can run
          # > `nix build .#pkgsCross.aarch64-multiplatform.barretenberg`
          # Ref https://discourse.nixos.org/t/how-do-i-cross-compile-a-flake/12062/12
          legacyPackages = import nixpkgs {
            inherit system;
            overlays = [ barretenbergOverlay ];
            crossOverlays = [ barretenbergOverlay ];
          };

          devShells = {
            default = pkgs.mkShell.override { stdenv = packages.default.stdenv; }
              ({
                inputsFrom =
                  [ packages.default ];
              } // shellDefaults);

            wasm32 = pkgs.mkShell.override
              {
                # TODO: This derivations forces wasi-sdk 12 so the stdenv will have the wrong tools
                stdenv = packages.wasm32.stdenv;
              }
              ({
                inputsFrom = [ packages.wasm32 ];
              } // shellDefaults);
          };
        }) // {
      overlays.default = barretenbergOverlay;
    };
}
