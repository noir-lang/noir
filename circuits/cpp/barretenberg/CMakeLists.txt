# barretenberg
# copyright 2019 Spilsbury Holdings Ltd

cmake_minimum_required(VERSION 3.16)

include(cmake/toolchain.cmake)

set(PROJECT_VERSION 0.1.0)
project(Barretenberg
    DESCRIPTION "BN254 elliptic curve library, and PLONK SNARK prover"
    LANGUAGES CXX C)

option(DISABLE_ASM "Disable custom assembly" OFF)
option(DISABLE_ADX "Disable ADX assembly variant" OFF)
option(MULTITHREADING "Enable multi-threading" ON)
option(TESTING "Build tests" ON)
option(BENCHMARKS "Build benchmarks" ON)
option(PROFILING "Build profiling" ON)
option(FORCE_CLANG "Force build with clang" OFF)

if(ARM)
    message(STATUS "Compiling for ARM.")
    set(DISABLE_ASM ON)
    set(RUN_HAVE_STD_REGEX 0)
    set(RUN_HAVE_POSIX_REGEX 0)
endif()

if(WASM)
    message(STATUS "Compiling for WebAssembly.")
    set(DISABLE_ASM ON)
    set(MULTITHREADING OFF)
    set(BENCHMARKS OFF)
    set(PROFILING OFF)
endif()

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_EXTENSIONS ON)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED TRUE)
set(CMAKE_CXX_EXTENSIONS ON)

include(GNUInstallDirs)
include(cmake/arch.cmake)
include(cmake/threading.cmake)
include(cmake/gtest.cmake)
include(cmake/build.cmake)

add_subdirectory(src)

#if(TESTING)
#    message(STATUS "Configuring tests and benchmarks")
#    enable_testing()
#    add_subdirectory(test)
#endif()
#
#if(BENCHMARKS)
#    add_subdirectory(test/benchmarks)
#endif()