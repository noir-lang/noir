if(APPLE)
    include("./cmake/toolchains/x86_64-apple-clang.cmake")
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