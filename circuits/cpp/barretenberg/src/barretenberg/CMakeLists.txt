# barretenburg
# copyright 2019 Spilsbury Holdings

include(FetchContent)

if(NOT WASM)
    FetchContent_Declare(
        leveldb
        GIT_REPOSITORY https://github.com/google/leveldb.git
        GIT_TAG 1.22
    )

    FetchContent_GetProperties(leveldb)
    if(NOT leveldb_POPULATED)
        FetchContent_Populate(leveldb)
        add_subdirectory(${leveldb_SOURCE_DIR} ${leveldb_BINARY_DIR} EXCLUDE_FROM_ALL)
    endif()

    set(
        CONDITIONAL_SOURCES
        waffle/stdlib/merkle_tree/leveldb_store.hpp
        waffle/stdlib/merkle_tree/leveldb_store.cpp
    )

    set(
        CONDITIONAL_LIBRARIES
        leveldb
    )
endif()

add_compile_options(-Werror -Wall -Wextra -Wconversion -Wsign-conversion)

if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wno-unguarded-availability-new -Wno-c99-extensions")
endif()

set(
    BARRETENBERG_SOURCES
    types.hpp
    uint256/uint512_impl.hpp
    uint256/uint512.hpp
    uint256/uint256_impl.hpp
    uint256/uint256.hpp
    fields/asm_macros.hpp
    fields/field_impl_generic.hpp
    fields/field_impl_x64.hpp
    fields/field_impl.hpp
    fields/field.hpp
    fields/field2_impl.hpp
    fields/field2.hpp
    fields/field6.hpp
    fields/field12.hpp
    groups/wnaf.hpp
    groups/affine_element_impl.hpp
    groups/affine_element.hpp
    groups/element_impl.hpp
    groups/element.hpp
    groups/group_impl_asm.tcc
    groups/group_impl_int128.tcc
    groups/group.hpp
    curves/bn254/fr.hpp
    curves/bn254/fq.hpp
    curves/bn254/fq2.hpp
    curves/bn254/fq6.hpp
    curves/bn254/fq12.hpp
    curves/bn254/g1.hpp
    curves/bn254/g2.hpp
    curves/bn254/scalar_multiplication/mmu.cpp
    curves/bn254/scalar_multiplication/mmu.hpp
    curves/bn254/scalar_multiplication/process_buckets.hpp
    curves/bn254/scalar_multiplication/scalar_multiplication_unsafe.cpp
    curves/bn254/scalar_multiplication/scalar_multiplication.cpp
    curves/bn254/scalar_multiplication/scalar_multiplication.hpp
    curves/bn254/pairing_impl.hpp
    curves/bn254/pairing.hpp
    curves/grumpkin/grumpkin.hpp
    misc_crypto/blake2s/blake2s.cpp
    misc_crypto/blake2s/blake2s.hpp
    misc_crypto/blake2s/blake2-impl.hpp
    misc_crypto/sha256/sha256.cpp
    misc_crypto/sha256/sha256.hpp
    misc_crypto/pedersen/pedersen.cpp
    misc_crypto/pedersen/pedersen.hpp
    misc_crypto/commitment/pedersen_note.cpp
    misc_crypto/commitment/pedersen_note.hpp
    misc_crypto/schnorr/c_bind.cpp
    misc_crypto/schnorr/schnorr.tcc
    misc_crypto/schnorr/schnorr.hpp
    misc_crypto/schnorr/c_bind.cpp
    io/io.hpp
    keccak/hash_types.h
    keccak/keccakf1600.c
    keccak/keccak.c
    keccak/keccak.h
    polynomials/evaluation_domain.cpp
    polynomials/evaluation_domain.hpp
    polynomials/polynomial_arithmetic.cpp
    polynomials/polynomial_arithmetic.hpp
    polynomials/polynomial.cpp
    polynomials/polynomial.hpp
    transcript/manifest.hpp
    transcript/transcript.cpp
    transcript/transcript.hpp
    waffle/waffle_types.hpp
    waffle/reference_string/reference_string.cpp
    waffle/reference_string/reference_string.hpp
    waffle/proof_system/proving_key/proving_key.cpp
    waffle/proof_system/proving_key/proving_key.hpp
    waffle/proof_system/verification_key/verification_key.cpp
    waffle/proof_system/verification_key/verification_key.hpp
    waffle/proof_system/linearizer.hpp
    waffle/proof_system/program_settings.hpp
    waffle/proof_system/permutation.hpp
    waffle/proof_system/preprocess.hpp
    waffle/proof_system/public_inputs/public_inputs.cpp
    waffle/proof_system/public_inputs/public_inputs.hpp
    waffle/proof_system/widgets/base_widget.hpp
    waffle/proof_system/widgets/arithmetic_widget.cpp
    waffle/proof_system/widgets/arithmetic_widget.hpp
    waffle/proof_system/widgets/bool_widget.cpp
    waffle/proof_system/widgets/bool_widget.hpp
    waffle/proof_system/widgets/mimc_widget.cpp
    waffle/proof_system/widgets/mimc_widget.hpp
    waffle/proof_system/widgets/sequential_widget.cpp
    waffle/proof_system/widgets/sequential_widget.hpp
    waffle/proof_system/widgets/turbo_arithmetic_widget.cpp
    waffle/proof_system/widgets/turbo_arithmetic_widget.hpp
    waffle/proof_system/widgets/turbo_fixed_base_widget.cpp
    waffle/proof_system/widgets/turbo_fixed_base_widget.hpp
    waffle/proof_system/widgets/turbo_range_widget.cpp
    waffle/proof_system/widgets/turbo_range_widget.hpp
    waffle/proof_system/widgets/turbo_logic_widget.cpp
    waffle/proof_system/widgets/turbo_logic_widget.hpp
    waffle/proof_system/prover/prover.cpp
    waffle/proof_system/prover/prover.hpp
    waffle/proof_system/verifier/verifier.cpp
    waffle/proof_system/verifier/verifier.hpp
    waffle/composer/composer_base.cpp
    waffle/composer/composer_base.hpp
    waffle/composer/standard_composer.cpp
    waffle/composer/standard_composer.hpp
    waffle/composer/mimc_composer.cpp
    waffle/composer/mimc_composer.hpp
    waffle/composer/turbo_composer.cpp
    waffle/composer/turbo_composer.hpp
    waffle/stdlib/common.hpp
    waffle/stdlib/int_utils.hpp
    waffle/stdlib/bool/bool.cpp
    waffle/stdlib/bool/bool.hpp
    waffle/stdlib/field/field.cpp
    waffle/stdlib/field/field.hpp
    waffle/stdlib/uint/arithmetic.cpp
    waffle/stdlib/uint/logic.cpp
    waffle/stdlib/uint/comparison.cpp
    waffle/stdlib/uint/uint.cpp
    waffle/stdlib/uint/uint.hpp
#    waffle/stdlib/uint/noir_uint.hpp
    waffle/stdlib/uint32/uint32.hpp
    waffle/stdlib/bitarray/bitarray.cpp
    waffle/stdlib/bitarray/bitarray.hpp
    waffle/stdlib/byte_array/byte_array.cpp
    waffle/stdlib/byte_array/byte_array.hpp
    waffle/stdlib/mimc.cpp
    waffle/stdlib/mimc.hpp
    waffle/stdlib/crypto/crypto.hpp
    waffle/stdlib/crypto/hash/blake2s.cpp
    waffle/stdlib/crypto/hash/blake2s.hpp
    waffle/stdlib/crypto/hash/sha256.cpp
    waffle/stdlib/crypto/hash/sha256.hpp
    waffle/stdlib/merkle_tree/hash.hpp
    waffle/stdlib/merkle_tree/merkle_tree.hpp
    waffle/stdlib/merkle_tree/memory_store.hpp
    waffle/stdlib/merkle_tree/memory_store.cpp
    waffle/stdlib/crypto/hash/pedersen.cpp
    waffle/stdlib/crypto/hash/pedersen.hpp
    waffle/stdlib/crypto/schnorr/schnorr.cpp
    waffle/stdlib/crypto/schnorr/schnorr.hpp
    waffle/stdlib/crypto/commitment/pedersen_note.cpp
    waffle/stdlib/crypto/commitment/pedersen_note.hpp
    ${CONDITIONAL_SOURCES}
)
    #noir/expression.cpp
    #noir/statement.cpp
    #noir/printer.cpp
    #noir/parse.cpp
    #noir/compiler/compiler.cpp
    #noir/compiler/function_statement_visitor.cpp
    #noir/compiler/expression_visitor.cpp
    #noir/compiler/function_call.cpp
    #noir/compiler/symbol_table.cpp

