# barretenberg
# copyright 2019 Spilsbry Holdings Ltd

cmake_minimum_required(VERSION 3.14)

option(DISABLE_ASM "Disable custom assembly" OFF)
option(DISABLE_ADX "Disable ADX assembly variant" OFF)
option(MULTITHREADING "Enable multi-threading" ON)
option(TESTING "Build tests" ON)
option(BENCHMARKS "Build benchmarks" ON)
option(PROFILING "Build profiling" ON)
option(FORCE_CLANG "force build with clang" OFF) # OFF is the default

if(ARM)
    set(DISABLE_ASM ON)
    set(RUN_HAVE_STD_REGEX 0)
    set(RUN_HAVE_POSIX_REGEX 0)
    set(CMAKE_TOOLCHAIN_FILE "./cmake/toolchain-arm64-linux-gcc.cmake")
endif()

if(WASM)
    message(STATUS "Compiling for WebAssembly.")
    set(DISABLE_ASM ON)
    set(MULTITHREADING OFF)
    set(BENCHMARKS OFF)
    set(PROFILING ON)
    set(CMAKE_TOOLCHAIN_FILE "./cmake/toolchain-wasm-linux-clang.cmake")
    set(WASI_SDK_PREFIX "${CMAKE_CURRENT_SOURCE_DIR}/src/wasi-sdk-8.0")
    add_compile_options(-fno-exceptions)
    add_definitions(-DBOOST_NO_EXCEPTIONS=1)
endif()

# On OS X we must use llvm from brew, not the hosts default.
if(APPLE)
    set(CMAKE_CXX_COMPILER "/usr/local/opt/llvm/bin/clang++")
    set(CMAKE_C_COMPILER "/usr/local/opt/llvm/bin/clang")
endif()
if(FORCE_CLANG)
    set(CMAKE_C_COMPILER "/usr/local/clang_9.0.0/bin/clang-9")
    set(CMAKE_CXX_COMPILER "/usr/local/clang_9.0.0/bin/clang++")
endif()

include(GNUInstallDirs)
include(cmake/OptimizeForArchitecture.cmake)
include(cmake/AddCompilerFlag.cmake)

set(PROJECT_VERSION 0.1.0)
project(Barretenberg
    DESCRIPTION "BN254 elliptic curve library, and PLONK SNARK prover"
    LANGUAGES CXX C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_EXTENSIONS ON)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED TRUE)
set(CMAKE_CXX_EXTENSIONS ON)

#OptimizeForArchitecture()
#list(FIND _available_vector_units_list bmi2 HAS_BMI2)

if(APPLE)
    if(CMAKE_C_COMPILER_ID MATCHES "Clang")
        set(OpenMP_C_FLAGS "-fopenmp")
        set(OpenMP_C_FLAGS_WORK "-fopenmp")
        set(OpenMP_C_LIB_NAMES "libomp")
        set(OpenMP_C_LIB_NAMES_WORK "libomp")
        set(OpenMP_libomp_LIBRARY "/usr/local/lib/libomp.a")
    endif()
    if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
        set(OpenMP_CXX_FLAGS "-fopenmp")
        set(OpenMP_CXX_FLAGS_WORK "-fopenmp")
        set(OpenMP_CXX_LIB_NAMES "libomp")
        set(OpenMP_CXX_LIB_NAMES_WORK "libomp")
        set(OpenMP_libomp_LIBRARY "/usr/local/lib/libomp.a")
    endif()
endif()

if(FORCE_CLANG)
    message(STATUS "Forcing use of clang compiler")
    if(CMAKE_C_COMPILER_ID MATCHES "Clang")
        set(OpenMP_C_FLAGS "-fopenmp")
        set(OpenMP_C_FLAGS_WORK "-fopenmp")
        set(OpenMP_C_LIB_NAMES "libomp")
        set(OpenMP_C_LIB_NAMES_WORK "libomp")
    endif()
    if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
        set(OpenMP_CXX_FLAGS "-fopenmp")
        set(OpenMP_CXX_FLAGS_WORK "-fopenmp")
        set(OpenMP_CXX_LIB_NAMES "libomp")
        set(OpenMP_CXX_LIB_NAMES_WORK "libomp")
    endif()
endif()

find_package(OpenMP)
if(MULTITHREADING AND OpenMP_CXX_FOUND)
    message(STATUS "Multithreading is enabled.")
    link_libraries(OpenMP::OpenMP_CXX)
else()
    message(STATUS "Multithreading is disabled.")
    add_definitions(-DNO_MULTITHREADING=1 -DBOOST_SP_NO_ATOMIC_ACCESS=1)
endif()

set(default_build_type "Release")
if(NOT CMAKE_BUILD_TYPE AND NOT CMAKE_CONFIGURATION_TYPES)
  message(STATUS "Setting build type to '${default_build_type}' as none was specified.")
  set(CMAKE_BUILD_TYPE "${default_build_type}" CACHE STRING "Choose the type of build." FORCE)
  # Set the possible values of build type for cmake-gui
  set_property(CACHE CMAKE_BUILD_TYPE PROPERTY STRINGS "Debug" "Release" "MinSizeRel" "RelWithDebInfo")
endif()

if((NOT DISABLE_ASM) AND (HAS_BMI2 EQUAL -1))
    message(STATUS "Cannot detect BMI2 instruction set.")
    set(DISABLE_ASM ON)
endif()

if(DISABLE_ASM)
    message(STATUS "Using fallback non-assembly methods for field multiplications.")
    add_definitions(-DDISABLE_SHENANIGANS=1)
else()
    message(STATUS "Using optimized assembly for field arithmetic.")
endif()

if(WIN32)
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wno-float-conversion -Wimplicit-fallthrough=0")
endif()

if(DISABLE_ADX)
    message(STATUS "Disabling ADX assembly variant.")
    add_definitions(-DDISABLE_ADX=1)
endif()

set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS}  -Wno-deprecated")

if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -fconstexpr-steps=100000000")
endif()

if(CMAKE_CXX_COMPILER_ID MATCHES "GNU")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wno-deprecated-copy -fconstexpr-ops-limit=100000000")
endif()

if(NOT DEFINED PIPPENGER_BLOCK_SIZE)
    set(PIPPENGER_BLOCK_SIZE "20" CACHE STRING "maximum multi-exponentiation size")
endif()
message(STATUS "Setting pippenger block size to " ${PIPPENGER_BLOCK_SIZE})
add_definitions(-DPIPPENGER_BLOCK_SIZE=${PIPPENGER_BLOCK_SIZE})

add_definitions(-DBOOST_SPIRIT_X3_NO_FILESYSTEM=1)

set(private_include_dir ${PROJECT_SOURCE_DIR}/src)

if(WIN32)
    set(private_os_libs "wsock32 ws2_32")
endif()

add_subdirectory(src)

if(TESTING)
    message(STATUS "Configuring tests and benchmarks")
    enable_testing()
    add_subdirectory(test)
endif()

if(BENCHMARKS)
    add_subdirectory(test/benchmarks)
endif()