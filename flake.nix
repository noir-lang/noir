{
  description = "Build the Noir programming language";

  inputs = {
    nixpkgs = {
      url = "github:NixOS/nixpkgs/nixos-22.11";
    };

    flake-utils = {
      url = "github:numtide/flake-utils";
    };

    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      # All of these inputs (a.k.a. dependencies) need to align with inputs we
      # use so they use the `inputs.*.follows` syntax to reference our inputs
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };

    crane = {
      url = "github:ipetkov/crane";
      # All of these inputs (a.k.a. dependencies) need to align with inputs we
      # use so they use the `inputs.*.follows` syntax to reference our inputs
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
        flake-compat.follows = "flake-compat";
        rust-overlay.follows = "rust-overlay";
      };
    };
  };

  outputs =
    { self, nixpkgs, crane, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          rust-overlay.overlays.default
        ];
      };

      rustVersion = "1.66.1";

      rustToolchain = pkgs.rust-bin.stable.${rustVersion}.default.override {
        # We include rust-src to ensure rust-analyzer works.
        # See https://discourse.nixos.org/t/rust-src-not-found-and-other-misadventures-of-developing-rust-on-nixos/11570/4
        extensions = [ "rust-src" ];
        targets = [ "wasm32-unknown-unknown" ]
          ++ pkgs.lib.optional (pkgs.hostPlatform.isx86_64 && pkgs.hostPlatform.isLinux) "x86_64-unknown-linux-gnu"
          ++ pkgs.lib.optional (pkgs.hostPlatform.isAarch64 && pkgs.hostPlatform.isLinux) "aarch64-unknown-linux-gnu"
          ++ pkgs.lib.optional (pkgs.hostPlatform.isx86_64 && pkgs.hostPlatform.isDarwin) "x86_64-apple-darwin"
          ++ pkgs.lib.optional (pkgs.hostPlatform.isAarch64 && pkgs.hostPlatform.isDarwin) "aarch64-apple-darwin";
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
        version = "0.11.1";
        # x-release-please-end

        src = pkgs.lib.cleanSourceWith {
          src = craneLib.path ./.;
          # Custom filter with various file extensions that we rely upon to build packages
          # Currently: `.nr`, `.sol`, `.sh`, `.json`, `.md`
          filter = path: type:
            (builtins.match ".*\.(nr|sol|sh|json|md)$" path != null) || (craneLib.filterCargoSources path type);
        };

        # TODO(#1198): It'd be nice to include these flags when running `cargo clippy` in a devShell.
        cargoClippyExtraArgs = "--all-targets -- -D warnings";

        # TODO(#1198): It'd be nice to include this flag when running `cargo test` in a devShell.
        cargoTestExtraArgs = "--workspace";
      };

      # Combine the environment and other configuration needed for crane to build our Rust packages
      nativeConfig = environment // config // {
        nativeBuildInputs = [ ];

        buildInputs = [ ] ++ extraBuildInputs;
      };

      # Combine the environmnet with cargo args needed to build wasm package
      wasmConfig = environment // config // rec {
        CARGO_TARGET_DIR = "./target";

        nativeBuildInputs = with pkgs; [
          which
          git
          jq
          rustToolchain
          wasm-bindgen-cli
          binaryen
          toml2json
        ];

        buildInputs = [ ] ++ extraBuildInputs;
      };

      # Build *just* the cargo dependencies, so we can reuse all of that work between runs
      native-cargo-artifacts = craneLib.buildDepsOnly (nativeConfig // {
        pname = "nargo";
      });
      noir-wasm-cargo-artifacts = craneLib.buildDepsOnly (wasmConfig // {
        pname = "noir_wasm";
      });
      noirc-abi-wasm-cargo-artifacts = craneLib.buildDepsOnly (wasmConfig // {
        pname = "noirc_abi_wasm";
      });

      nargo = craneLib.buildPackage (nativeConfig // {
        pname = "nargo";

        inherit GIT_COMMIT GIT_DIRTY;

        cargoArtifacts = native-cargo-artifacts;

        # We don't want to run tests because they don't work in the Nix sandbox
        doCheck = false;
      });

      noir_wasm = craneLib.buildPackage (wasmConfig // rec {
        pname = "noir_wasm";

        inherit GIT_COMMIT GIT_DIRTY;

        cargoArtifacts = noir-wasm-cargo-artifacts;

        cargoExtraArgs = "--package ${pname} --target wasm32-unknown-unknown";

        buildPhaseCargoCommand = ''
          bash compiler/wasm/buildPhaseCargoCommand.sh release
        '';

        installPhase = ''
          bash compiler/wasm/installPhase.sh
        '';

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
          doCheck = true;
        });
      };

      packages = {
        default = nargo;

        inherit nargo;
        inherit noir_wasm;
        inherit noirc_abi_wasm;

        # We expose the `*-cargo-artifacts` derivations so we can cache our cargo dependencies in CI
        inherit native-cargo-artifacts;
        inherit noir-wasm-cargo-artifacts;
        inherit noirc-abi-wasm-cargo-artifacts;
      };

      # Setup the environment to match the environment settings, the inputs from our checks derivations,
      # and extra tooling via `nativeBuildInputs`
      devShells.default = pkgs.mkShell (environment // {
        inputsFrom = builtins.attrValues checks;

        # Additional tools that weren't included as `nativeBuildInputs` of any of the derivations in `inputsFrom`
        nativeBuildInputs = with pkgs; [
          # Need to install various packages used by the `bb` binary.
          # pkgs.curl
          # stdenv.cc.cc.lib
          # pkgs.gcc.cc.lib
          # pkgs.gzip
          curl
          gzip
          which
          starship
          git
          nil
          nixpkgs-fmt
          toml2json
          llvmPackages.lldb # This ensures the right lldb is in the environment for running rust-lldb
          wasm-bindgen-cli
          jq
          binaryen
          yarn
          rust-bin.stable.${rustVersion}.default
          rust-analyzer
          rustup
          nodejs-18_x
        ];

        shellHook = ''
          eval "$(starship init bash)"
        '';
      });
    });
}

