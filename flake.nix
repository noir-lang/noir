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

        BARRETENBERG_ARCHIVE = builtins.fetchurl {
          url = "https://github.com/AztecProtocol/barretenberg/releases/download/barretenberg-v0.4.5/acvm_backend.wasm.tar.gz";
          sha256 = "sha256:0z24yhvxc0dr13xj7y4xs9p42lzxwpazrmsrdpcgynfajkk6vqy4";
        };
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

      # We use `include_str!` macro to embed the solidity verifier template so we need to create a special
      # source filter to include .sol files in addition to usual rust/cargo source files.
      solidityFilter = path: _type: builtins.match ".*sol$" path != null;
      # We use `.bytecode` and `.tr` files to test interactions with `bb` so we add a source filter to include these.
      bytecodeFilter = path: _type: builtins.match ".*bytecode$" path != null;
      witnessFilter = path: _type: builtins.match ".*tr$" path != null;
      # We use `.nr` and `.toml` files in tests so we need to create a special source
      # filter to include those files in addition to usual rust/cargo source files
      noirFilter = path: _type: builtins.match ".*nr$" path != null;
      tomlFilter = path: _type: builtins.match ".*toml$" path != null;
      sourceFilter = path: type:
        (solidityFilter path type) || (bytecodeFilter path type)|| (witnessFilter path type) || (noirFilter path type) || (tomlFilter path type) || (craneLib.filterCargoSources path type);

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
      ] ++ [
        # Need to install various packages used by the `bb` binary.
        pkgs.curl
        stdenv.cc.cc.lib
        pkgs.gcc.cc.lib
        pkgs.gzip
      ];

      sharedArgs = {
        # x-release-please-start-version
        version = "0.11.1";
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
        ] ++ pkgs.lib.optionals stdenv.isLinux [
          # This is linux specific and used to patch the rpath and interpreter of the bb binary
          pkgs.patchelf
        ];

        buildInputs = [
        ] ++ extraBuildInputs;
      };

      # Combine the environmnet with cargo args needed to build wasm package
      noirWasmArgs = sharedEnvironment // sharedArgs // {
        pname = "noir_wasm";

        src = ./.;

        cargoExtraArgs = "--package noir_wasm --target wasm32-unknown-unknown";

        buildInputs = [ ] ++ extraBuildInputs;

        doCheck = false;
      };

      # Combine the environment with cargo args needed to build wasm package
      noirc_abi_WasmArgs = sharedEnvironment // sharedArgs // {
        pname = "noirc_abi_wasm";

        src = ./.;

        cargoExtraArgs = "--package noirc_abi_wasm --target wasm32-unknown-unknown";

        buildInputs = [ ] ++ extraBuildInputs;

        doCheck = false;
      };
      
      # Conditionally download the binary based on whether it is linux or mac
      bb_binary = let
        platformSpecificUrl = if stdenv.hostPlatform.isLinux then
          "https://github.com/AztecProtocol/barretenberg/releases/download/barretenberg-v0.4.3/bb-ubuntu.tar.gz"
        else if stdenv.hostPlatform.isDarwin then
          "https://github.com/AztecProtocol/barretenberg/releases/download/barretenberg-v0.4.3/barretenberg-x86_64-apple-darwin.tar.gz"
        else
          throw "Unsupported platform";

        platformSpecificHash = if stdenv.hostPlatform.isLinux then
          "sha256:0rcsjws87f4v28cw9734c10pg7c49apigf4lg3m0ji5vbhhmfnhr"
        else if stdenv.hostPlatform.isDarwin then
          "sha256:0pnsd56z0vkai7m0advawfgcvq9jbnpqm7lk98n5flqj583x3w35"
        else
          throw "Unsupported platform";
      in builtins.fetchurl {
        url = platformSpecificUrl;
        sha256 = platformSpecificHash;
      };

      # The `port` is parameterized to support parallel test runs without colliding static servers
      testArgs = port: testEnvironment // {
        BB_BINARY_PATH = "/tmp/backend_binary";

        BB_BINARY_URL = "http://0.0.0.0:${toString port}/${builtins.baseNameOf bb_binary}";

        # We provide `barretenberg-transcript00` from the overlay to the tests as a URL hosted via a static server
        # This is necessary because the Nix sandbox has no network access and downloading during tests would fail
        BARRETENBERG_TRANSCRIPT_URL = "http://0.0.0.0:${toString port}/${builtins.baseNameOf pkgs.barretenberg-transcript00}";

        # This copies the `barretenberg-transcript00` from the Nix store into this sandbox
        # which avoids exposing the entire Nix store to the static server it starts
        # The static server is moved to the background and killed after checks are completed
        #
        # We also set the NARGO_BACKEND_CACHE_DIR environment variable to the $TMP directory so we can successfully cache
        # the transcript; which isn't possible with the default path because the Nix sandbox disabled $HOME
        preCheck = ''
          echo "Extracting bb binary"
          mkdir extracted
          tar -xf ${bb_binary} -C extracted

          # Conditionally patch the binary for Linux
          ${if stdenv.hostPlatform.isLinux then ''

            cp extracted/cpp/build/bin/bb /tmp/backend_binary
          
            echo "Patching bb binary for Linux"
            patchelf --set-rpath "${stdenv.cc.cc.lib}/lib:${pkgs.gcc.cc.lib}/lib" /tmp/backend_binary
            patchelf --set-interpreter ${stdenv.cc.libc}/lib/ld-linux-x86-64.so.2 /tmp/backend_binary
          '' else if stdenv.hostPlatform.isDarwin then ''
            cp extracted/bb /tmp/backend_binary
          '' else
            throw "Unsupported platform"
          }

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
      noir-wasm-cargo-artifacts = craneLib.buildDepsOnly noirWasmArgs;
      noirc-abi-wasm-cargo-artifacts = craneLib.buildDepsOnly noirc_abi_WasmArgs;

      noir-native = craneLib.buildPackage (nativeArgs // {
        inherit GIT_COMMIT GIT_DIRTY;

        cargoArtifacts = native-cargo-artifacts;

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

        cargo-fmt = craneLib.cargoFmt (nativeArgs // {
          inherit GIT_COMMIT GIT_DIRTY;

          cargoArtifacts = native-cargo-artifacts;
          doCheck = true;
        });

        cargo-test = craneLib.cargoTest (nativeArgs // (testArgs 8000) // {
          inherit GIT_COMMIT GIT_DIRTY;

          cargoArtifacts = native-cargo-artifacts;
        });
      };

      packages = {
        default = noir-native;

        inherit noir-native;

        # We expose the `*-cargo-artifacts` derivations so we can cache our cargo dependencies in CI
        inherit native-cargo-artifacts;
        inherit noir-wasm-cargo-artifacts;
        inherit noirc-abi-wasm-cargo-artifacts;
      };

      # TODO(#1197): Look into installable apps with Nix flakes
      # apps.default = flake-utils.lib.mkApp { drv = nargo; };

      # Setup the environment to match the stdenv from `nix build` & `nix flake check`, and
      # combine it with the environment settings, the inputs from our checks derivations,
      # and extra tooling via `nativeBuildInputs`
      devShells.default = pkgs.mkShell.override { inherit stdenv; } (nativeEnvironment // wasmEnvironment // testEnvironment // {
        inputsFrom = builtins.attrValues checks;

        nativeBuildInputs = with pkgs; [
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
          rust-bin.stable."1.66.1".default
          rust-analyzer
          rustup
          nodejs-18_x 
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
          bash compiler/wasm/buildPhaseCargoCommand.sh release
        '';

        installPhase = ''
          bash compiler/wasm/installPhase.sh
        '';

      });

      # TODO: This fails with a "section too large" error on MacOS so we should limit to linux targets
      # or fix the failure
      packages.noirc_abi_wasm = craneLib.buildPackage (noirc_abi_WasmArgs // {

        inherit GIT_COMMIT;
        inherit GIT_DIRTY;
        doCheck = false;

        cargoArtifacts = noirc-abi-wasm-cargo-artifacts;

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
          bash tooling/noirc_abi_wasm/buildPhaseCargoCommand.sh release
        '';

        installPhase = ''
          bash tooling/noirc_abi_wasm/installPhase.sh
        '';

      });

    });
}

