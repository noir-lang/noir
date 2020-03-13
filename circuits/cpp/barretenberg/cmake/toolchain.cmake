if(APPLE)
    set(CMAKE_CXX_COMPILER "/usr/local/opt/llvm/bin/clang++")
    set(CMAKE_C_COMPILER "/usr/local/opt/llvm/bin/clang")
endif()

if(FORCE_CLANG)
    set(CMAKE_C_COMPILER "/usr/local/clang_9.0.0/bin/clang-9")
    set(CMAKE_CXX_COMPILER "/usr/local/clang_9.0.0/bin/clang++")
endif()

if(ARM)
    include("./cmake/toolchains/arm64-linux-gcc.cmake")
endif()

if(WASM)
    include("./cmake/toolchains/wasm-linux-clang.cmake")
endif()