add_library(
    barretenberg STATIC
    ${BARRETENBERG_SOURCES}
)

target_precompile_headers(
    barretenberg
    PUBLIC
    "$<$<COMPILE_LANGUAGE:CXX>:<random$<ANGLE-R>>"
    $<$<COMPILE_LANGUAGE:CXX>:"${CMAKE_CURRENT_SOURCE_DIR}/assert.hpp">
    $<$<COMPILE_LANGUAGE:CXX>:"${CMAKE_CURRENT_SOURCE_DIR}/types.hpp">
    $<$<COMPILE_LANGUAGE:CXX>:"${CMAKE_CURRENT_SOURCE_DIR}/utils.hpp">
    $<$<COMPILE_LANGUAGE:CXX>:"${CMAKE_CURRENT_SOURCE_DIR}/uint256/uint256.hpp">
    $<$<COMPILE_LANGUAGE:CXX>:"${CMAKE_CURRENT_SOURCE_DIR}/uint256/uint256_impl.hpp">
    $<$<COMPILE_LANGUAGE:CXX>:"${CMAKE_CURRENT_SOURCE_DIR}/fields/asm_macros.hpp">
    $<$<COMPILE_LANGUAGE:CXX>:"${CMAKE_CURRENT_SOURCE_DIR}/fields/field.hpp">
    $<$<COMPILE_LANGUAGE:CXX>:"${CMAKE_CURRENT_SOURCE_DIR}/fields/field_impl_generic.hpp">
    $<$<COMPILE_LANGUAGE:CXX>:"${CMAKE_CURRENT_SOURCE_DIR}/fields/field_impl_x64.hpp">
)

