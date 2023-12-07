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
add_compile_options(-Werror -Wall -Wextra -Wconversion -Wsign-conversion)

if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
    add_compile_options(-fcolor-diagnostics -fconstexpr-steps=100000000)

    if(MEMORY_CHECKS)
        message(STATUS "Compiling with memory checks.")
        set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -fsanitize=address")
    endif()
endif()

if(CMAKE_CXX_COMPILER_ID MATCHES "GNU")
    add_compile_options(-fconstexpr-ops-limit=100000000)
endif()

# We enable -O1 level optimsations, even when compiling debug wasm, otherwise we get "local count too large" at runtime.
# We prioritize reducing size of final artifacts in release with -Oz.
if(WASM)
    set(CMAKE_CXX_FLAGS_DEBUG "-O1 -g")
    set(CMAKE_C_FLAGS_DEBUG "-O1 -g")
    set(CMAKE_CXX_FLAGS_RELEASE "-Oz -DNDEBUG")
    set(CMAKE_C_FLAGS_RELEASE "-Oz -DNDEBUG")
    add_link_options(-Wl,--export-memory,--import-memory,--stack-first,-z,stack-size=1048576,--max-memory=4294967296)
endif()

include_directories(${CMAKE_CURRENT_SOURCE_DIR} ${MSGPACK_INCLUDE})

# I feel this should be limited to ecc, however it's currently used in headers that go across libraries,
# and there currently isn't an easy way to inherit the DDISABLE_SHENANIGANS parameter.
if(DISABLE_ASM)
    message("Using fallback non-assembly methods for field multiplications.")
    add_definitions(-DDISABLE_SHENANIGANS=1)
else()
    message(STATUS "Using optimized assembly for field arithmetic.")
endif()

add_subdirectory(barretenberg/bb)
add_subdirectory(barretenberg/commitment_schemes)
add_subdirectory(barretenberg/common)
add_subdirectory(barretenberg/crypto)
add_subdirectory(barretenberg/dsl)
add_subdirectory(barretenberg/ecc)
add_subdirectory(barretenberg/eccvm)
add_subdirectory(barretenberg/env)
add_subdirectory(barretenberg/examples)
add_subdirectory(barretenberg/flavor)
add_subdirectory(barretenberg/goblin)
add_subdirectory(barretenberg/grumpkin_srs_gen)
add_subdirectory(barretenberg/honk)
add_subdirectory(barretenberg/join_split_example)
add_subdirectory(barretenberg/numeric)
add_subdirectory(barretenberg/plonk)
add_subdirectory(barretenberg/polynomials)
add_subdirectory(barretenberg/proof_system)
add_subdirectory(barretenberg/protogalaxy)
add_subdirectory(barretenberg/relations)
add_subdirectory(barretenberg/serialize)
add_subdirectory(barretenberg/solidity_helpers)
add_subdirectory(barretenberg/srs)
add_subdirectory(barretenberg/stdlib)
add_subdirectory(barretenberg/sumcheck)
add_subdirectory(barretenberg/transcript)
add_subdirectory(barretenberg/translator_vm)
add_subdirectory(barretenberg/ultra_honk)
add_subdirectory(barretenberg/vm)
add_subdirectory(barretenberg/wasi)


if(SMT)
    include_directories(${CMAKE_CURRENT_SOURCE_DIR} $ENV{HOME}/cvc5/tmp-lib/include)
    add_subdirectory(barretenberg/smt_verification)
endif()

if(BENCHMARKS)
    add_subdirectory(barretenberg/benchmark)
endif()

include(GNUInstallDirs)

# For this library we include everything but the env and wasi modules, as it is the responsibility of the
# consumer of this library to define how and in what environment its artifact will run.
# libbarretenberg + libwasi = a wasi "reactor" that implements it's own env (e.g. logstr), e.g. barretenberg.wasm.
# libbarretenberg + env = a wasi "command" that expects a full wasi runtime (e.g. wasmtime), e.g. test binaries.
message(STATUS "Compiling all-in-one barretenberg archive")
add_library(
    barretenberg
    STATIC
    $<TARGET_OBJECTS:common_objects>
    $<TARGET_OBJECTS:crypto_aes128_objects>
    $<TARGET_OBJECTS:crypto_blake2s_objects>
    $<TARGET_OBJECTS:crypto_blake3s_objects>
    $<TARGET_OBJECTS:crypto_ecdsa_objects>
    $<TARGET_OBJECTS:crypto_keccak_objects>
    $<TARGET_OBJECTS:crypto_pedersen_commitment_objects>
    $<TARGET_OBJECTS:crypto_pedersen_hash_objects>
    $<TARGET_OBJECTS:crypto_schnorr_objects>
    $<TARGET_OBJECTS:crypto_sha256_objects>
    $<TARGET_OBJECTS:dsl_objects>
    $<TARGET_OBJECTS:ecc_objects>
    $<TARGET_OBJECTS:examples_objects>
    $<TARGET_OBJECTS:honk_objects>
    $<TARGET_OBJECTS:numeric_objects>
    $<TARGET_OBJECTS:plonk_objects>
    $<TARGET_OBJECTS:polynomials_objects>
    $<TARGET_OBJECTS:proof_system_objects>
    $<TARGET_OBJECTS:srs_objects>
    $<TARGET_OBJECTS:stdlib_aes128_objects>
    $<TARGET_OBJECTS:stdlib_blake2s_objects>
    $<TARGET_OBJECTS:stdlib_blake3s_objects>
    $<TARGET_OBJECTS:stdlib_keccak_objects>
    $<TARGET_OBJECTS:stdlib_merkle_tree_objects>
    $<TARGET_OBJECTS:stdlib_pedersen_commitment_objects>
    $<TARGET_OBJECTS:stdlib_pedersen_hash_objects>
    $<TARGET_OBJECTS:stdlib_primitives_objects>
    $<TARGET_OBJECTS:stdlib_schnorr_objects>
    $<TARGET_OBJECTS:stdlib_sha256_objects>
    $<TARGET_OBJECTS:transcript_objects>
)

