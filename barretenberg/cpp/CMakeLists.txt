# barretenberg
# copyright 2019 Spilsbury Holdings Ltd

cmake_minimum_required(VERSION 3.24 FATAL_ERROR)

project(
    Barretenberg
    DESCRIPTION "BN254 elliptic curve library, and PLONK SNARK prover"
    VERSION 0.33.0 # x-release-please-version
    LANGUAGES CXX C
)
# Insert version into `bb` config file
configure_file(
    ${CMAKE_CURRENT_SOURCE_DIR}/src/barretenberg/bb/config.hpp.in
    ${CMAKE_CURRENT_SOURCE_DIR}/src/barretenberg/bb/config.hpp
    @ONLY
)

# Add doxygen build command
find_package(Doxygen)
if (DOXYGEN_FOUND)
add_custom_target(build_docs 
COMMAND ${DOXYGEN_EXECUTABLE} ${PROJECT_SOURCE_DIR}/docs/Doxyfile
WORKING_DIRECTORY ${PROJECT_SOURCE_DIR}
COMMENT "Generate documentation with Doxygen")
endif(DOXYGEN_FOUND)

option(DISABLE_ASM "Disable custom assembly" OFF)
option(DISABLE_ADX "Disable ADX assembly variant" OFF)
option(MULTITHREADING "Enable multi-threading" ON)
option(OMP_MULTITHREADING "Enable OMP multi-threading" OFF)
option(FUZZING "Build ONLY fuzzing harnesses" OFF)
option(DISABLE_TBB "Intel Thread Building Blocks" ON)
option(COVERAGE "Enable collecting coverage from tests" OFF)
option(ENABLE_ASAN "Address sanitizer for debugging tricky memory corruption" OFF)
option(ENABLE_HEAVY_TESTS "Enable heavy tests when collecting coverage" OFF)

if(CMAKE_SYSTEM_PROCESSOR MATCHES "aarch64" OR CMAKE_SYSTEM_PROCESSOR MATCHES "arm64")
    message(STATUS "Compiling for ARM.")
    set(ARM ON)
    set(DISABLE_ASM ON)
    set(DISABLE_ADX ON)
    set(RUN_HAVE_STD_REGEX 0)
    set(RUN_HAVE_POSIX_REGEX 0)
    set(DISABLE_TBB 0)
endif()

if(ENABLE_ASAN)
    add_compile_options(-fsanitize=address)
    add_link_options(-fsanitize=address)
    set(DISABLE_ASM ON)
endif()

if(FUZZING)
    add_definitions(-DFUZZING=1)

    if(DISABLE_CUSTOM_MUTATORS)
        add_definitions(-DDISABLE_CUSTOM_MUTATORS=1)
    endif()

    add_compile_options(-fsanitize=fuzzer)
    add_link_options(-fsanitize=fuzzer)

    set(MULTITHREADING OFF)
endif()

if(CMAKE_SYSTEM_PROCESSOR MATCHES "wasm32")
    message(STATUS "Compiling for WebAssembly.")
    set(WASM ON)
    set(DISABLE_ASM ON)
    set(OMP_MULTITHREADING OFF)
    set(DISABLE_TBB 1)
    add_compile_definitions(_WASI_EMULATED_PROCESS_CLOCKS=1)
endif()

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_EXTENSIONS ON)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED TRUE)
set(CMAKE_CXX_EXTENSIONS ON)

if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
    if(CMAKE_CXX_COMPILER_VERSION VERSION_LESS "14")
        message(WARNING "Clang <14 is not supported")
    endif()
elseif(CMAKE_CXX_COMPILER_ID MATCHES "GNU")
    if(CMAKE_CXX_COMPILER_VERSION VERSION_LESS "10")
        message(WARNING "GCC <10 is not supported")
    endif()
else()
    message(WARNING "Unsupported compiler, use Clang >14 or GCC >10")
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

    # Add profiling compile options and disable optimizations
    add_compile_options(-fprofile-instr-generate -fcoverage-mapping -O0)

    # Add a custom target for creating the report
    add_custom_target(create_full_coverage_report
        COMMAND "${CMAKE_SOURCE_DIR}/scripts/collect_coverage_information.sh" ${PROFDATA_EXECUTABLE} ${COV_EXECUTABLE}
        VERBATIM
    )
endif()

include(cmake/build.cmake)
include(GNUInstallDirs)
include(cmake/arch.cmake)
include(cmake/threading.cmake)
include(cmake/gtest.cmake)
include(cmake/benchmark.cmake)
include(cmake/module.cmake)
include(cmake/msgpack.cmake)
add_subdirectory(src)
if (ENABLE_ASAN AND NOT(FUZZING))
    find_program(LLVM_SYMBOLIZER_PATH NAMES llvm-symbolizer-16)
    if (NOT(LLVM_SYMBOLIZER_PATH))
        message(WARNING "LLVM symbolizer not found, so ASAN output will be limited")
    else()
        message(AUTHOR_WARNING "Run `export ASAN_SYMBOLIZER_PATH=\"${LLVM_SYMBOLIZER_PATH}\"` before running tests for better ASAN output (at which lines the bugs are).")
    endif()
endif()
