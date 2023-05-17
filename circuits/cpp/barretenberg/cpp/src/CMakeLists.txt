set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)

# Enable the following warnings project wide.
# If any compilation issues arise in the future, they should not be silenced here but rather in the
# module's own CMakeLists.txt by adding conditional compilation flags like the following
# ```
# if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
# target_compile_options(crypto_blake3s_full_objects PRIVATE -Wno-error=shorten-64-to-32)
# endif()
# ```
# Specifying `-Wno-${ERROR_NAME}` will silence the error completely.
# To preserve the warning, but prevent them from causing the build to fail,
# use the flag `-Wno-error=${ERROR_NAME}`
add_compile_options(-Werror -Wall -Wextra -Wconversion -Wsign-conversion -Wfatal-errors)

if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
    add_compile_options(-fconstexpr-steps=100000000)

    if(MEMORY_CHECKS)
        message(STATUS "Compiling with memory checks.")
        set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -fsanitize=address")
    endif()
endif()

if(CMAKE_CXX_COMPILER_ID MATCHES "GNU")
    add_compile_options(-fconstexpr-ops-limit=100000000)
endif()

include_directories(${CMAKE_CURRENT_SOURCE_DIR} ${CMAKE_CURRENT_SOURCE_DIR}/msgpack-c/include)

# I feel this should be limited to ecc, however it's currently used in headers that go across libraries,
# and there currently isn't an easy way to inherit the DDISABLE_SHENANIGANS parameter.
if(DISABLE_ASM)
    message(STATUS "Using fallback non-assembly methods for field multiplications.")
    add_definitions(-DDISABLE_SHENANIGANS=1)
else()
    message(STATUS "Using optimized assembly for field arithmetic.")
endif()

add_subdirectory(barretenberg/common)
add_subdirectory(barretenberg/env)
add_subdirectory(barretenberg/numeric)
add_subdirectory(barretenberg/srs)
add_subdirectory(barretenberg/ecc)
add_subdirectory(barretenberg/crypto)
add_subdirectory(barretenberg/polynomials)
add_subdirectory(barretenberg/proof_system)
add_subdirectory(barretenberg/transcript)
add_subdirectory(barretenberg/honk)
add_subdirectory(barretenberg/plonk)
add_subdirectory(barretenberg/stdlib)
add_subdirectory(barretenberg/join_split_example)
add_subdirectory(barretenberg/dsl)
add_subdirectory(barretenberg/serialize)
add_subdirectory(barretenberg/solidity_helpers)

if(BENCHMARKS)
    add_subdirectory(barretenberg/benchmark)
endif()

include(GNUInstallDirs)

