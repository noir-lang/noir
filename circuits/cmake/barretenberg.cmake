# Here we Set up barretenberg as an ExternalProject
# - Point to its source and build directories
# - Construct its `configure` and `build` command lines
# - include its `src/` in `search path for includes
# - Depend on specific libraries from barretenberg
#
# If barretenberg's cmake files change, its configure and build are triggered
# If barretenberg's source files change, build is triggered

include(ExternalProject)

if (WASM)
    set(BBERG_BUILD_DIR ${BBERG_DIR}/build-wasm)
else()
    set(BBERG_BUILD_DIR ${BBERG_DIR}/build)
endif()

# If the OpenMP library is included via this option, propogate to ExternalProject configure
if (OpenMP_omp_LIBRARY)
    set(LIB_OMP_OPTION -DOpenMP_omp_LIBRARY=${OpenMP_omp_LIBRARY})
endif()

# Make sure barretenberg doesn't set its own WASI_SDK_PREFIX
if (WASI_SDK_PREFIX)
    set(WASI_SDK_OPTION -DWASI_SDK_PREFIX=${WASI_SDK_PREFIX})
endif()

# cmake configure cli args for ExternalProject
set(BBERG_CONFIGURE_ARGS -DTOOLCHAIN=${TOOLCHAIN} ${WASI_SDK_OPTION} ${LIB_OMP_OPTION} -DCI=${CI})

# Naming: Project: Barretenberg, Libraries: barretenberg, env
# Need BUILD_ALWAYS to ensure that barretenberg is automatically reconfigured when its CMake files change
# "Enabling this option forces the build step to always be run. This can be the easiest way to robustly
#  ensure that the external project's own build dependencies are evaluated rather than relying on the
#  default success timestamp-based method." - https://cmake.org/cmake/help/latest/module/ExternalProject.html
ExternalProject_Add(Barretenberg
    SOURCE_DIR ${BBERG_DIR}
    BINARY_DIR ${BBERG_BUILD_DIR} # build directory
    BUILD_ALWAYS TRUE
    UPDATE_COMMAND ""
    INSTALL_COMMAND ""
    CONFIGURE_COMMAND ${CMAKE_COMMAND} ${BBERG_CONFIGURE_ARGS} ..
    BUILD_COMMAND ${CMAKE_COMMAND} --build . --parallel --target barretenberg --target env)

include_directories(${BBERG_DIR}/src/aztec)

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