target_include_directories(
    barretenberg
    SYSTEM PRIVATE
)
#         ${private_include_dir}/boost

set_target_properties(
    barretenberg
    PROPERTIES
    LINKER_LANGUAGE CXX
)

target_link_libraries(
    barretenberg
    PRIVATE
    ${CONDITIONAL_LIBRARIES}
)

if(WASM)
    add_executable(
        barretenberg.wasm
        ${BARRETENBERG_SOURCES}
    )

    target_include_directories(
        barretenberg.wasm
        SYSTEM PRIVATE
        ${private_include_dir}/boost
    )

    target_link_options(
        barretenberg.wasm
        PRIVATE
        -nostartfiles -Wl,--no-entry -Wl,--export-dynamic -Wl,--import-memory
    )
endif()

if(PROFILING)
    add_executable(
        barretenberg_profiling
        main.cpp
        ${BARRETENBERG_SOURCES}
    )

    target_link_libraries(
        barretenberg_profiling
        PRIVATE
        leveldb
    )

    set_target_properties(
        barretenberg_profiling
        PROPERTIES
        LINKER_LANGUAGE CXX
        RUNTIME_OUTPUT_DIRECTORY ../../test
    )
endif()

install(
    DIRECTORY
    ${include_dir}
    DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
    FILES_MATCHING
    PATTERN "*.hpp"
)
