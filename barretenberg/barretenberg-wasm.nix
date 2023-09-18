{ stdenv, cmake, ninja, binaryen, callPackage }:
let
  toolchain_file = ./cpp/cmake/toolchains/wasm32-wasi.cmake;
  wasi-sdk = callPackage ./wasi-sdk.nix { };
in
stdenv.mkDerivation
{
  pname = "barretenberg.wasm";
  version = "0.7.6"; # x-release-please-version

  src = ./cpp;

  nativeBuildInputs = [ cmake ninja wasi-sdk ];

  buildInputs = [ ];

  cmakeFlags = [
    "-GNinja"
    "-DTESTING=OFF"
    "-DBENCHMARKS=OFF"
    "-DCMAKE_TOOLCHAIN_FILE=${toolchain_file}"
    "-DCMAKE_C_COMPILER=${wasi-sdk}/bin/clang"
    "-DCMAKE_CXX_COMPILER=${wasi-sdk}/bin/clang++"
    "-DCMAKE_AR=${wasi-sdk}/bin/llvm-ar"
    "-DCMAKE_RANLIB=${wasi-sdk}/bin/llvm-ranlib"
    "-DCMAKE_SYSROOT=${wasi-sdk}/share/wasi-sysroot"
    "-DCMAKE_FIND_ROOT_PATH_MODE_PROGRAM=NEVER"
    "-DCMAKE_FIND_ROOT_PATH_MODE_LIBRARY=ONLY"
    "-DCMAKE_FIND_ROOT_PATH_MODE_INCLUDE=ONLY"
    "-DCMAKE_FIND_ROOT_PATH_MODE_PACKAGE=ONLY"
    "-DCMAKE_C_COMPILER_WORKS=ON"
    "-DCMAKE_CXX_COMPILER_WORKS=ON"
  ];

  buildPhase = ''
    cmake --build . --target barretenberg.wasm --parallel
  '';
}
