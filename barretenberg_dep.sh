sudo apt-get update -y && sudo apt-get install -y \
  xz-utils \
  build-essential \
  curl \
  && curl -SL http://releases.llvm.org/9.0.0/clang+llvm-9.0.0-x86_64-linux-gnu-ubuntu-18.04.tar.xz | tar -xJC . \
  && mv clang+llvm-9.0.0-x86_64-linux-gnu-ubuntu-18.04 /usr/local/clang_9.0.0

export PATH="/usr/local/clang_9.0.0/bin:$PATH"
export LD_LIBRARY_PATH="/usr/local/clang_9.0.0/lib:$LD_LIBRARY_PATH"