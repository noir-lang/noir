include(ExternalProject)

# External project: Download cvc5 from GitHub
set(CVC5_PREFIX "${CMAKE_BINARY_DIR}/_deps/cvc5")
set(CVC5_BUILD "${CVC5_PREFIX}/src/cvc5-build")
set(CVC5_LIB "${CVC5_BUILD}/lib/libcvc5.so")
set(CVC5_INCLUDE "${CVC5_BUILD}/include")

ExternalProject_Add(
    cvc5
    PREFIX ${CVC5_PREFIX}
    GIT_REPOSITORY "https://github.com/cvc5/cvc5.git"
    GIT_TAG main
    BUILD_IN_SOURCE YES
    CONFIGURE_COMMAND ${SHELL} ./configure.sh production --auto-download --cocoa --cryptominisat --kissat -DCMAKE_C_COMPILER=/usr/bin/clang -DCMAKE_CXX_COMPILER=/usr/bin/clang++ --prefix=${CVC5_BUILD}
    BUILD_COMMAND make -C build
    INSTALL_COMMAND make -C build install
    UPDATE_COMMAND ""     # No update step
    # needed by ninja
    # See https://stackoverflow.com/questions/48142082/cmake-externalproject-add-project-not-building-before-targets-that-depend-on-it
    BUILD_BYPRODUCTS ${CVC5_LIB} ${CVC5_INCLUDE}
)

add_library(cvc5-lib SHARED IMPORTED)
add_dependencies(cvc5-lib cvc5)
include_directories(${CVC5_INCLUDE})
set_target_properties(cvc5-lib PROPERTIES IMPORTED_LOCATION ${CVC5_LIB})

barretenberg_module(smt_verification common proof_system stdlib_primitives stdlib_sha256 circuit_checker transcript plonk cvc5-lib)
# We have no easy way to add a dependency to an external target, we list the built targets explicit. Could be cleaner.
add_dependencies(smt_verification cvc5)
add_dependencies(smt_verification_objects cvc5)
add_dependencies(smt_verification_tests cvc5)
add_dependencies(smt_verification_test_objects cvc5)