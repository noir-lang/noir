{
  description = "Build Nargo";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };

    libbarretenberg_flake = {
      url = "github:kobyhallx/aztec-connect/kh-ndsl-w-flake";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self, nixpkgs, crane, flake-utils, rust-overlay, libbarretenberg_flake, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };


        craneLib = (crane.mkLib pkgs).overrideScope' (final: prev: {
          stdenv = pkgs.llvmPackages.stdenv;
        });

        # TODO: line below looks terrible we can do better naming here in referenced flake
        libbarretenberg = libbarretenberg_flake.packages.${system}.default;

        commonArgs = {
          src = craneLib.cleanCargoSource ./.;

          doCheck = false;

          cargoCheckCommand = "true";
          cargoBuildCommand = "cargo build --release";

        };

        GIT_COMMIT = pkgs.lib.optionalString (self ? rev) self.rev;
        GIT_DIRTY = "false";

        nargo = craneLib.buildPackage ({
          pname = "nargo";
          src = craneLib.cleanCargoSource ./.;

          doCheck = false;

          inherit GIT_COMMIT;
          inherit GIT_DIRTY;
          
          # Bindegn needs these
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          BINDGEN_EXTRA_CLANG_ARGS = "-I${libbarretenberg}/include/aztec -L${libbarretenberg}";
          RUSTFLAGS = "-L${libbarretenberg}/lib -lomp";

          buildInputs = [
            pkgs.llvmPackages.openmp
            libbarretenberg
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];
        } // commonArgs);

      in rec {
        checks = { inherit nargo; };

        packages.default = nargo;

        # apps.default = flake-utils.lib.mkApp {
        #   drv = pkgs.writeShellScriptBin "barretenberg_wrapper" ''
        #     ${my-crate}/bin/barretenberg_wrapper
        #   '';
        # };

        apps.default = flake-utils.lib.mkApp { drv = nargo; };

        devShells.default = pkgs.mkShell {
          inputsFrom = builtins.attrValues self.checks;

          buildInputs = packages.default.buildInputs ;

          BINDGEN_EXTRA_CLANG_ARGS = "-I${libbarretenberg}/include/aztec -L${libbarretenberg}";

          nativeBuildInputs = with pkgs; [
            cargo
            rustc ];
        };
      });
}
