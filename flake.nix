{
  description = "Build the Noir programming language";

  # All of these inputs (a.k.a. dependencies) need to align with inputs we
  # use so they use the `inputs.*.follows` syntax to reference our inputs
  inputs = {
    nixpkgs = {
      url = "github:NixOS/nixpkgs/nixos-23.05";
    };

    flake-utils = {
      url = "github:numtide/flake-utils";
    };

    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };

    crane = {
      url = "github:ipetkov/crane";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
        flake-compat.follows = "flake-compat";
      };
    };
  };

  outputs =
    { self, nixpkgs, crane, flake-utils, fenix, ... }:
    flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
      };

      rustToolchain = fenix.packages.${system}.fromToolchainFile {
        file = ./rust-toolchain.toml;
        sha256 = "sha256-R0F0Risbr74xg9mEYydyebx/z0Wu6HI0/KWwrV30vZo=";
      };

      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

      # The `self.rev` property is only available when the working tree is not dirty
      GIT_COMMIT = if (self ? rev) then self.rev else "unknown";
      GIT_DIRTY = if (self ? rev) then "false" else "true";

      extraBuildInputs = [ ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
        # Need libiconv and apple Security on Darwin. See https://github.com/ipetkov/crane/issues/156
        pkgs.libiconv
        pkgs.darwin.apple_sdk.frameworks.Security
      ];

      environment = {
        # We enable backtraces on any failure for help with debugging
        RUST_BACKTRACE = "1";

        # We download the Wasm version of `acvm_backend` in the barretenberg releases for the ACVM `blackbox_solver`
        BARRETENBERG_ARCHIVE = pkgs.fetchurl {
          url = "https://github.com/AztecProtocol/barretenberg/releases/download/barretenberg-v0.4.5/acvm_backend.wasm.tar.gz";
          sha256 = "sha256-xONt5pTKWf/YbVnX/NXl/VNBbtKd+CP7CLkB1jf0RHw=";
        };
      };

      # Configuration shared between builds
      config = {
        # x-release-please-start-version
        version = "0.23.0";
        # x-release-please-end

        src = pkgs.lib.cleanSourceWith {
          src = craneLib.path ./.;
          # Custom filter with various file extensions that we rely upon to build packages
          # Currently: `.nr`, `.sol`, `.sh`, `.json`, `.md` and `.wasm`
          filter = path: type:
            (builtins.match ".*\.(nr|sol|sh|json|md|wasm)$" path != null) || (craneLib.filterCargoSources path type);
        };

        # TODO(#1198): It'd be nice to include these flags when running `cargo clippy` in a devShell.
        cargoClippyExtraArgs = "--all-targets -- -D warnings";

        # TODO(#1198): It'd be nice to include this flag when running `cargo test` in a devShell.
        cargoTestExtraArgs = "--workspace";
      };

      # Combine the environment and other configuration needed for Crane to build our Rust packages
      nativeConfig = environment // config // {
        nativeBuildInputs = [ ];

        buildInputs = [ ] ++ extraBuildInputs;
      };

      # Combine the environmnet and other configuration needed for Crane to build our Wasm packages
      wasmConfig = environment // config // {
        CARGO_TARGET_DIR = "./target";

        nativeBuildInputs = with pkgs; [
          which
          git
          jq
          rustToolchain
          wasm-bindgen-cli
          binaryen
        ];

        buildInputs = [ ] ++ extraBuildInputs;
      };

      # Build *just* the cargo dependencies, so we can reuse all of that work between runs
      native-cargo-artifacts = craneLib.buildDepsOnly (nativeConfig // {
        pname = "nargo";
      });
      noirc-abi-wasm-cargo-artifacts = craneLib.buildDepsOnly (wasmConfig // {
        pname = "noirc_abi_wasm";
      });
      acvm-js-cargo-artifacts = craneLib.buildDepsOnly (wasmConfig // {
        pname = "acvm_js";
      });

      nargo = craneLib.buildPackage (nativeConfig // {
        pname = "nargo";

        inherit GIT_COMMIT GIT_DIRTY;

        cargoArtifacts = native-cargo-artifacts;

        # We don't want to run tests because they don't work in the Nix sandbox
        doCheck = false;
      });

      noirc_abi_wasm = craneLib.buildPackage (wasmConfig // rec {
        pname = "noirc_abi_wasm";

        inherit GIT_COMMIT GIT_DIRTY;

        cargoArtifacts = noirc-abi-wasm-cargo-artifacts;

        cargoExtraArgs = "--package ${pname} --target wasm32-unknown-unknown";

        buildPhaseCargoCommand = ''
          bash tooling/noirc_abi_wasm/buildPhaseCargoCommand.sh release
        '';

        installPhase = ''
          bash tooling/noirc_abi_wasm/installPhase.sh
        '';

        # We don't want to run tests because they don't work in the Nix sandbox
        doCheck = false;
      });

      acvm_js = craneLib.buildPackage (wasmConfig // rec {
        pname = "acvm_js";

        inherit GIT_COMMIT GIT_DIRTY;

        cargoArtifacts = acvm-js-cargo-artifacts;

        cargoExtraArgs = "--package ${pname} --target wasm32-unknown-unknown";

        buildPhaseCargoCommand = ''
          bash acvm-repo/acvm_js/buildPhaseCargoCommand.sh release
        '';

        installPhase = ''
          bash acvm-repo/acvm_js/installPhase.sh
        '';

        # We don't want to run tests because they don't work in the Nix sandbox
        doCheck = false;
      });

      wasm-bindgen-cli = pkgs.callPackage ./wasm-bindgen-cli.nix {
        rustPlatform = pkgs.makeRustPlatform {
          rustc = rustToolchain;
          cargo = rustToolchain;
        };
      };
    in
    {
      # We use `checks` to run `cargo clippy` and `cargo fmt` since we disable checks in the primary derivations
      checks = {
        cargo-clippy = craneLib.cargoClippy (nativeConfig // {
          pname = "noir";

          inherit GIT_COMMIT GIT_DIRTY;

          cargoArtifacts = native-cargo-artifacts;
        });

        cargo-fmt = craneLib.cargoFmt (nativeConfig // {
          pname = "noir";

          inherit GIT_COMMIT GIT_DIRTY;

          cargoArtifacts = native-cargo-artifacts;
        });
      };

      packages = {
        default = nargo;

        # Nix flakes cannot build more than one derivation in one command (see https://github.com/NixOS/nix/issues/5591)
        # so we use `symlinkJoin` to build everything as the "all" package.
        all = pkgs.symlinkJoin { name = "all"; paths = [ nargo noirc_abi_wasm acvm_js ]; };
        all_wasm = pkgs.symlinkJoin { name = "all_wasm"; paths = [ noirc_abi_wasm acvm_js ]; };

        # We also export individual packages to enable `nix build .#nargo -L`, etc.
        inherit nargo;
        inherit noirc_abi_wasm;
        inherit acvm_js;

        # We expose the `*-cargo-artifacts` derivations so we can cache our cargo dependencies in CI
        inherit native-cargo-artifacts;
        inherit noirc-abi-wasm-cargo-artifacts;
        inherit acvm-js-cargo-artifacts;
      };

      # Setup the environment to match the environment settings, the inputs from our checks derivations,
      # and extra tooling via `nativeBuildInputs`
      devShells.default = pkgs.mkShell (environment // {
        inputsFrom = [
          nargo
          noirc_abi_wasm
          acvm_js
        ];

        # Additional tools that weren't included as `nativeBuildInputs` of any of the derivations in `inputsFrom`
        nativeBuildInputs = with pkgs; [
          # Rust toolchain
          rustToolchain
          # Other tools
          starship
          yarn
          nodejs-18_x
          # Used by the `bb` binary
          curl
          gzip
          # This ensures the right lldb is in the environment for running rust-lldb
          llvmPackages.lldb
          # Nix tools
          nil
          nixpkgs-fmt
        ];

        shellHook = ''
          eval "$(starship init bash)"
        '';
      });
    });
}

