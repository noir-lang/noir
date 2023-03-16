{
  description = "Nargo";

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

    barretenberg = {
      url = "git+https://github.com/AztecProtocol/barretenberg";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self, nixpkgs, crane, flake-utils, rust-overlay, barretenberg, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            rust-overlay.overlays.default
            barretenberg.overlays.default
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
          src = ./.;

          doCheck = false;

          cargoCheckCommand = "true";
          cargoBuildCommand = "cargo build --release";

        };

        src = pkgs.copyPathToStore ./.;

        # This is a problem for now
        GIT_COMMIT = if (self ? rev) then self.rev else "unknown";
        GIT_DIRTY = "false";

        nargo = craneLib.buildPackage ({
          pname = "nargo";

          doCheck = false;

          inherit GIT_COMMIT;
          inherit GIT_DIRTY;
          
          # Bindegn needs these
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

          nativeBuildInputs = [
            pkgs.pkg-config
            pkgs.llvmPackages.bintools
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

        apps.default = flake-utils.lib.mkApp { drv = nargo; };

        devShells.default = pkgs.mkShell.override { stdenv = pkgs.llvmPackages.stdenv; } {
          inputsFrom = builtins.attrValues self.checks;

          buildInputs = packages.default.buildInputs ;

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          # Uncertain if below line is needed. dev Shell is not yet fully working
          # BINDGEN_EXTRA_CLANG_ARGS = "-I${pkgs.barretenberg}/include -isystem ${pkgs.llvmPackages.libcxx.dev}/include";

          nativeBuildInputs = with pkgs; [
            which
            starship
            git
            cargo
            rustc
            pkg-config            
          ];

          shellHook = ''
            eval "$(starship init bash)"
          '';

        };
      });
}
