#include "acir_format.hpp"
#include "block_constraint.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/plonk/proof_system/verification_key/verification_key.hpp"

#include <gtest/gtest.h>
#include <vector>

size_t generate_block_constraint(acir_format::BlockConstraint& constraint, std::vector<fr>& witness_values)
{
    size_t witness_len = 1;
    witness_values.emplace_back(1);
    witness_len++;

    fr two = fr::one() + fr::one();
    poly_triple a0 = poly_triple{
        .a = 1,
        .b = 0,
        .c = 0,
        .q_m = 0,
        .q_l = two,
        .q_r = 0,
        .q_o = 0,
        .q_c = 0,
    };
    fr three = fr::one() + two;
    poly_triple a1 = poly_triple{
        .a = 0,
        .b = 0,
        .c = 0,
        .q_m = 0,
        .q_l = 0,
        .q_r = 0,
        .q_o = 0,
        .q_c = three,
    };
    poly_triple r1 = poly_triple{
        .a = 1,
        .b = 0,
        .c = 0,
        .q_m = 0,
        .q_l = fr::one(),
        .q_r = 0,
        .q_o = 0,
        .q_c = fr::neg_one(),
    };
    poly_triple r2 = poly_triple{
        .a = 1,
        .b = 0,
        .c = 0,
        .q_m = 0,
        .q_l = two,
        .q_r = 0,
        .q_o = 0,
        .q_c = fr::neg_one(),
    };
    poly_triple y = poly_triple{
        .a = 2,
        .b = 0,
        .c = 0,
        .q_m = 0,
        .q_l = fr::one(),
        .q_r = 0,
        .q_o = 0,
        .q_c = 0,
    };
    witness_values.emplace_back(2);
    witness_len++;
    poly_triple z = poly_triple{
        .a = 3,
        .b = 0,
        .c = 0,
        .q_m = 0,
        .q_l = fr::one(),
        .q_r = 0,
        .q_o = 0,
        .q_c = 0,
    };
    witness_values.emplace_back(3);
    witness_len++;
    acir_format::MemOp op1 = acir_format::MemOp{
        .access_type = 0,
        .index = r1,
        .value = y,
    };
    acir_format::MemOp op2 = acir_format::MemOp{
        .access_type = 0,
        .index = r2,
        .value = z,
    };
    constraint = acir_format::BlockConstraint{
        .init = { a0, a1 },
        .trace = { op1, op2 },
        .type = acir_format::BlockType::ROM,
    };

    return witness_len;
}

TEST(up_ram, TestBlockConstraint)
{
    acir_format::BlockConstraint block;
    std::vector<fr> witness_values;
    size_t num_variables = generate_block_constraint(block, witness_values);
    acir_format::acir_format constraint_system{
        .varnum = static_cast<uint32_t>(num_variables),
        .public_inputs = {},
        .fixed_base_scalar_mul_constraints = {},
        .logic_constraints = {},
        .range_constraints = {},
        .schnorr_constraints = {},
        .ecdsa_constraints = {},
        .sha256_constraints = {},
        .blake2s_constraints = {},
        .keccak_constraints = {},
        .keccak_var_constraints = {},
        .hash_to_field_constraints = {},
        .pedersen_constraints = {},
        .compute_merkle_root_constraints = {},
        .block_constraints = { block },
        .constraints = {},
    };

    auto composer = acir_format::create_circuit_with_witness(constraint_system, witness_values);

    auto prover = composer.create_prover();

    auto proof = prover.construct_proof();
    auto verifier = composer.create_verifier();
    EXPECT_EQ(verifier.verify_proof(proof), true);
}
