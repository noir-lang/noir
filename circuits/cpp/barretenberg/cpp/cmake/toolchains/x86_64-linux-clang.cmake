# Specifically use clang 15 binaries if available. Otherwise fallback on default version.
find_program(CLANGXX_15 "clang++-15")
if (CLANGXX_15)
    set(CMAKE_C_COMPILER "clang-15")
    set(CMAKE_CXX_COMPILER "clang++-15")
else()
    set(CMAKE_C_COMPILER "clang")
    set(CMAKE_CXX_COMPILER "clang++")
endif()