if(APPLE)
    set(CMAKE_CXX_COMPILER "/usr/local/opt/llvm/bin/clang++")
    set(CMAKE_C_COMPILER "/usr/local/opt/llvm/bin/clang")
endif()

if(LINUX_CLANG)
    include("./cmake/toolchains/x86_64-linux-clang.cmake")
endif()

if(ARM)
    include("./cmake/toolchains/arm64-linux-gcc.cmake")
endif()

if(WASM)
    include("./cmake/toolchains/wasm-linux-clang.cmake")
endif()