if(WASM)
    # Well, this is awkward. We can't build a wasm module by just linking to the libraries as that produces, nothing.
    # There are a couple of other ways to avoiding listing all the object files here and leveraging the dependency
    # tree, but they come with the problem that they will import the 'env' object files. We explicitly want to avoid
    # that as functions in 'env' should be implemented in JS itself.
    # It turns out that just explicitly telling the wasm module which object files to include was easiest.
    add_executable(
        barretenberg.wasm
        $<TARGET_OBJECTS:transcript_objects>
        $<TARGET_OBJECTS:srs_objects>
        $<TARGET_OBJECTS:numeric_objects>
        $<TARGET_OBJECTS:crypto_sha256_objects>
        $<TARGET_OBJECTS:crypto_aes128_objects>
        $<TARGET_OBJECTS:crypto_blake2s_objects>
        $<TARGET_OBJECTS:crypto_blake3s_objects>
        $<TARGET_OBJECTS:crypto_keccak_objects>
        $<TARGET_OBJECTS:crypto_ecdsa_objects>
        $<TARGET_OBJECTS:crypto_schnorr_objects>
        $<TARGET_OBJECTS:crypto_generators_objects>
        $<TARGET_OBJECTS:crypto_pedersen_hash_objects>
        $<TARGET_OBJECTS:crypto_pedersen_commitment_objects>
        $<TARGET_OBJECTS:ecc_objects>
        $<TARGET_OBJECTS:polynomials_objects>
        $<TARGET_OBJECTS:plonk_objects>
        $<TARGET_OBJECTS:honk_objects>
        $<TARGET_OBJECTS:proof_system_objects>
        $<TARGET_OBJECTS:stdlib_primitives_objects>
        $<TARGET_OBJECTS:stdlib_schnorr_objects>
        $<TARGET_OBJECTS:stdlib_pedersen_hash_objects>
        $<TARGET_OBJECTS:stdlib_pedersen_commitment_objects>
        $<TARGET_OBJECTS:stdlib_blake2s_objects>
        $<TARGET_OBJECTS:stdlib_blake3s_objects>
        $<TARGET_OBJECTS:stdlib_keccak_objects>
        $<TARGET_OBJECTS:stdlib_sha256_objects>
        $<TARGET_OBJECTS:stdlib_aes128_objects>
        $<TARGET_OBJECTS:stdlib_merkle_tree_objects>
        $<TARGET_OBJECTS:acir_format_objects>
        $<TARGET_OBJECTS:acir_proofs_objects>
        $<TARGET_OBJECTS:stdlib_sha256_objects>
        $<TARGET_OBJECTS:stdlib_aes128_objects>
        $<TARGET_OBJECTS:stdlib_merkle_tree_objects>
    )

    # With binaryen installed, it seems its wasm backend optimiser gets invoked automatically.
    # Due to either a bug in the optimiser, or non-standards compliant c++ in crypto/aes, tests start failing with
    # -O3 level optimisations. We force down to -O2 for current workaround.
    # Presumably the -O3 when compiling the object files is fine as it's llvms IR optimiser.
    # The backend optimiser is presumably triggered after linking.
    target_link_options(
        barretenberg.wasm
        PRIVATE
        -nostartfiles -O2 -Wl,--no-entry -Wl,--export-dynamic -Wl,--import-memory -Wl,--allow-undefined -Wl,--stack-first -Wl,-z,stack-size=1048576
    )

    # Repeat the above but for the smaller primitives.wasm
    # Used in packages where we don't need the full contents of barretenberg
    add_executable(
        primitives.wasm
        $<TARGET_OBJECTS:srs_objects>
        $<TARGET_OBJECTS:numeric_objects>
        $<TARGET_OBJECTS:crypto_sha256_objects>
        $<TARGET_OBJECTS:crypto_aes128_objects>
        $<TARGET_OBJECTS:crypto_blake2s_objects>
        $<TARGET_OBJECTS:crypto_blake3s_objects>
        $<TARGET_OBJECTS:crypto_generators_objects>
        $<TARGET_OBJECTS:crypto_keccak_objects>
        $<TARGET_OBJECTS:crypto_ecdsa_objects>
        $<TARGET_OBJECTS:crypto_schnorr_objects>
        $<TARGET_OBJECTS:crypto_pedersen_hash_objects>
        $<TARGET_OBJECTS:crypto_pedersen_commitment_objects>
        $<TARGET_OBJECTS:ecc_objects>
    )

    target_link_options(
        primitives.wasm
        PRIVATE
        -nostartfiles -O2 -Wl,--no-entry -Wl,--export-dynamic -Wl,--import-memory -Wl,--allow-undefined -Wl,--stack-first -Wl,-z,stack-size=1048576
    )

    # TODO(blaine): Figure out how to Asyncify the wasm output.
    # Binaryen's Asyncify transform produces wasm that has too many local variables to run in a WebAssembly
    # instance. This likely would be "solved" by enabling the optimizations to reduce the number of locals,
    # but using any optimization level results in a wasm file that takes an unusable amount of time to solve the
    # most simple prood.
    # find_program(WASM_OPT wasm-opt)

    # if(NOT WASM_OPT)
    #     message(FATAL_ERROR "wasm-opt executable not found. Please install binaryen.")
    # endif()

    # add_custom_command(
    #     TARGET barretenberg.wasm
    #     POST_BUILD
    #     COMMAND wasm-opt "$<TARGET_FILE:barretenberg.wasm>" -O0 --asyncify -o "$<TARGET_FILE:barretenberg.wasm>"
    #     VERBATIM
    # )

    add_custom_command(
        TARGET primitives.wasm
        POST_BUILD
        COMMAND wasm-opt "$<TARGET_FILE:primitives.wasm>" -O2 -o "$<TARGET_FILE:primitives.wasm>"
        VERBATIM
    )

    if(INSTALL_BARRETENBERG)
        install(TARGETS barretenberg.wasm DESTINATION ${CMAKE_INSTALL_BINDIR})
    endif()

    # For use when compiling dependent cpp projects for WASM
    message(STATUS "Compiling all-in-one barretenberg WASM archive")
    add_library(
        barretenberg
        STATIC
        $<TARGET_OBJECTS:transcript_objects>
        $<TARGET_OBJECTS:srs_objects>
        $<TARGET_OBJECTS:numeric_objects>
        $<TARGET_OBJECTS:crypto_sha256_objects>
        $<TARGET_OBJECTS:crypto_aes128_objects>
        $<TARGET_OBJECTS:crypto_blake2s_objects>
        $<TARGET_OBJECTS:crypto_blake3s_objects>
        $<TARGET_OBJECTS:crypto_keccak_objects>
        $<TARGET_OBJECTS:crypto_ecdsa_objects>
        $<TARGET_OBJECTS:crypto_schnorr_objects>
        $<TARGET_OBJECTS:crypto_generators_objects>
        $<TARGET_OBJECTS:crypto_pedersen_hash_objects>
        $<TARGET_OBJECTS:crypto_pedersen_commitment_objects>
        $<TARGET_OBJECTS:ecc_objects>
        $<TARGET_OBJECTS:polynomials_objects>
        $<TARGET_OBJECTS:plonk_objects>
        $<TARGET_OBJECTS:honk_objects>
        $<TARGET_OBJECTS:proof_system_objects>
        $<TARGET_OBJECTS:stdlib_primitives_objects>
        $<TARGET_OBJECTS:stdlib_schnorr_objects>
        $<TARGET_OBJECTS:stdlib_pedersen_hash_objects>
        $<TARGET_OBJECTS:stdlib_pedersen_commitment_objects>
        $<TARGET_OBJECTS:stdlib_blake2s_objects>
        $<TARGET_OBJECTS:stdlib_blake3s_objects>
        $<TARGET_OBJECTS:stdlib_keccak_objects>
        $<TARGET_OBJECTS:stdlib_sha256_objects>
        $<TARGET_OBJECTS:stdlib_aes128_objects>
        $<TARGET_OBJECTS:stdlib_merkle_tree_objects>
        $<TARGET_OBJECTS:acir_format_objects>
        $<TARGET_OBJECTS:acir_proofs_objects>
    )
