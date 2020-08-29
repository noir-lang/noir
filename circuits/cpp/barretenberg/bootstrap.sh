# Build native.
mkdir -p build && cd build
cmake ..
cmake --build . --parallel
cd ..

# Install the webassembly toolchain and patch runtime.
if [ ! -d ./src/wasi-sdk-8.0 ]; then
  cd ./src
  if [[ "$OSTYPE" == "darwin"* ]]; then
      OS=macos
  elif [[ "$OSTYPE" == "linux-gnu" ]]; then
      OS=linux
  else
      echo "Unknown OS: $OSTYPE"
      exit 1
  fi
  curl -s -L https://github.com/CraneStation/wasi-sdk/releases/download/wasi-sdk-8/wasi-sdk-8.0-$OS.tar.gz | tar zxfv -
  sed -e $'213i\\\n#include "../../../../wasi/stdlib-hook.h"' -i.old ./wasi-sdk-8.0/share/wasi-sysroot/include/stdlib.h
  cd ..
fi

# Build WASM.
mkdir -p build-wasm && cd build-wasm
cmake -DWASM=ON ..
cmake --build . --parallel --target barretenberg.wasm
cd ..