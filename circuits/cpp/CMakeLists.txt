# aztec3 circuits package
# copyright 2019 Spilsbury Holdings Ltd

cmake_minimum_required(VERSION 3.24)

# Get the full path to barretenberg. This is helpful because the required
# relative path changes based on where in cmake the path is used.
# `BBERG_DIR` must be set before toolchain.cmake is imported because
# `BBERG_DIR` is used in toolchain.cmake to determine `WASI_SDK_PREFIX`
get_filename_component(BBERG_DIR ../barretenberg/cpp
                       REALPATH BASE_DIR "${CMAKE_BINARY_DIR}")

project(
    Aztec3Circuits
    DESCRIPTION "Project containing Aztec3 Circuits Infrastucture in C++."
    VERSION 0.1.0
    LANGUAGES CXX C
)

option(DISABLE_ASM "Disable custom assembly" OFF)
option(DISABLE_ADX "Disable ADX assembly variant" OFF)
option(MULTITHREADING "Enable multi-threading" ON)
option(TESTING "Build tests" ON)
option(ENABLE_ASAN "Address sanitizer for debugging tricky memory corruption" OFF)
option(BENCHMARKS "Build benchmarks" ON)
option(FUZZING "Build fuzzing harnesses" OFF)
option(DISABLE_TBB "Intel Thread Building Blocks" ON)
option(COVERAGE "Enable collecting coverage from tests" OFF)
option(ENABLE_HEAVY_TESTS "Enable heavy tests when collecting coverage" OFF)

message(STATUS "Building barretenberg for UltraPlonk Composer.")

if(ENABLE_ASAN)
    add_compile_options(-fsanitize=address)
    add_link_options(-fsanitize=address)
endif()

if(CMAKE_SYSTEM_PROCESSOR MATCHES "aarch64" OR CMAKE_SYSTEM_PROCESSOR MATCHES "arm64")
    message(STATUS "Compiling for ARM.")
    set(ARM ON)
    set(DISABLE_ASM ON)
    set(DISABLE_ADX ON)
    set(RUN_HAVE_STD_REGEX 0)
    set(RUN_HAVE_POSIX_REGEX 0)
    set(DISABLE_TBB 0)
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
    set(MULTITHREADING OFF)
    set(TESTING OFF)
endif()

if(CMAKE_SYSTEM_PROCESSOR MATCHES "wasm32")
    message(STATUS "Compiling for WebAssembly.")
    set(WASM ON)
    set(DISABLE_ASM ON)
    set(MULTITHREADING OFF)
    set(BENCHMARKS OFF)
    set(DISABLE_TBB 1)
    add_compile_definitions(_WASI_EMULATED_PROCESS_CLOCKS=1)
    # TODO resolve how we can all get on the same generator counts
    # Wasm (currently) takes less generators
    set(CMAKE_BBERG_CXX_FLAGS "-DBARRETENBERG_CRYPTO_GENERATOR_PARAMETERS_HACK=64,32,16,64")
else()
    # Native can take more generators
    set(CMAKE_BBERG_CXX_FLAGS "-DBARRETENBERG_CRYPTO_GENERATOR_PARAMETERS_HACK=2048,32,128,2048")
endif()

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_EXTENSIONS ON)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED TRUE)
set(CMAKE_CXX_EXTENSIONS ON)

if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
    if (CMAKE_CXX_COMPILER_VERSION VERSION_LESS "10")
        message(WARNING "Clang <10 is not supported")
    endif()
elseif(CMAKE_CXX_COMPILER_ID MATCHES "GNU")
    if (CMAKE_CXX_COMPILER_VERSION VERSION_LESS "10")
        message(WARNING "GCC <10 is not supported")
    endif()
else()
    message(WARNING "Unsuported compiler, use Clang >10 or GCC >10")
endif()

if(COVERAGE)

    # We've only set up LLVM coverage
    if(NOT(CMAKE_CXX_COMPILER_ID MATCHES "Clang"))
        message(FATAL_ERROR "Creating coverage is only available for clang")
    endif()

    # Get major clang version
    string(REPLACE "." ";" VERSION_LIST ${CMAKE_CXX_COMPILER_VERSION})
    list(GET VERSION_LIST 0 CLANG_VERSION_MAJOR)

    # Find llvm-profdata
    set(PROFDATA_EXECUTABLE_NAME "llvm-profdata-${CLANG_VERSION_MAJOR}")
    find_program(PROFDATA_EXECUTABLE ${PROFDATA_EXECUTABLE_NAME})
    if(PROFDATA_EXECUTABLE MATCHES "NOTFOUND")
        message(FATAL_ERROR "Couldn't find ${PROFDATA_EXECUTABLE_NAME}")
    endif()

    # Find llvm-cov
    set(COV_EXECUTABLE_NAME "llvm-cov-${CLANG_VERSION_MAJOR}")
    find_program(COV_EXECUTABLE ${COV_EXECUTABLE_NAME})
    if(COV_EXECUTABLE MATCHES "NOTFOUND")
        message(FATAL_ERROR "Couldn't find ${COV_EXECUTABLE_NAME}")
    endif()

    # Add profiling compile options and disable optimisations
    add_compile_options(-fprofile-instr-generate -fcoverage-mapping -O0)

    # Add a custom target for creating the report
    add_custom_target(create_full_coverage_report
    COMMAND "${CMAKE_SOURCE_DIR}/scripts/collect_coverage_information.sh" ${PROFDATA_EXECUTABLE} ${COV_EXECUTABLE}
    VERBATIM
    )
endif()

# include barretenberg as ExternalProject
# (needs WASM to be set already)
include(cmake/barretenberg.cmake)

include(cmake/build.cmake)
include(GNUInstallDirs)
include(cmake/arch.cmake)
include(cmake/threading.cmake)
include(cmake/gtest.cmake)
include(cmake/benchmark.cmake)
include(cmake/module.cmake)
add_subdirectory(src)
