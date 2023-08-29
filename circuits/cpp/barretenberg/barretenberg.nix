{ overrideCC, stdenv, llvmPackages, cmake, ninja, lib, callPackage, gcc11 }:
let
  targetPlatform = stdenv.targetPlatform;
  buildEnv =
    if (stdenv.targetPlatform.isGnu && stdenv.targetPlatform.isAarch64) then
    # As per https://discourse.nixos.org/t/gcc11stdenv-and-clang/17734/7 since it seems that aarch64-linux uses
    # gcc9 instead of gcc11 for the C++ stdlib, while all other targets we support provide the correct libstdc++
      overrideCC llvmPackages.stdenv (llvmPackages.clang.override { gccForLibs = gcc11.cc; })
    else
      llvmPackages.stdenv;
  optionals = lib.lists.optionals;
  toolchain_file = ./cpp/cmake/toolchains/${targetPlatform.system}.cmake;
in
buildEnv.mkDerivation
{
  pname = "libbarretenberg";
  version = "0.4.6"; # x-release-please-version

  src = ./cpp;

  nativeBuildInputs = [ cmake ninja ];

  buildInputs = [ llvmPackages.openmp ];

  cmakeFlags = [
    "-DTESTING=OFF"
    "-DBENCHMARKS=OFF"
    "-DDISABLE_ASM=ON"
    "-DDISABLE_ADX=ON"
    "-DCMAKE_TOOLCHAIN_FILE=${toolchain_file}"
    "-DCMAKE_BUILD_TYPE=RelWithAssert"
  ];

  NIX_CFLAGS_COMPILE =
    optionals targetPlatform.isDarwin [ " -fno-aligned-allocation" ];

  enableParallelBuilding = true;
}
