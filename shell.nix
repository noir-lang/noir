# stanm: The single purpose of this script is to bootstrap rustup.
{
  pkgs,
  self',
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
      ]
      ++ lib.optionals stdenv.isDarwin [
        pkgs.libiconv
        frameworks.CoreServices
      ];
  }
