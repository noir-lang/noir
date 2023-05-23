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
      };

      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

      environment = {
        # rust-bindgen needs to know the location of libclang
        LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

        # Barretenberg fails if tests are run on multiple threads, so we set the test thread
        # count to 1 throughout the entire project
        #
        # Note: Setting this allows for consistent behavior across build and shells, but is mostly
        # hidden from the developer - i.e. when they see the command being run via `nix flake check`
        RUST_TEST_THREADS = "1";

        # We enable backtraces on any failure for help with debugging
        RUST_BACKTRACE = "1";

        # We set the environment variable because barretenberg must be compiled in a special way for wasm
        BARRETENBERG_BIN_DIR = "${pkgs.barretenberg-wasm}/bin";
      };

      # The `self.rev` property is only available when the working tree is not dirty
      GIT_COMMIT = if (self ? rev) then self.rev else "unknown";
      GIT_DIRTY = if (self ? rev) then "false" else "true";

      # As per https://discourse.nixos.org/t/gcc11stdenv-and-clang/17734/7 since it seems that aarch64-linux uses
      # gcc9 instead of gcc11 for the C++ stdlib, while all other targets we support provide the correct libstdc++
      stdenv =
        if (pkgs.stdenv.targetPlatform.isGnu && pkgs.stdenv.targetPlatform.isAarch64) then
          pkgs.overrideCC pkgs.llvmPackages.stdenv (pkgs.llvmPackages.clang.override { gccForLibs = pkgs.gcc11.cc; })
        else
          pkgs.llvmPackages.stdenv;

      # Combine the environment and other configuration needed for crane to build our Rust packages
      commonArgs = environment // {
        pname = "noir";
        # x-release-please-start-version
        version = "0.6.0";
        # x-release-please-end

        # Use our custom stdenv to build and test our Rust project
        inherit stdenv;

        src = ./.;

        # Running checks don't do much more than compiling itself and increase
        # the build time by a lot, so we disable them throughout all our flakes
        doCheck = false;

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

        inherit GIT_COMMIT;
        inherit GIT_DIRTY;
      };

      # The `port` is parameterized to support parallel test runs without colliding static servers
      testArgs = port: {
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
      cargoArtifacts = craneLib.buildDepsOnly commonArgs;

      noir = craneLib.buildPackage (commonArgs // {
        inherit cargoArtifacts;
      });
    in
    rec {
      checks = {
        cargo-clippy = craneLib.cargoClippy (commonArgs // {
          inherit cargoArtifacts;

          # TODO(#1198): It'd be nice to include these flags when running `cargo clippy` in a devShell.
          cargoClippyExtraArgs = "--all-targets -- -D warnings";
        });

        cargo-test = craneLib.cargoTest (commonArgs // (testArgs 8000) // {
          inherit cargoArtifacts;

          # TODO(#1198): It'd be nice to include this flag when running `cargo test` in a devShell.
          cargoTestExtraArgs = "--workspace";

          # It's unclear why doCheck needs to be enabled for tests to run but not clippy
          doCheck = true;
        });
      };

      packages.default = noir;

      # We expose the `cargo-artifacts` derivation so we can cache our cargo dependencies in CI
      packages.cargo-artifacts = cargoArtifacts;

      # TODO(#1197): Look into installable apps with Nix flakes
      # apps.default = flake-utils.lib.mkApp { drv = nargo; };

      # Setup the environment to match the stdenv from `nix build` & `nix flake check`, and
      # combine it with the environment settings, the inputs from our checks derivations,
      # and extra tooling via `nativeBuildInputs`
      devShells.default = pkgs.mkShell.override { inherit stdenv; } (environment // {
        inputsFrom = builtins.attrValues checks;

        nativeBuildInputs = with pkgs; [
          which
          starship
          git
          nil
          nixpkgs-fmt
          llvmPackages.lldb # This ensures the right lldb is in the environment for running rust-lldb
        ];

        shellHook = ''
          eval "$(starship init bash)"
        '';
      });
    });
}
