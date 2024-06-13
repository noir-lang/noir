#include "acir_format.hpp"

acir_format::AcirFormatOriginalOpcodeIndices create_empty_original_opcode_indices()
{
    return acir_format::AcirFormatOriginalOpcodeIndices{
        .logic_constraints = {},
        .range_constraints = {},
        .aes128_constraints = {},
        .sha256_constraints = {},
        .sha256_compression = {},
        .schnorr_constraints = {},
        .ecdsa_k1_constraints = {},
        .ecdsa_r1_constraints = {},
        .blake2s_constraints = {},
        .blake3_constraints = {},
        .keccak_constraints = {},
        .keccak_permutations = {},
        .pedersen_constraints = {},
        .pedersen_hash_constraints = {},
        .poseidon2_constraints = {},
        .multi_scalar_mul_constraints = {},
        .ec_add_constraints = {},
        .recursion_constraints = {},
        .honk_recursion_constraints = {},
        .bigint_from_le_bytes_constraints = {},
        .bigint_to_le_bytes_constraints = {},
        .bigint_operations = {},
        .poly_triple_constraints = {},
        .quad_constraints = {},
        .block_constraints = {},
    };
}

void mock_opcode_indices(acir_format::AcirFormat& constraint_system)
{
    size_t current_opcode = 0;
    for (size_t i = 0; i < constraint_system.logic_constraints.size(); i++) {
        constraint_system.original_opcode_indices.logic_constraints.push_back(current_opcode++);
    }
    for (size_t i = 0; i < constraint_system.range_constraints.size(); i++) {
        constraint_system.original_opcode_indices.range_constraints.push_back(current_opcode++);
    }
    for (size_t i = 0; i < constraint_system.aes128_constraints.size(); i++) {
        constraint_system.original_opcode_indices.aes128_constraints.push_back(current_opcode++);
    }
    for (size_t i = 0; i < constraint_system.sha256_constraints.size(); i++) {
        constraint_system.original_opcode_indices.sha256_constraints.push_back(current_opcode++);
    }
    for (size_t i = 0; i < constraint_system.sha256_compression.size(); i++) {
        constraint_system.original_opcode_indices.sha256_compression.push_back(current_opcode++);
    }
    for (size_t i = 0; i < constraint_system.schnorr_constraints.size(); i++) {
        constraint_system.original_opcode_indices.schnorr_constraints.push_back(current_opcode++);
    }
    for (size_t i = 0; i < constraint_system.ecdsa_k1_constraints.size(); i++) {
        constraint_system.original_opcode_indices.ecdsa_k1_constraints.push_back(current_opcode++);
    }
    for (size_t i = 0; i < constraint_system.ecdsa_r1_constraints.size(); i++) {
        constraint_system.original_opcode_indices.ecdsa_r1_constraints.push_back(current_opcode++);
    }
    for (size_t i = 0; i < constraint_system.blake2s_constraints.size(); i++) {
        constraint_system.original_opcode_indices.blake2s_constraints.push_back(current_opcode++);
    }
    for (size_t i = 0; i < constraint_system.blake3_constraints.size(); i++) {
        constraint_system.original_opcode_indices.blake3_constraints.push_back(current_opcode++);
    }
    for (size_t i = 0; i < constraint_system.keccak_constraints.size(); i++) {
        constraint_system.original_opcode_indices.keccak_constraints.push_back(current_opcode++);
    }
    for (size_t i = 0; i < constraint_system.keccak_permutations.size(); i++) {
        constraint_system.original_opcode_indices.keccak_permutations.push_back(current_opcode++);
    }
    for (size_t i = 0; i < constraint_system.pedersen_constraints.size(); i++) {
        constraint_system.original_opcode_indices.pedersen_constraints.push_back(current_opcode++);
    }
    for (size_t i = 0; i < constraint_system.pedersen_hash_constraints.size(); i++) {
        constraint_system.original_opcode_indices.pedersen_hash_constraints.push_back(current_opcode++);
    }
    for (size_t i = 0; i < constraint_system.poseidon2_constraints.size(); i++) {
        constraint_system.original_opcode_indices.poseidon2_constraints.push_back(current_opcode++);
    }
    for (size_t i = 0; i < constraint_system.multi_scalar_mul_constraints.size(); i++) {
        constraint_system.original_opcode_indices.multi_scalar_mul_constraints.push_back(current_opcode++);
    }
    for (size_t i = 0; i < constraint_system.ec_add_constraints.size(); i++) {
        constraint_system.original_opcode_indices.ec_add_constraints.push_back(current_opcode++);
    }
    for (size_t i = 0; i < constraint_system.recursion_constraints.size(); i++) {
        constraint_system.original_opcode_indices.recursion_constraints.push_back(current_opcode++);
    }
    for (size_t i = 0; i < constraint_system.honk_recursion_constraints.size(); i++) {
        constraint_system.original_opcode_indices.honk_recursion_constraints.push_back(current_opcode++);
    }
    for (size_t i = 0; i < constraint_system.bigint_from_le_bytes_constraints.size(); i++) {
        constraint_system.original_opcode_indices.bigint_from_le_bytes_constraints.push_back(current_opcode++);
    }
    for (size_t i = 0; i < constraint_system.bigint_to_le_bytes_constraints.size(); i++) {
        constraint_system.original_opcode_indices.bigint_to_le_bytes_constraints.push_back(current_opcode++);
    }
    for (size_t i = 0; i < constraint_system.bigint_operations.size(); i++) {
        constraint_system.original_opcode_indices.bigint_operations.push_back(current_opcode++);
    }
    for (size_t i = 0; i < constraint_system.poly_triple_constraints.size(); i++) {
        constraint_system.original_opcode_indices.poly_triple_constraints.push_back(current_opcode++);
    }
    for (size_t i = 0; i < constraint_system.quad_constraints.size(); i++) {
        constraint_system.original_opcode_indices.quad_constraints.push_back(current_opcode++);
    }
    for (size_t i = 0; i < constraint_system.block_constraints.size(); i++) {
        std::vector<size_t> block_indices;
        for (size_t j = 0; j < constraint_system.block_constraints[i].trace.size(); j++) {
            block_indices.push_back(current_opcode++);
        }
        constraint_system.original_opcode_indices.block_constraints.push_back(block_indices);
    }

    constraint_system.num_acir_opcodes = static_cast<uint32_t>(current_opcode);
}