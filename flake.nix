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
      url = "git+https://github.com/AztecProtocol/barretenberg?ref=phated/nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self, nixpkgs, crane, flake-utils, rust-overlay, libbarretenberg_flake, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            rust-overlay.overlays.default
            libbarretenberg_flake.overlays.default
          ];
        };

        rustToolchain = pkgs.rust-bin.stable."1.66.0".default;

        craneLibScope = (crane.mkLib pkgs).overrideScope' (final: prev: {
          # As per https://discourse.nixos.org/t/gcc11stdenv-and-clang/17734/7
          stdenv = with pkgs;
            if (stdenv.targetPlatform.isGnu && stdenv.targetPlatform.isAarch64) then
              overrideCC llvmPackages.stdenv (llvmPackages.clang.override { gccForLibs = gcc11.cc; })
            else
              llvmPackages.stdenv;
        });

        craneLib = craneLibScope.overrideToolchain rustToolchain;

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
          BINDGEN_EXTRA_CLANG_ARGS = "-I${libbarretenberg_flake}/include/barretenberg -L${libbarretenberg_flake}";
          # RUSTFLAGS = "-L${libbarretenberg}/lib -lomp";

          nativeBuildInputs = [
            pkgs.pkg-config
          ];

          buildInputs = [
            pkgs.llvmPackages.openmp
            pkgs.barretenberg
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

          BINDGEN_EXTRA_CLANG_ARGS = "-I${libbarretenberg_flake}/include/barretenberg -L${libbarretenberg_flake}";

          nativeBuildInputs = with pkgs; [
            cargo
            rustc ];
        };
      });
}
