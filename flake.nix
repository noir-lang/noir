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

    barretenberg = {
      url = "github:AztecProtocol/barretenberg";
      # All of these inputs (a.k.a. dependencies) need to align with inputs we
      # use so they use the `inputs.*.follows` syntax to reference our inputs
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
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

      rustToolchain = pkgs.rust-bin.stable."1.66.0".default.override {
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

      sharedEnvironment = {
        # We enable backtraces on any failure for help with debugging
        RUST_BACKTRACE = "1";
      };

      nativeEnvironment = sharedEnvironment // {
        # rust-bindgen needs to know the location of libclang
        LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
      };

      wasmEnvironment = sharedEnvironment // {
        # We set the environment variable because barretenberg must be compiled in a special way for wasm
        BARRETENBERG_BIN_DIR = "${pkgs.barretenberg-wasm}/bin";
      };

      testEnvironment = sharedEnvironment // { };

      # The `self.rev` property is only available when the working tree is not dirty
      GIT_COMMIT = if (self ? rev) then self.rev else "unknown";
      GIT_DIRTY = if (self ? rev) then "false" else "true";

      # We use `.nr` and `.toml` files in tests so we need to create a special source
      # filter to include those files in addition to usual rust/cargo source files
      noirFilter = path: _type: builtins.match ".*nr$" path != null;
      tomlFilter = path: _type: builtins.match ".*toml$" path != null;
      sourceFilter = path: type:
        (noirFilter path type) || (tomlFilter path type) || (craneLib.filterCargoSources path type);

      # As per https://discourse.nixos.org/t/gcc11stdenv-and-clang/17734/7 since it seems that aarch64-linux uses
      # gcc9 instead of gcc11 for the C++ stdlib, while all other targets we support provide the correct libstdc++
      stdenv =
        if (pkgs.stdenv.targetPlatform.isGnu && pkgs.stdenv.targetPlatform.isAarch64) then
          pkgs.overrideCC pkgs.llvmPackages.stdenv (pkgs.llvmPackages.clang.override { gccForLibs = pkgs.gcc11.cc; })
        else
          pkgs.llvmPackages.stdenv;

      extraBuildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
        # Need libiconv and apple Security on Darwin. See https://github.com/ipetkov/crane/issues/156
        pkgs.libiconv
        pkgs.darwin.apple_sdk.frameworks.Security
      ];

      sharedArgs = {
        # x-release-please-start-version
        version = "0.10.1";
        # x-release-please-end

        src = pkgs.lib.cleanSourceWith {
          src = craneLib.path ./.;
          filter = sourceFilter;
        };

        # TODO(#1198): It'd be nice to include these flags when running `cargo clippy` in a devShell.
        cargoClippyExtraArgs = "--all-targets -- -D warnings";

        # TODO(#1198): It'd be nice to include this flag when running `cargo test` in a devShell.
        cargoTestExtraArgs = "--workspace";
      };

      # Combine the environment and other configuration needed for crane to build our Rust packages
      nativeArgs = nativeEnvironment // sharedArgs // {
        pname = "noir-native";

        # Use our custom stdenv to build and test our Rust project
        inherit stdenv;

        nativeBuildInputs = [
          # This provides the pkg-config tool to find barretenberg & other native libraries
          pkgs.pkg-config
          # This provides the `lld` linker to cargo
          pkgs.llvmPackages.bintools
        ];

        buildInputs = [
          pkgs.llvmPackages.openmp
          pkgs.barretenberg
        ] ++ extraBuildInputs;
      };

      # Combine the environment and other configuration needed for crane to build with the wasm feature
      wasmArgs = wasmEnvironment // sharedArgs // {
        pname = "noir-wasm";

        # We disable the default "plonk_bn254" feature and enable the "plonk_bn254_wasm" feature
        cargoExtraArgs = "--no-default-features --features='plonk_bn254_wasm'";

        buildInputs = [ ] ++ extraBuildInputs;
      };

      # Combine the environmnet with cargo args needed to build wasm package
      noirWasmArgs = sharedEnvironment // sharedArgs // {
        pname = "noir_wasm";

        src = ./.;

        cargoExtraArgs = "--package noir_wasm --target wasm32-unknown-unknown";

        buildInputs = [ ] ++ extraBuildInputs;

        doCheck = false;
      };

      # The `port` is parameterized to support parallel test runs without colliding static servers
      testArgs = port: testEnvironment // {
        # We provide `barretenberg-transcript00` from the overlay to the tests as a URL hosted via a static server
        # This is necessary because the Nix sandbox has no network access and downloading during tests would fail
        TRANSCRIPT_URL = "http://0.0.0.0:${toString port}/${builtins.baseNameOf pkgs.barretenberg-transcript00}";

        # This copies the `barretenberg-transcript00` from the Nix store into this sandbox
        # which avoids exposing the entire Nix store to the static server it starts
        # The static server is moved to the background and killed after checks are completed
        #
        # We also set the NARGO_BACKEND_CACHE_DIR environment variable to the $TMP directory so we can successfully cache
        # the transcript; which isn't possible with the default path because the Nix sandbox disabled $HOME
        preCheck = ''
          export NARGO_BACKEND_CACHE_DIR=$TMP
          cp ${pkgs.barretenberg-transcript00} .
          echo "Starting simple static server"
          ${pkgs.simple-http-server}/bin/simple-http-server --port ${toString port} --silent &
          HTTP_SERVER_PID=$!
        '';

        postCheck = ''
          kill $HTTP_SERVER_PID
        '';
      };

      # Build *just* the cargo dependencies, so we can reuse all of that work between runs
      native-cargo-artifacts = craneLib.buildDepsOnly nativeArgs;
      wasm-cargo-artifacts = craneLib.buildDepsOnly wasmArgs;
      noir-wasm-cargo-artifacts = craneLib.buildDepsOnly noirWasmArgs;

      noir-native = craneLib.buildPackage (nativeArgs // {
        inherit GIT_COMMIT GIT_DIRTY;

        cargoArtifacts = native-cargo-artifacts;

        # We don't want to run checks or tests when just building the project
        doCheck = false;
      });

      noir-wasm = craneLib.buildPackage (wasmArgs // {
        inherit GIT_COMMIT GIT_DIRTY;

        cargoArtifacts = wasm-cargo-artifacts;

        # We don't want to run checks or tests when just building the project
        doCheck = false;
      });

      wasm-bindgen-cli = pkgs.callPackage ./wasm-bindgen-cli.nix {
        rustPlatform = pkgs.makeRustPlatform {
          rustc = rustToolchain;
          cargo = rustToolchain;
        };
      };
    in
    rec {
      checks = {
        cargo-clippy = craneLib.cargoClippy (nativeArgs // {
          inherit GIT_COMMIT GIT_DIRTY;

          cargoArtifacts = native-cargo-artifacts;
        });

        cargo-test = craneLib.cargoTest (nativeArgs // (testArgs 8000) // {
          inherit GIT_COMMIT GIT_DIRTY;

          cargoArtifacts = native-cargo-artifacts;
        });
      };

      packages = {
        default = noir-native;

        inherit noir-native;
        inherit noir-wasm;

        # We expose the `*-cargo-artifacts` derivations so we can cache our cargo dependencies in CI
        inherit native-cargo-artifacts;
        inherit wasm-cargo-artifacts;
        inherit noir-wasm-cargo-artifacts;
      };

      # TODO(#1197): Look into installable apps with Nix flakes
      # apps.default = flake-utils.lib.mkApp { drv = nargo; };

      # Setup the environment to match the stdenv from `nix build` & `nix flake check`, and
      # combine it with the environment settings, the inputs from our checks derivations,
      # and extra tooling via `nativeBuildInputs`
      devShells.default = pkgs.mkShell.override { inherit stdenv; } (nativeEnvironment // wasmEnvironment // testEnvironment // {
        inputsFrom = builtins.attrValues checks;

        nativeBuildInputs = with pkgs; [
          which
          starship
          git
          nil
          nixpkgs-fmt
          toml2json
          llvmPackages.lldb # This ensures the right lldb is in the environment for running rust-lldb
          wasm-bindgen-cli
          jq
        ];

        shellHook = ''
          eval "$(starship init bash)"
        '';
      });

      # TODO: This fails with a "section too large" error on MacOS so we should limit to linux targets
      # or fix the failure
      packages.wasm = craneLib.buildPackage (noirWasmArgs // {

        inherit GIT_COMMIT;
        inherit GIT_DIRTY;
        doCheck = false;

        cargoArtifacts = noir-wasm-cargo-artifacts;

        COMMIT_SHORT = builtins.substring 0 7 GIT_COMMIT;
        VERSION_APPENDIX = if GIT_DIRTY == "true" then "-dirty" else "";
        PKG_PATH = "./pkg";
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

        buildPhaseCargoCommand = ''
          bash crates/wasm/buildPhaseCargoCommand.sh release
        '';

        installPhase = ''
          bash crates/wasm/installPhase.sh
        '';

      });
    });
}

