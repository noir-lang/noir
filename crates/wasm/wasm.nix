{ nixpkgs ? import <nixpkgs> {} }:

let
  rustOverlay = builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
  llvm11stdenv = pkgs.llvmPackages_10.stdenv;

  # NixOS 22.05
  pinnedPkgs = fetchTarball "https://github.com/NixOS/nixpkgs/archive/0938d73bb143f4ae037143572f11f4338c7b2d1c.tar.gz"; 
  

  pkgs = import pinnedPkgs {
    overlays = [ (import rustOverlay) ];
  };

  rustbin = pkgs.rust-bin.stable.latest.default.override {
    extensions = [ "rust-src" ];
    targets = [ "wasm32-unknown-unknown" ];
  };
in
pkgs.mkShell.override { stdenv = llvm11stdenv;} {
  
  nativeBuildInputs = with pkgs; [
    binaryen
    jq
  ];

  buildInputs = with pkgs; [
    pkg-config
    rustbin
    wasm-pack
  ];

}