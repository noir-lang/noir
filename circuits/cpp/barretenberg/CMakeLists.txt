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

if(ARM)
    message(STATUS "Compiling for ARM.")
    set(DISABLE_ASM ON)
    set(DISABLE_ADX ON)
    set(RUN_HAVE_STD_REGEX 0)
    set(RUN_HAVE_POSIX_REGEX 0)
endif()

if(FUZZING)
    add_definitions(-DFUZZING=1)

    if(DISABLE_CUSTOM_MUTATORS)
        add_definitions(-DDISABLE_CUSTOM_MUTATORS=1)
    endif()

    set(SANITIZER_OPTIONS "")

    if(ADDRESS_SANITIZER)
        set(SANITIZER_OPTIONS ${SANITIZER_OPTIONS} -fsanitize=address)
    endif()

    if(UNDEFINED_BEHAVIOUR_SANITIZER)
        set(SANITIZER_OPTIONS ${SANITIZER_OPTIONS} -fsanitize=undefined  -fno-sanitize=alignment)
    endif()
    
    add_compile_options(-fsanitize=fuzzer-no-link ${SANITIZER_OPTIONS})

    set(WASM OFF)
    set(BENCHMARKS OFF)
    set(TESTING OFF)
endif()

if(WASM)
    message(STATUS "Compiling for WebAssembly.")
    set(DISABLE_ASM ON)
    set(MULTITHREADING OFF)
    set(BENCHMARKS OFF)
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
include(cmake/benchmark.cmake)
include(cmake/module.cmake)

add_subdirectory(src/aztec)