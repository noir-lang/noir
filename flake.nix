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

        environment = {
          # rust-bindgen needs to know the location of libclang
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

          # We set the environment variable because requiring 2 versions of bb collide when pkg-config searches for it
          BARRETENBERG_BIN_DIR = "${pkgs.pkgsCross.wasi32.barretenberg}/bin";

          # We fetch the transcript as a dependency and provide it to the build.
          # This is necessary because the Nix sandbox is read-only and downloading during tests would fail
          BARRETENBERG_TRANSCRIPT = pkgs.fetchurl {
            url = "http://aztec-ignition.s3.amazonaws.com/MAIN%20IGNITION/sealed/transcript00.dat";
            sha256 = "sha256-ryR/d+vpOCxa3gM0lze2UVUKNUinj0nN3ScCfysN84k=";
          };
        };

        # if file exists in git tree, commit hash wil bea read from it
        # or unknown value will be assigned
        COMMIT_HASH = if builtins.pathExists ./.commit
        then builtins.readFile ./.commit
        else "unknown";

        # rev attribute meta is only available when nix build https://github.com/noir-lang/noir 
        # is issued therefore reading this info from file is a hack for CI
        GIT_COMMIT = if (self ? rev) then self.rev else COMMIT_HASH;
        GIT_DIRTY = "false";

        commonArgs = {
          pname = "nargo";
          src = ./.;

          nativeBuildInputs = [
            # This provides the pkg-config tool to find barretenberg & other native libraries
            pkgs.pkg-config
            # This provides the `lld` linker to cargo
            pkgs.llvmPackages.bintools
          ];

          buildInputs = [
            pkgs.llvmPackages.openmp
            pkgs.barretenberg
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            # Need libiconv and apple Security on Darwin. See https://github.com/ipetkov/crane/issues/156
            pkgs.libiconv
            pkgs.darwin.apple_sdk.frameworks.Security
          ];

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

          inherit GIT_COMMIT;
          inherit GIT_DIRTY;

        } // environment;

        src = pkgs.copyPathToStore ./.;


        nargo = craneLib.buildPackage ({

          doCheck = true;
          
          cargoBuildCommand = "cargo build --release";

        } // commonArgs);

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      in rec {
        checks = { 
          cargo-check = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;

            doCheck = true;
          });

          cargo-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;

            cargoClippyExtraArgs = "--all-targets --workspace -- -D warnings";

            doCheck = true;
          });

          cargo-test = craneLib.cargoTest (commonArgs // {
            inherit cargoArtifacts;

            cargoTestArgs = "--workspace -- --test-threads=1";

            doCheck = true;
          });
        };

        packages.default = nargo;

        apps.default = flake-utils.lib.mkApp { drv = nargo; };

        devShells.default = pkgs.mkShell.override { stdenv = pkgs.llvmPackages.stdenv; } {
          inputsFrom = builtins.attrValues self.checks;

          buildInputs = packages.default.buildInputs ;

          inherit COMMIT_HASH;

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          # Uncertain if below line is needed. dev Shell is not yet fully working
          # BINDGEN_EXTRA_CLANG_ARGS = "-I${pkgs.barretenberg}/include -isystem ${pkgs.llvmPackages.libcxx.dev}/include";

          TERM = "xterm-256color";

          nativeBuildInputs = with pkgs; packages.default.buildInputs ++ [
            which
            starship
            git
            cargo
            rustc
          ];

          shellHook = ''
            eval "$(starship init bash)"
          '';

        };
      });
}
