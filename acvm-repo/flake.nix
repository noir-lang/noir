{
  description = "Javascript bindings for the ACVM";

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
    { self, nixpkgs, crane, flake-utils, rust-overlay, ... }: #, barretenberg
    flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          rust-overlay.overlays.default
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

      crateACVMJSDefinitions = craneLib.crateNameFromCargoToml {
        cargoToml = ./acvm_js/Cargo.toml;
      };

      crateACVMDefinitions = craneLib.crateNameFromCargoToml {
        cargoToml = ./acvm/Cargo.toml;
      };


      sharedEnvironment = {
        # Barretenberg fails if tests are run on multiple threads, so we set the test thread
        # count to 1 throughout the entire project
        #
        # Note: Setting this allows for consistent behavior across build and shells, but is mostly
        # hidden from the developer - i.e. when they see the command being run via `nix flake check`
        # RUST_TEST_THREADS = "1";
        BARRETENBERG_ARCHIVE = builtins.fetchurl {
          url = "https://github.com/AztecProtocol/barretenberg/releases/download/barretenberg-v0.4.5/acvm_backend.wasm.tar.gz";
          sha256 = "sha256:0z24yhvxc0dr13xj7y4xs9p42lzxwpazrmsrdpcgynfajkk6vqy4";
        };
      };

      wasmEnvironment = sharedEnvironment // { };

      sourceFilter = path: type:
        (craneLib.filterCargoSources path type);

      # The `self.rev` property is only available when the working tree is not dirty
      GIT_COMMIT = if (self ? rev) then self.rev else "unknown";
      GIT_DIRTY = if (self ? rev) then "false" else "true";

      extraBuildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
        # Need libiconv and apple Security on Darwin. See https://github.com/ipetkov/crane/issues/156
        pkgs.libiconv
        pkgs.darwin.apple_sdk.frameworks.Security
      ];

      commonArgs = {
        inherit (crateACVMDefinitions) pname version;
        src = pkgs.lib.cleanSourceWith {
          src = craneLib.path {
            path = ./.;
          };
          filter = sourceFilter;
        };

        cargoClippyExtraArgs = "--package acvm_js --all-targets -- -D warnings";
        cargoTestExtraArgs = "--workspace";

        # We don't want to run checks or tests when just building the project
        doCheck = false;
      };

      # Combine the environment and other configuration needed for crane to build with the wasm feature
      acvmjsWasmArgs = wasmEnvironment // commonArgs // {

        inherit (crateACVMJSDefinitions) pname version;

        cargoExtraArgs = "--package acvm_js --target=wasm32-unknown-unknown";

        cargoVendorDir = craneLib.vendorCargoDeps {
          src = ./.;
        };

        buildInputs = [ ] ++ extraBuildInputs;

      };

      # Build *just* the cargo dependencies, so we can reuse all of that work between runs
      cargo-artifacts = craneLib.buildDepsOnly commonArgs;

      wasm-bindgen-cli = pkgs.callPackage ./acvm_js/nix/wasm-bindgen-cli/default.nix {
        rustPlatform = pkgs.makeRustPlatform {
          rustc = rustToolchain;
          cargo = rustToolchain;
        };
      };

    in
    rec {
      checks = {

        cargo-clippy = craneLib.cargoClippy (commonArgs // sharedEnvironment // {
          inherit GIT_COMMIT GIT_DIRTY;

          cargoArtifacts = cargo-artifacts;
          doCheck = true;
        });

        cargo-test = craneLib.cargoTest (commonArgs // sharedEnvironment // {
          inherit GIT_COMMIT GIT_DIRTY;

          cargoArtifacts = cargo-artifacts;
          doCheck = true;
        });

        cargo-fmt = craneLib.cargoFmt (commonArgs // sharedEnvironment // {
          inherit GIT_COMMIT GIT_DIRTY;

          cargoArtifacts = cargo-artifacts;
          doCheck = true;
        });

      };

      packages = {
        default = craneLib.mkCargoDerivation (acvmjsWasmArgs // rec {

          cargoArtifacts = cargo-artifacts;

          inherit GIT_COMMIT;
          inherit GIT_DIRTY;

          COMMIT_SHORT = builtins.substring 0 7 GIT_COMMIT;
          VERSION_APPENDIX = if GIT_DIRTY == "true" then "-dirty" else "";

          src = ./.; #craneLib.cleanCargoSource (craneLib.path ./.);

          nativeBuildInputs = with pkgs; [
            binaryen
            which
            git
            jq
            rustToolchain
            wasm-bindgen-cli
          ];

          CARGO_TARGET_DIR = "./target";

          buildPhaseCargoCommand = ''
            bash ./acvm_js/buildPhaseCargoCommand.sh
          '';

          installPhase = ''
            bash ./acvm_js/installPhase.sh        
          '';

        });
        inherit cargo-artifacts;
      };

      # Setup the environment to match the stdenv from `nix build` & `nix flake check`, and
      # combine it with the environment settings, the inputs from our checks derivations,
      # and extra tooling via `nativeBuildInputs`
      devShells.default = pkgs.mkShell (wasmEnvironment // {
        # inputsFrom = builtins.attrValues checks;

        nativeBuildInputs = with pkgs; [
          starship
          nil
          nixpkgs-fmt
          which
          git
          jq
          toml2json
          rustToolchain
          wasm-bindgen-cli
          nodejs
          yarn
        ];

        shellHook = ''
          eval "$(starship init bash)"
        '';
      });
    });
}
