set(CMAKE_SYSTEM_NAME Darwin)
set(CMAKE_SYSTEM_PROCESSOR x86_64)

if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
    # Clang allows us to cross compile on Mac
    # so we explicitly specify the arch to the compiler
    # If you just select the x86_64 toolchain and are on an
    # M1/arm64 mac, it will compile for arm64.
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -arch x86_64")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -arch x86_64")
endif()
