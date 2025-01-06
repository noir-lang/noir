{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];
      perSystem = {
        pkgs,
        inputs',
        self',
        ...
      }: {
        legacyPackages.rustToolchain = with inputs'.fenix.packages;
        with latest;
          combine [
            cargo
            clippy
            rust-analyzer
            rust-src
            rustc
            rustfmt
            llvm-tools
          ];

        devShells.default = import ./shell.nix {inherit pkgs self' inputs';};

        packages.default = import ./noir.nix {inherit pkgs;};
      };
    };
}
