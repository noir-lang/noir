# aztec-connect-cpp
# copyright 2019 Spilsbury Holdings Ltd

cmake_minimum_required(VERSION 3.16)

# Get the full path to barretenberg. This is helpful because the required
# relative path changes based on where in cmake the path is used.
# `BBERG_DIR` must be set before toolchain.cmake is imported because
# `BBERG_DIR` is used in toolchain.cmake to determine `WASI_SDK_PREFIX`
get_filename_component(BBERG_DIR ../barretenberg/cpp
                       REALPATH BASE_DIR "${CMAKE_BINARY_DIR}")

include(cmake/toolchain.cmake)

set(PROJECT_VERSION 0.1.0)
project(Aztec3Circuits
    DESCRIPTION "Project containing Aztec3 Circuits Infrastucture in C++."
    LANGUAGES CXX C)

# include barretenberg as ExternalProject
include(cmake/barretenberg.cmake)

option(DISABLE_ASM "Disable custom assembly" OFF)
option(DISABLE_ADX "Disable ADX assembly variant" OFF)
option(MULTITHREADING "Enable multi-threading" ON)
option(TESTING "Build tests" ON)

if(ARM)
    message(STATUS "Compiling for ARM.")
    set(DISABLE_ASM ON)
    set(DISABLE_ADX ON)
    set(RUN_HAVE_STD_REGEX 0)
    set(RUN_HAVE_POSIX_REGEX 0)
endif()

if(WASM)
    message(STATUS "Compiling for WebAssembly.")
    set(DISABLE_ASM ON)
    set(MULTITHREADING OFF)
endif()

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_EXTENSIONS ON)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED TRUE)
set(CMAKE_CXX_EXTENSIONS ON)

include(cmake/build.cmake)
include(cmake/arch.cmake)
include(cmake/threading.cmake)
include(cmake/gtest.cmake)
include(cmake/module.cmake)

add_subdirectory(src)