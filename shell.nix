# stanm: The single purpose of this script is to bootstrap rustup.
{
  pkgs,
  inputs',
  ...
}: let
  frameworks = pkgs.darwin.apple_sdk.frameworks;
  rust = with inputs'.fenix.packages;
  with pkgs;
  with latest;
    combine ([
        cargo
        clippy
        rust-analyzer
        rust-src
        rustc
        rustfmt
        libiconv
        alejandra
      ]
      ++ lib.optionals stdenv.isDarwin [
        libiconv
        frameworks.CoreServices
      ]);
in
  pkgs.mkShell {
    packages = [
      rust
    ];
  }
