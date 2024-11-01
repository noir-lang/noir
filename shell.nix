# stanm: The single purpose of this script is to bootstrap rustup.
{
  pkgs,
  self',
  inputs',
  ...
}: let
  inherit (pkgs) lib stdenv mkShell;
  inherit (pkgs.darwin.apple_sdk) frameworks;
in
  mkShell {
    packages =
      [
        pkgs.alejandra
        self'.legacyPackages.rustToolchain
        pkgs.rustfilt
      ]
      ++ lib.optionals stdenv.isDarwin [
        pkgs.libiconv
        frameworks.CoreServices
      ];
    shellHook = ''
      export PATH="${inputs'.fenix.packages.latest.llvm-tools}/lib/rustlib/x86_64-unknown-linux-gnu/bin:$PATH"
    '';
  }
