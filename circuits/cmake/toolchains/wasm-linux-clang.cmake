# Cmake toolchain description file for the Makefile

# This is arbitrary, AFAIK, for now.
cmake_minimum_required(VERSION 3.4.0)

set(WASM ON)
set(CMAKE_SYSTEM_NAME Generic)
set(CMAKE_SYSTEM_VERSION 1)
set(CMAKE_SYSTEM_PROCESSOR wasm32)
set(triple wasm32-wasi)

if (NOT WASI_SDK_PREFIX)
  # can be set by a dependent project
  set(WASI_SDK_PREFIX "${CMAKE_CURRENT_SOURCE_DIR}/src/wasi-sdk-12.0")
endif()
set(CMAKE_C_COMPILER ${WASI_SDK_PREFIX}/bin/clang)
set(CMAKE_CXX_COMPILER ${WASI_SDK_PREFIX}/bin/clang++)
set(CMAKE_AR ${WASI_SDK_PREFIX}/bin/llvm-ar CACHE STRING "wasi-sdk build")
set(CMAKE_RANLIB ${WASI_SDK_PREFIX}/bin/llvm-ranlib CACHE STRING "wasi-sdk build")
set(CMAKE_C_COMPILER_TARGET ${triple} CACHE STRING "wasi-sdk build")
set(CMAKE_CXX_COMPILER_TARGET ${triple} CACHE STRING "wasi-sdk build")
#set(CMAKE_EXE_LINKER_FLAGS "-Wl,--no-threads" CACHE STRING "wasi-sdk build")

set(CMAKE_SYSROOT ${WASI_SDK_PREFIX}/share/wasi-sysroot CACHE STRING "wasi-sdk build")
set(CMAKE_STAGING_PREFIX ${WASI_SDK_PREFIX}/share/wasi-sysroot CACHE STRING "wasi-sdk build")

# Don't look in the sysroot for executables to run during the build
set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
# Only look in the sysroot (not in the host paths) for the rest
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_PACKAGE ONLY)

# Some other hacks
set(CMAKE_C_COMPILER_WORKS ON)
set(CMAKE_CXX_COMPILER_WORKS ON)

add_definitions(-D_WASI_EMULATED_PROCESS_CLOCKS=1)