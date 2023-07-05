# Copied from https://github.com/ereslibre/nixities/blob/2c60af777fc863f90e6e4eeffcf3465def93a1f3/packages/wasi-sdk/default.nix
# with a fix for the autoPatchelfHook needing libstdc++.so and some refactor
{ lib, pkgs, stdenv }:
let
  pname = "wasi-sdk";
  version = "20";
in
pkgs.stdenv.mkDerivation {
  inherit pname version;

  sourceRoot = "${pname}-${version}.0";
  dontBuild = true;
  dontConfigure = true;
  dontStrip = true;

  nativeBuildInputs =
    lib.optional stdenv.isLinux (with pkgs; [ autoPatchelfHook ]);

  # Needed by autoPatchelfHook to have libstdc++
  # see https://discourse.nixos.org/t/autopatchelfhook-not-patching-all-dependencies/14634/6
  buildInputs =
    lib.optional stdenv.isLinux [ stdenv.cc.cc.lib ];

  installPhase = ''
    mkdir -p $out/{bin,lib,share}
    mv bin/* $out/bin/
    mv lib/* $out/lib/
    mv share/* $out/share/
  '';

  src =
    let
      tarball =
        if stdenv.hostPlatform.isDarwin then {
          suffix = "macos";
          hash = "sha256-juJfnD/eYY/upcV62tOFFSYmeEtra1L7Vj5e2DK/U+8=";
        } else {
          suffix = "linux";
          hash = "sha256-cDATnUlaGfvsy5RJFQwrFTHhXY+3RBmHKnGadYCq0Pk=";
        };
    in

    pkgs.fetchurl {
      url =
        "https://github.com/WebAssembly/${pname}/releases/download/${pname}-${version}/${pname}-${version}.0-${tarball.suffix}.tar.gz";
      hash = tarball.hash;
    };
}
