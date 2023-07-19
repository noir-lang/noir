# Here we Set up barretenberg as an ExternalProject
# - Point to its source and build directories
# - Construct its `configure` and `build` command lines
# - include its `src/` in `search path for includes
# - Depend on specific libraries from barretenberg
#
# If barretenberg's cmake files change, its configure and build are triggered
# If barretenberg's source files change, build is triggered

include(ExternalProject)

# Reference barretenberg artifacts (like library archives) via this dir:
if (WASM)
    set(BBERG_BUILD_DIR ${BBERG_DIR}/build-wasm)
    set(BBERG_TARGETS --target barretenberg --target env --target wasi --target barretenberg.wasm)
else()
    set(BBERG_BUILD_DIR ${BBERG_DIR}/build)
    set(BBERG_TARGETS --target barretenberg --target env --target wasi)
endif()

if(NOT CMAKE_BBERG_PRESET)
    set(CMAKE_BBERG_PRESET default)
endif()

# Naming: Project: Barretenberg, Libraries: barretenberg, env
# Need BUILD_ALWAYS to ensure that barretenberg is automatically reconfigured when its CMake files change
# "Enabling this option forces the build step to always be run. This can be the easiest way to robustly
#  ensure that the external project's own build dependencies are evaluated rather than relying on the
#  default success timestamp-based method." - https://cmake.org/cmake/help/latest/module/ExternalProject.html

ExternalProject_Add(Barretenberg
    SOURCE_DIR ${BBERG_DIR}
    BUILD_IN_SOURCE TRUE
    BUILD_ALWAYS TRUE
    UPDATE_COMMAND ""
    INSTALL_COMMAND ""
    CONFIGURE_COMMAND
        ${CMAKE_COMMAND}
        --preset ${CMAKE_BBERG_PRESET}
        -DCMAKE_CXX_FLAGS=${CMAKE_BBERG_CXX_FLAGS}
        -DMULTITHREADING=${MULTITHREADING}
        -DENABLE_ASAN=${ENABLE_ASAN}
        -DCMAKE_BUILD_TYPE=${CMAKE_BUILD_TYPE}
    BUILD_COMMAND
        ${CMAKE_COMMAND}
        --build
        --preset ${CMAKE_BBERG_PRESET}
        ${BBERG_TARGETS}
    # byproducts needed by ninja generator (not needed by make)
    BUILD_BYPRODUCTS
        ${BBERG_BUILD_DIR}/lib/libbarretenberg.a
        ${BBERG_BUILD_DIR}/lib/libenv.a
)

include_directories(${BBERG_DIR}/src)

# Add the imported barretenberg and env libraries, point to their library archives,
# and add a dependency of these libraries on the imported project
add_library(barretenberg STATIC IMPORTED)
set_target_properties(barretenberg PROPERTIES IMPORTED_LOCATION ${BBERG_BUILD_DIR}/lib/libbarretenberg.a)
add_dependencies(barretenberg Barretenberg)

# env is needed for logstr in native executables and wasm tests
# It is otherwise omitted from wasm to prevent use of C++ logstr instead of imported/Typescript
add_library(env STATIC IMPORTED)
set_target_properties(env PROPERTIES IMPORTED_LOCATION ${BBERG_BUILD_DIR}/lib/libenv.a)
add_dependencies(env Barretenberg)

# wasi is needed to initialize global statics and ensure we're following the reactor wasi pattern.
add_library(wasi STATIC IMPORTED)
set_target_properties(wasi PROPERTIES IMPORTED_LOCATION ${BBERG_BUILD_DIR}/lib/libwasi.a)
add_dependencies(wasi Barretenberg)