else()
    # For use when compiling dependent cpp projects
    message(STATUS "Compiling all-in-one barretenberg archive")
    add_library(
        barretenberg
        STATIC
        $<TARGET_OBJECTS:transcript_objects>
        $<TARGET_OBJECTS:srs_objects>
        $<TARGET_OBJECTS:numeric_objects>
        $<TARGET_OBJECTS:crypto_sha256_objects>
        $<TARGET_OBJECTS:crypto_aes128_objects>
        $<TARGET_OBJECTS:crypto_blake2s_objects>
        $<TARGET_OBJECTS:crypto_blake3s_objects>
        $<TARGET_OBJECTS:crypto_keccak_objects>
        $<TARGET_OBJECTS:crypto_ecdsa_objects>
        $<TARGET_OBJECTS:crypto_schnorr_objects>
        $<TARGET_OBJECTS:crypto_generators_objects>
        $<TARGET_OBJECTS:crypto_pedersen_hash_objects>
        $<TARGET_OBJECTS:crypto_pedersen_commitment_objects>
        $<TARGET_OBJECTS:ecc_objects>
        $<TARGET_OBJECTS:polynomials_objects>
        $<TARGET_OBJECTS:plonk_objects>
        $<TARGET_OBJECTS:honk_objects>
        $<TARGET_OBJECTS:proof_system_objects>
        $<TARGET_OBJECTS:stdlib_primitives_objects>
        $<TARGET_OBJECTS:stdlib_schnorr_objects>
        $<TARGET_OBJECTS:stdlib_pedersen_hash_objects>
        $<TARGET_OBJECTS:stdlib_pedersen_commitment_objects>
        $<TARGET_OBJECTS:stdlib_blake2s_objects>
        $<TARGET_OBJECTS:stdlib_blake3s_objects>
        $<TARGET_OBJECTS:stdlib_keccak_objects>
        $<TARGET_OBJECTS:stdlib_sha256_objects>
        $<TARGET_OBJECTS:stdlib_aes128_objects>
        $<TARGET_OBJECTS:stdlib_merkle_tree_objects>
        $<TARGET_OBJECTS:acir_format_objects>
        $<TARGET_OBJECTS:acir_proofs_objects>
        $<TARGET_OBJECTS:env_objects>
    )

    if(INSTALL_BARRETENBERG)
        # The `install` function takes targets to install in different destinations on the system.
        install(
            TARGETS barretenberg barretenberg_headers
            # We also give it an optional export name in case something wants to target the install.
            EXPORT barretenbergTargets
            # The ARCHIVE output signifies static libraries that should be installed
            # and we use the GNUInstallDirs location to install into the standard system library location
            ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
            # The FILE_SET output is used instead of PUBLIC_HEADER & PRIVATE_HEADER outputs because
            # our headers don't have a clear delineation between public & private, but we still use
            # the GNUInstallDirs location to install into the standard system header location
            FILE_SET HEADERS DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
        )

        set(pkg_config "${PROJECT_BINARY_DIR}/barretenberg.pc")

        configure_file(
            "${PROJECT_SOURCE_DIR}/cmake/barretenberg.pc.in"
            "${pkg_config}"
            @ONLY
        )

        install(
            FILES "${pkg_config}"
            DESTINATION "${CMAKE_INSTALL_LIBDIR}/pkgconfig"
        )
    endif()
endif()
