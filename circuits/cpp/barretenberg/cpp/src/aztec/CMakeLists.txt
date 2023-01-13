set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)

# Enable the following warnings project wide.
# If any compilation issues arise in the future, they should not be silenced here but rather in the 
# module's own CMakeLists.txt by adding conditional compilation flags like the following
# ```
#   if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
#     target_compile_options(crypto_blake3s_full_objects PRIVATE -Wno-error=shorten-64-to-32)
#   endif()
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

include_directories(${CMAKE_CURRENT_SOURCE_DIR})

# I feel this should be limited to ecc, however it's currently used in headers that go across libraries,
# and there currently isn't an easy way to inherit the DDISABLE_SHENANIGANS parameter.
if(DISABLE_ASM)
    message(STATUS "Using fallback non-assembly methods for field multiplications.")
    add_definitions(-DDISABLE_SHENANIGANS=1)
else()
    message(STATUS "Using optimized assembly for field arithmetic.")
endif()

add_subdirectory(env)
add_subdirectory(numeric)
add_subdirectory(srs)
add_subdirectory(ecc)
add_subdirectory(crypto)
add_subdirectory(polynomials)
add_subdirectory(proof_system)
add_subdirectory(transcript)
add_subdirectory(honk)
add_subdirectory(plonk)
add_subdirectory(stdlib)
add_subdirectory(rollup)
add_subdirectory(lagrange_base_gen)

if(BENCHMARKS)
    add_subdirectory(benchmark)
endif()

if(WASM)
    # Well, this is awkward. We can't build a wasm module by just linking to the libraries as that produces, nothing.
    # There are a couple of other ways to avoiding listing all the object files here and leveraging the dependency
    # tree, but they come with the problem that they will import the 'env' object files. We explicitly want to avoid
    # that as functions in 'env' should be implemented in JS itself.
    # It turns out that just explicitly telling the wasm module which object files to include was easiest.
    add_executable(
        barretenberg-step1.wasm
        $<TARGET_OBJECTS:srs_objects>
        $<TARGET_OBJECTS:numeric_objects>
        $<TARGET_OBJECTS:crypto_sha256_objects>
        $<TARGET_OBJECTS:crypto_aes128_objects>
        $<TARGET_OBJECTS:crypto_blake2s_objects>
        $<TARGET_OBJECTS:crypto_blake3s_objects>
        $<TARGET_OBJECTS:crypto_keccak_objects>
        $<TARGET_OBJECTS:crypto_schnorr_objects>
        $<TARGET_OBJECTS:crypto_pedersen_objects>
        $<TARGET_OBJECTS:ecc_objects>
        $<TARGET_OBJECTS:polynomials_objects>
        $<TARGET_OBJECTS:plonk_objects>
        $<TARGET_OBJECTS:stdlib_primitives_objects>
        $<TARGET_OBJECTS:stdlib_schnorr_objects>
        $<TARGET_OBJECTS:stdlib_pedersen_objects>
        $<TARGET_OBJECTS:stdlib_blake2s_objects>
        $<TARGET_OBJECTS:stdlib_blake3s_objects>
        $<TARGET_OBJECTS:rollup_proofs_inner_proof_data_objects>
        $<TARGET_OBJECTS:rollup_proofs_notes_objects>
        $<TARGET_OBJECTS:rollup_proofs_account_objects>
        $<TARGET_OBJECTS:rollup_proofs_join_split_objects>
        $<TARGET_OBJECTS:rollup_proofs_claim_objects>
    )

    # With binaryen installed, it seems its wasm backend optimiser gets invoked automatically.
    # Due to either a bug in the optimiser, or non-standards compliant c++ in crypto/aes, tests start failing with
    # -O3 level optimisations. We force down to -O2 for current workaround.
    # Presumably the -O3 when compiling the object files is fine as it's llvms IR optimiser.
    # The backend optimiser is presumably triggered after linking.
    target_link_options(
        barretenberg-step1.wasm
        PRIVATE
        -nostartfiles -O2 -Wl,--no-entry -Wl,--export-dynamic -Wl,--import-memory -Wl,--allow-undefined -Wl,--stack-first -Wl,-z,stack-size=1048576
    )

    add_custom_command(
        OUTPUT ${CMAKE_RUNTIME_OUTPUT_DIRECTORY}/barretenberg.wasm
        COMMAND wasm-opt ${CMAKE_RUNTIME_OUTPUT_DIRECTORY}/barretenberg-step1.wasm -O2 --asyncify -o ${CMAKE_RUNTIME_OUTPUT_DIRECTORY}/barretenberg.wasm
        DEPENDS barretenberg-step1.wasm
        VERBATIM
    )

    add_custom_target(
        barretenberg.wasm
        DEPENDS ${CMAKE_RUNTIME_OUTPUT_DIRECTORY}/barretenberg.wasm
    )

endif()
