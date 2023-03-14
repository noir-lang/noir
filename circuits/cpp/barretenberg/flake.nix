{
  description =
    "Barretenberg: C++ cryptographic library, BN254 elliptic curve library, and PLONK SNARK prover";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    let
      barretenbergOverlay = self: super: {
        # It seems that llvmPackages_11 can't build WASI, so default to llvmPackages_12
        barretenberg = super.callPackage ./barretenberg.nix {
          llvmPackages = self.llvmPackages_12;
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
                value = pkgs.pkgsCross.aarch64-darwin.barretenberg.override {
                  # llvmPackages_12 seems to fail when we try to cross-compile but llvmPackages_11 works
                  llvmPackages = pkgs.llvmPackages_11;
                };
              }
            );

          shellDefaults = {
            nativeBuildInputs = [
              pkgs.starship
              pkgs.llvmPackages_12.llvm
            ];

            shellHook = ''
              eval "$(starship init bash)"
            '';
          };
        in
        rec {
          packages = {
            llvm11 = pkgs.barretenberg.override {
              llvmPackages = pkgs.llvmPackages_11;
            };
            llvm12 = pkgs.barretenberg;
            llvm13 = pkgs.barretenberg.override {
              llvmPackages = pkgs.llvmPackages_13;
            };
            llvm14 = pkgs.barretenberg.override {
              llvmPackages = pkgs.llvmPackages_14;
            };
            wasm32 = pkgs.pkgsCross.wasi32.barretenberg;

            # Defaulting to llvm12 so we can ensure consistent shells
            default = packages.llvm12;
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
