set(CMAKE_SYSTEM_NAME Darwin)
set(CMAKE_SYSTEM_PROCESSOR aarch64)

if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
    # Clang allows us to cross compile on Mac
    # so we explicitly specify the arch to the compiler
    # If you just select the arch toolchain and are on an
    # x86_64, it will compile for x86_64 mac.
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -arch arm64")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -arch arm64")
endif()