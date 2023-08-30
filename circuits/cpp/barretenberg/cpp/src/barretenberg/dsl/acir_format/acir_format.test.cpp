#include <gtest/gtest.h>
#include <vector>

#include "acir_format.hpp"
#include "barretenberg/common/streams.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/serialize/test_helper.hpp"
#include "ecdsa_secp256k1.hpp"

namespace acir_format::tests {
TEST(acir_format, test_a_single_constraint_no_pub_inputs)
{

    poly_triple constraint{
        .a = 1,
        .b = 2,
        .c = 3,
        .q_m = 0,
        .q_l = 1,
        .q_r = 1,
        .q_o = -1,
        .q_c = 0,
    };

    acir_format constraint_system{
        .varnum = 4,
        .public_inputs = {},
        .logic_constraints = {},
        .range_constraints = {},
        .sha256_constraints = {},
        .schnorr_constraints = {},
        .ecdsa_k1_constraints = {},
        .ecdsa_r1_constraints = {},
        .blake2s_constraints = {},
        .keccak_constraints = {},
        .keccak_var_constraints = {},
        .pedersen_constraints = {},
        .hash_to_field_constraints = {},
        .fixed_base_scalar_mul_constraints = {},
        .recursion_constraints = {},
        .constraints = { constraint },
        .block_constraints = {},
    };

    auto builder = create_circuit_with_witness(constraint_system, { 0, 0, 1 });

    auto composer = Composer();
    auto prover = composer.create_ultra_with_keccak_prover(builder);
    auto proof = prover.construct_proof();

    auto verifier = composer.create_ultra_with_keccak_verifier(builder);

    EXPECT_EQ(verifier.verify_proof(proof), false);
}

TEST(acir_format, msgpack_logic_constraint)
{
    auto [actual, expected] = msgpack_roundtrip(LogicConstraint{});
    EXPECT_EQ(actual, expected);
}
TEST(acir_format, test_logic_gate_from_noir_circuit)
{
    /**
     * constraints produced by Noir program:
     * fn main(x : u32, y : pub u32) {
     * let z = x ^ y;
     *
     * constrain z != 10;
     * }
     **/
    RangeConstraint range_a{
        .witness = 1,
        .num_bits = 32,
    };
    RangeConstraint range_b{
        .witness = 2,
        .num_bits = 32,
    };

    LogicConstraint logic_constraint{
        .a = 1,
        .b = 2,
        .result = 3,
        .num_bits = 32,
        .is_xor_gate = 1,
    };
    poly_triple expr_a{
        .a = 3,
        .b = 4,
        .c = 0,
        .q_m = 0,
        .q_l = 1,
        .q_r = -1,
        .q_o = 0,
        .q_c = -10,
    };
    poly_triple expr_b{
        .a = 4,
        .b = 5,
        .c = 6,
        .q_m = 1,
        .q_l = 0,
        .q_r = 0,
        .q_o = -1,
        .q_c = 0,
    };
    poly_triple expr_c{
        .a = 4,
        .b = 6,
        .c = 4,
        .q_m = 1,
        .q_l = 0,
        .q_r = 0,
        .q_o = -1,
        .q_c = 0,

    };
    poly_triple expr_d{
        .a = 6,
        .b = 0,
        .c = 0,
        .q_m = 0,
        .q_l = -1,
        .q_r = 0,
        .q_o = 0,
        .q_c = 1,
    };
    // EXPR [ (1, _4, _5) (-1, _6) 0 ]
    // EXPR [ (1, _4, _6) (-1, _4) 0 ]
    // EXPR [ (-1, _6) 1 ]

    acir_format constraint_system{ .varnum = 7,
                                   .public_inputs = { 2 },
                                   .logic_constraints = { logic_constraint },
                                   .range_constraints = { range_a, range_b },
                                   .sha256_constraints = {},
                                   .schnorr_constraints = {},
                                   .ecdsa_k1_constraints = {},
                                   .ecdsa_r1_constraints = {},
                                   .blake2s_constraints = {},
                                   .keccak_constraints = {},
                                   .keccak_var_constraints = {},
                                   .pedersen_constraints = {},
                                   .hash_to_field_constraints = {},
                                   .fixed_base_scalar_mul_constraints = {},
                                   .recursion_constraints = {},
                                   .constraints = { expr_a, expr_b, expr_c, expr_d },
                                   .block_constraints = {} };

    uint256_t inverse_of_five = fr(5).invert();
    auto builder = create_circuit_with_witness(constraint_system,
                                               {
                                                   5,
                                                   10,
                                                   15,
                                                   5,
                                                   inverse_of_five,
                                                   1,
                                               });

    auto composer = Composer();
    auto prover = composer.create_ultra_with_keccak_prover(builder);
    auto proof = prover.construct_proof();

    auto verifier = composer.create_ultra_with_keccak_verifier(builder);

    EXPECT_EQ(verifier.verify_proof(proof), true);
}

TEST(acir_format, test_schnorr_verify_pass)
{
    std::vector<RangeConstraint> range_constraints;
    for (uint32_t i = 0; i < 10; i++) {
        range_constraints.push_back(RangeConstraint{
            .witness = i + 1,
            .num_bits = 15,
        });
    }

    std::vector<uint32_t> signature(64);
    for (uint32_t i = 0, value = 13; i < 64; i++, value++) {
        signature[i] = value;
        range_constraints.push_back(RangeConstraint{
            .witness = value,
            .num_bits = 15,
        });
    }

    SchnorrConstraint schnorr_constraint{
        .message = { 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 },
        .public_key_x = 11,
        .public_key_y = 12,
        .result = 77,
        .signature = signature,
    };
    acir_format constraint_system{ .varnum = 82,
                                   .public_inputs = {},
                                   .logic_constraints = {},
                                   .range_constraints = range_constraints,
                                   .sha256_constraints = {},
                                   .schnorr_constraints = { schnorr_constraint },
                                   .ecdsa_k1_constraints = {},
                                   .ecdsa_r1_constraints = {},
                                   .blake2s_constraints = {},
                                   .keccak_constraints = {},
                                   .keccak_var_constraints = {},
                                   .pedersen_constraints = {},
                                   .hash_to_field_constraints = {},
                                   .fixed_base_scalar_mul_constraints = {},
                                   .recursion_constraints = {},
                                   .constraints = { poly_triple{
                                       .a = schnorr_constraint.result,
                                       .b = schnorr_constraint.result,
                                       .c = schnorr_constraint.result,
                                       .q_m = 0,
                                       .q_l = 0,
                                       .q_r = 0,
                                       .q_o = 1,
                                       .q_c = fr::neg_one(),
                                   } },
                                   .block_constraints = {} };

    uint256_t pub_x = uint256_t("17cbd3ed3151ccfd170efe1d54280a6a4822640bf5c369908ad74ea21518a9c5");
    uint256_t pub_y = uint256_t("0e0456e3795c1a31f20035b741cd6158929eeccd320d299cfcac962865a6bc74");

    auto builder = create_circuit_with_witness(
        constraint_system,
        { 0,  1,   2,   3,   4,   5,   6,   7,   8,   9,   pub_x, pub_y, 5,   202, 31, 146, 81,  242, 246, 69,
          43, 107, 249, 153, 198, 44,  14,  111, 191, 121, 137,   166,   160, 103, 18, 181, 243, 233, 226, 95,
          67, 16,  37,  128, 85,  76,  19,  253, 30,  77,  192,   53,    138, 205, 69, 33,  236, 163, 83,  194,
          84, 137, 184, 221, 176, 121, 179, 27,  63,  70,  54,    16,    176, 250, 39, 239, 1,   0,   0,   0 });

    auto composer = Composer();
    auto prover = composer.create_ultra_with_keccak_prover(builder);
    auto proof = prover.construct_proof();

    auto verifier = composer.create_ultra_with_keccak_verifier(builder);

    EXPECT_EQ(verifier.verify_proof(proof), true);
}

TEST(acir_format, test_schnorr_verify_small_range)
{
    std::vector<RangeConstraint> range_constraints;
    for (uint32_t i = 0; i < 10; i++) {
        range_constraints.push_back(RangeConstraint{
            .witness = i + 1,
            .num_bits = 8,
        });
    }

    std::vector<uint32_t> signature(64);
    for (uint32_t i = 0, value = 13; i < 64; i++, value++) {
        signature[i] = value;
        range_constraints.push_back(RangeConstraint{
            .witness = value,
            .num_bits = 8,
        });
    }

    SchnorrConstraint schnorr_constraint{
        .message = { 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 },
        .public_key_x = 11,
        .public_key_y = 12,
        .result = 77,
        .signature = signature,
    };
    acir_format constraint_system{
        .varnum = 82,
        .public_inputs = {},
        .logic_constraints = {},
        .range_constraints = range_constraints,
        .sha256_constraints = {},
        .schnorr_constraints = { schnorr_constraint },
        .ecdsa_k1_constraints = {},
        .ecdsa_r1_constraints = {},
        .blake2s_constraints = {},
        .keccak_constraints = {},
        .keccak_var_constraints = {},
        .pedersen_constraints = {},
        .hash_to_field_constraints = {},
        .fixed_base_scalar_mul_constraints = {},
        .recursion_constraints = {},
        .constraints = { poly_triple{
            .a = schnorr_constraint.result,
            .b = schnorr_constraint.result,
            .c = schnorr_constraint.result,
            .q_m = 0,
            .q_l = 0,
            .q_r = 0,
            .q_o = 1,
            .q_c = fr::neg_one(),
        } },
        .block_constraints = {},
    };

    uint256_t pub_x = uint256_t("17cbd3ed3151ccfd170efe1d54280a6a4822640bf5c369908ad74ea21518a9c5");
    uint256_t pub_y = uint256_t("0e0456e3795c1a31f20035b741cd6158929eeccd320d299cfcac962865a6bc74");

    auto builder = create_circuit_with_witness(
        constraint_system,
        { 0,  1,   2,   3,   4,   5,   6,   7,   8,   9,   pub_x, pub_y, 5,   202, 31, 146, 81,  242, 246, 69,
          43, 107, 249, 153, 198, 44,  14,  111, 191, 121, 137,   166,   160, 103, 18, 181, 243, 233, 226, 95,
          67, 16,  37,  128, 85,  76,  19,  253, 30,  77,  192,   53,    138, 205, 69, 33,  236, 163, 83,  194,
          84, 137, 184, 221, 176, 121, 179, 27,  63,  70,  54,    16,    176, 250, 39, 239, 1,   0,   0,   0 });

    auto composer = Composer();
    auto prover = composer.create_ultra_with_keccak_prover(builder);
    auto proof = prover.construct_proof();
    auto verifier = composer.create_ultra_with_keccak_verifier(builder);
    EXPECT_EQ(verifier.verify_proof(proof), true);
}

TEST(acir_format, test_var_keccak)
{
    HashInput input1;
    input1.witness = 1;
    input1.num_bits = 8;
    HashInput input2;
    input2.witness = 2;
    input2.num_bits = 8;
    HashInput input3;
    input3.witness = 3;
    input3.num_bits = 8;
    KeccakVarConstraint keccak;
    keccak.inputs = { input1, input2, input3 };
    keccak.var_message_size = 4;
    keccak.result = { 5,  6,  7,  8,  9,  10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
                      21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36 };

    RangeConstraint range_a{
        .witness = 1,
        .num_bits = 8,
    };
    RangeConstraint range_b{
        .witness = 2,
        .num_bits = 8,
    };
    RangeConstraint range_c{
        .witness = 3,
        .num_bits = 8,
    };
    RangeConstraint range_d{
        .witness = 4,
        .num_bits = 8,
    };

    auto dummy = poly_triple{
        .a = 1,
        .b = 0,
        .c = 0,
        .q_m = 0,
        .q_l = 1,
        .q_r = 0,
        .q_o = 0,
        .q_c = fr::neg_one() * fr(4),
    };

    acir_format constraint_system{
        .varnum = 37,
        .public_inputs = {},
        .logic_constraints = {},
        .range_constraints = { range_a, range_b, range_c, range_d },
        .sha256_constraints = {},
        .schnorr_constraints = {},
        .ecdsa_k1_constraints = {},
        .ecdsa_r1_constraints = {},
        .blake2s_constraints = {},
        .keccak_constraints = {},
        .keccak_var_constraints = { keccak },
        .pedersen_constraints = {},
        .hash_to_field_constraints = {},
        .fixed_base_scalar_mul_constraints = {},
        .recursion_constraints = {},
        .constraints = { dummy },
        .block_constraints = {},
    };

    auto builder = create_circuit_with_witness(constraint_system, { 4, 2, 6, 2 });

    auto composer = Composer();
    auto prover = composer.create_ultra_with_keccak_prover(builder);
    auto proof = prover.construct_proof();
    auto verifier = composer.create_ultra_with_keccak_verifier(builder);
    EXPECT_EQ(verifier.verify_proof(proof), true);
}

} // namespace acir_format::tests
