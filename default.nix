{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.openssl
    pkgs.pkg-config
    pkgs.cmake
    pkgs.llvmPackages.openmp
    pkgs.rustup
  ];
}