if(WASM)
    # With binaryen installed, it seems its wasm backend optimizer gets invoked automatically.
    # Due to either a bug in the optimizer, or non-standards compliant c++ in crypto/aes, tests start failing with
    # -O3 level optimizations. We force down to -O2 for current workaround.
    # TODO: Time has passed, check if this is still needed.
    # UPDATE: Uninstall binaryen and any need downstream.
    set(CMAKE_CXX_FLAGS_RELEASE "-O2")

    # When building this wasm "executable", we include the wasi module but exclude the env module.
    # That's because we expect this wasm to be run as a wasi "reactor" and for the host environment
    # to implement the functions in env.
    add_executable(
        barretenberg.wasm
        $<TARGET_OBJECTS:common_objects>
        $<TARGET_OBJECTS:crypto_aes128_objects>
        $<TARGET_OBJECTS:crypto_blake2s_objects>
        $<TARGET_OBJECTS:crypto_blake3s_objects>
        $<TARGET_OBJECTS:crypto_ecdsa_objects>
        $<TARGET_OBJECTS:crypto_keccak_objects>
        $<TARGET_OBJECTS:crypto_pedersen_commitment_objects>
        $<TARGET_OBJECTS:crypto_pedersen_hash_objects>
        $<TARGET_OBJECTS:crypto_schnorr_objects>
        $<TARGET_OBJECTS:crypto_sha256_objects>
        $<TARGET_OBJECTS:dsl_objects>
        $<TARGET_OBJECTS:ecc_objects>
        $<TARGET_OBJECTS:examples_objects>
        $<TARGET_OBJECTS:honk_objects>
        $<TARGET_OBJECTS:numeric_objects>
        $<TARGET_OBJECTS:plonk_objects>
        $<TARGET_OBJECTS:polynomials_objects>
        $<TARGET_OBJECTS:proof_system_objects>
        $<TARGET_OBJECTS:srs_objects>
        $<TARGET_OBJECTS:stdlib_aes128_objects>
        $<TARGET_OBJECTS:stdlib_blake2s_objects>
        $<TARGET_OBJECTS:stdlib_blake3s_objects>
        $<TARGET_OBJECTS:stdlib_keccak_objects>
        $<TARGET_OBJECTS:stdlib_merkle_tree_objects>
        $<TARGET_OBJECTS:stdlib_pedersen_commitment_objects>
        $<TARGET_OBJECTS:stdlib_pedersen_hash_objects>
        $<TARGET_OBJECTS:stdlib_primitives_objects>
        $<TARGET_OBJECTS:stdlib_schnorr_objects>
        $<TARGET_OBJECTS:stdlib_sha256_objects>
        $<TARGET_OBJECTS:transcript_objects>
        $<TARGET_OBJECTS:wasi_objects>
    )

    add_executable(
        acvm_backend.wasm
        $<TARGET_OBJECTS:wasi_objects>
        $<TARGET_OBJECTS:env_objects>
        $<TARGET_OBJECTS:common_objects>
        $<TARGET_OBJECTS:numeric_objects>
        $<TARGET_OBJECTS:ecc_objects>
        $<TARGET_OBJECTS:crypto_blake2s_objects>
        $<TARGET_OBJECTS:crypto_keccak_objects>
        $<TARGET_OBJECTS:crypto_schnorr_objects>
        $<TARGET_OBJECTS:crypto_pedersen_hash_objects>
        $<TARGET_OBJECTS:crypto_pedersen_commitment_objects>
    )

    target_link_options(
        barretenberg.wasm
        PRIVATE
        -nostartfiles -Wl,--no-entry,--export-dynamic,--allow-undefined
    )

    target_link_options(
        acvm_backend.wasm
        PRIVATE
        -nostartfiles -Wl,--no-entry,--export-dynamic,--allow-undefined
    )
endif()
