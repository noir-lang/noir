{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [ 
    which
    git
    pkg-config
    cmake
  ];
  
  buildInputs = with pkgs; [ 
    llvmPackages.openmp
    openssl
    rustup
  ];

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
    pkgs.llvmPackages.openmp
    pkgs.openssl
  ];
  
  shellHook = ''
    echo 🧪 NIX_CFLAGS_COMPILE=$NIX_CFLAGS_COMPILE
    echo 🧪 NIX_LDFLAGS=$NIX_LDFLAGS
    echo 🧪 LD_LIBRARY_PATH=$LD_LIBRARY_PATH
    echo 🧪 CPATH=$CPATH
    echo 🧪 $CC $AR $CXX $LD
    echo 🧪 $(which $CC) 
    echo 🧪 $(which $AR) 
    echo 🧪 $(which $CXX) 
    echo 🧪 $(which $LD)
    echo 🧪 $(which pkg-config)
    echo 🧪 pkg-config --list-all ↩️
    pkg-config --list-all
    echo ⌛
  '';

}
