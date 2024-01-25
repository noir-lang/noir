#include <gtest/gtest.h>
#include <vector>

#include "acir_format.hpp"
#include "barretenberg/common/streams.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/serialize/test_helper.hpp"
#include "ecdsa_secp256k1.hpp"

using namespace acir_format;

class AcirFormatTests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }
};
TEST_F(AcirFormatTests, TestASingleConstraintNoPubInputs)
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

    AcirFormat constraint_system{
        .varnum = 4,
        .public_inputs = {},
        .logic_constraints = {},
        .range_constraints = {},
        .sha256_constraints = {},
        .schnorr_constraints = {},
        .ecdsa_k1_constraints = {},
        .ecdsa_r1_constraints = {},
        .blake2s_constraints = {},
        .blake3_constraints = {},
        .keccak_constraints = {},
        .keccak_var_constraints = {},
        .keccak_permutations = {},
        .pedersen_constraints = {},
        .pedersen_hash_constraints = {},
        .fixed_base_scalar_mul_constraints = {},
        .ec_add_constraints = {},
        .recursion_constraints = {},
        .bigint_from_le_bytes_constraints = {},
        .bigint_operations = {},
        .constraints = { constraint },
        .block_constraints = {},
    };

    WitnessVector witness{ 0, 0, 1 };
    auto builder = create_circuit(constraint_system, /*size_hint*/ 0, witness);

    auto composer = Composer();
    auto prover = composer.create_ultra_with_keccak_prover(builder);
    auto proof = prover.construct_proof();

    auto verifier = composer.create_ultra_with_keccak_verifier(builder);

    EXPECT_EQ(verifier.verify_proof(proof), false);
}

TEST_F(AcirFormatTests, MsgpackLogicConstraint)
{
    auto [actual, expected] = msgpack_roundtrip(LogicConstraint{});
    EXPECT_EQ(actual, expected);
}
TEST_F(AcirFormatTests, TestLogicGateFromNoirCircuit)
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
        .witness = 0,
        .num_bits = 32,
    };
    RangeConstraint range_b{
        .witness = 1,
        .num_bits = 32,
    };

    LogicConstraint logic_constraint{
        .a = 0,
        .b = 1,
        .result = 2,
        .num_bits = 32,
        .is_xor_gate = 1,
    };
    poly_triple expr_a{
        .a = 2,
        .b = 3,
        .c = 0,
        .q_m = 0,
        .q_l = 1,
        .q_r = -1,
        .q_o = 0,
        .q_c = -10,
    };
    poly_triple expr_b{
        .a = 3,
        .b = 4,
        .c = 5,
        .q_m = 1,
        .q_l = 0,
        .q_r = 0,
        .q_o = -1,
        .q_c = 0,
    };
    poly_triple expr_c{
        .a = 3,
        .b = 5,
        .c = 3,
        .q_m = 1,
        .q_l = 0,
        .q_r = 0,
        .q_o = -1,
        .q_c = 0,

    };
    poly_triple expr_d{
        .a = 5,
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

    AcirFormat constraint_system{ .varnum = 6,
                                  .public_inputs = { 1 },
                                  .logic_constraints = { logic_constraint },
                                  .range_constraints = { range_a, range_b },
                                  .sha256_constraints = {},
                                  .schnorr_constraints = {},
                                  .ecdsa_k1_constraints = {},
                                  .ecdsa_r1_constraints = {},
                                  .blake2s_constraints = {},
                                  .blake3_constraints = {},
                                  .keccak_constraints = {},
                                  .keccak_var_constraints = {},
                                  .keccak_permutations = {},
                                  .pedersen_constraints = {},
                                  .pedersen_hash_constraints = {},
                                  .fixed_base_scalar_mul_constraints = {},
                                  .ec_add_constraints = {},
                                  .recursion_constraints = {},
                                  .bigint_from_le_bytes_constraints = {},
                                  .bigint_operations = {},
                                  .constraints = { expr_a, expr_b, expr_c, expr_d },
                                  .block_constraints = {} };

    uint256_t inverse_of_five = fr(5).invert();
    WitnessVector witness{
        5, 10, 15, 5, inverse_of_five, 1,
    };
    auto builder = create_circuit(constraint_system, /*size_hint*/ 0, witness);

    auto composer = Composer();
    auto prover = composer.create_ultra_with_keccak_prover(builder);
    auto proof = prover.construct_proof();

    auto verifier = composer.create_ultra_with_keccak_verifier(builder);

    EXPECT_EQ(verifier.verify_proof(proof), true);
}

TEST_F(AcirFormatTests, TestSchnorrVerifyPass)
{
    std::vector<RangeConstraint> range_constraints;
    for (uint32_t i = 0; i < 10; i++) {
        range_constraints.push_back(RangeConstraint{
            .witness = i,
            .num_bits = 15,
        });
    }

    std::vector<uint32_t> signature(64);
    for (uint32_t i = 0, value = 12; i < 64; i++, value++) {
        signature[i] = value;
        range_constraints.push_back(RangeConstraint{
            .witness = value,
            .num_bits = 15,
        });
    }

    SchnorrConstraint schnorr_constraint{
        .message = { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 },
        .public_key_x = 10,
        .public_key_y = 11,
        .result = 76,
        .signature = signature,
    };
    AcirFormat constraint_system{ .varnum = 81,
                                  .public_inputs = {},
                                  .logic_constraints = {},
                                  .range_constraints = range_constraints,
                                  .sha256_constraints = {},
                                  .schnorr_constraints = { schnorr_constraint },
                                  .ecdsa_k1_constraints = {},
                                  .ecdsa_r1_constraints = {},
                                  .blake2s_constraints = {},
                                  .blake3_constraints = {},
                                  .keccak_constraints = {},
                                  .keccak_var_constraints = {},
                                  .keccak_permutations = {},
                                  .pedersen_constraints = {},
                                  .pedersen_hash_constraints = {},
                                  .fixed_base_scalar_mul_constraints = {},
                                  .ec_add_constraints = {},
                                  .recursion_constraints = {},
                                  .bigint_from_le_bytes_constraints = {},
                                  .bigint_operations = {},
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

    std::string message_string = "tenletters";
    crypto::schnorr_key_pair<grumpkin::fr, grumpkin::g1> account;
    account.private_key = grumpkin::fr::random_element();
    account.public_key = grumpkin::g1::one * account.private_key;
    crypto::schnorr_signature signature_raw =
        crypto::schnorr_construct_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(message_string,
                                                                                                     account);
    uint256_t pub_x = account.public_key.x;
    uint256_t pub_y = account.public_key.y;
    WitnessVector witness{ 0,   1,   2,   3,   4,   5,   6,   7,   8,   9,   pub_x, pub_y, 5,   202, 31,  146,
                           81,  242, 246, 69,  43,  107, 249, 153, 198, 44,  14,    111,   191, 121, 137, 166,
                           160, 103, 18,  181, 243, 233, 226, 95,  67,  16,  37,    128,   85,  76,  19,  253,
                           30,  77,  192, 53,  138, 205, 69,  33,  236, 163, 83,    194,   84,  137, 184, 221,
                           176, 121, 179, 27,  63,  70,  54,  16,  176, 250, 39,    239,   1,   0,   0,   0 };
    for (size_t i = 0; i < 32; ++i) {
        witness[13 + i - 1] = signature_raw.s[i];
        witness[13 + 32 + i - 1] = signature_raw.e[i];
    }
    for (size_t i = 0; i < 10; ++i) {
        witness[i] = message_string[i];
    }

    auto builder = create_circuit(constraint_system, /*size_hint*/ 0, witness);

    auto composer = Composer();
    auto prover = composer.create_ultra_with_keccak_prover(builder);
    auto proof = prover.construct_proof();

    auto verifier = composer.create_ultra_with_keccak_verifier(builder);

    EXPECT_EQ(verifier.verify_proof(proof), true);
}

TEST_F(AcirFormatTests, TestSchnorrVerifySmallRange)
{
    std::vector<RangeConstraint> range_constraints;
    for (uint32_t i = 0; i < 10; i++) {
        range_constraints.push_back(RangeConstraint{
            .witness = i,
            .num_bits = 8,
        });
    }

    std::vector<uint32_t> signature(64);
    for (uint32_t i = 0, value = 12; i < 64; i++, value++) {
        signature[i] = value;
        range_constraints.push_back(RangeConstraint{
            .witness = value,
            .num_bits = 8,
        });
    }

    SchnorrConstraint schnorr_constraint{
        .message = { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 },
        .public_key_x = 10,
        .public_key_y = 11,
        .result = 76,
        .signature = signature,
    };
    AcirFormat constraint_system{
        .varnum = 81,
        .public_inputs = {},
        .logic_constraints = {},
        .range_constraints = range_constraints,
        .sha256_constraints = {},
        .schnorr_constraints = { schnorr_constraint },
        .ecdsa_k1_constraints = {},
        .ecdsa_r1_constraints = {},
        .blake2s_constraints = {},
        .blake3_constraints = {},
        .keccak_constraints = {},
        .keccak_var_constraints = {},
        .keccak_permutations = {},
        .pedersen_constraints = {},
        .pedersen_hash_constraints = {},
        .fixed_base_scalar_mul_constraints = {},
        .ec_add_constraints = {},
        .recursion_constraints = {},
        .bigint_from_le_bytes_constraints = {},
        .bigint_operations = {},
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

    std::string message_string = "tenletters";
    crypto::schnorr_key_pair<grumpkin::fr, grumpkin::g1> account;
    account.private_key = grumpkin::fr::random_element();
    account.public_key = grumpkin::g1::one * account.private_key;
    crypto::schnorr_signature signature_raw =
        crypto::schnorr_construct_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(message_string,
                                                                                                     account);
    uint256_t pub_x = account.public_key.x;
    uint256_t pub_y = account.public_key.y;
    WitnessVector witness{ 0,   1,   2,   3,   4,   5,   6,   7,   8,   9,   pub_x, pub_y, 5,   202, 31,  146,
                           81,  242, 246, 69,  43,  107, 249, 153, 198, 44,  14,    111,   191, 121, 137, 166,
                           160, 103, 18,  181, 243, 233, 226, 95,  67,  16,  37,    128,   85,  76,  19,  253,
                           30,  77,  192, 53,  138, 205, 69,  33,  236, 163, 83,    194,   84,  137, 184, 221,
                           176, 121, 179, 27,  63,  70,  54,  16,  176, 250, 39,    239,   1,   0,   0,   0 };
    for (size_t i = 0; i < 32; ++i) {
        witness[13 + i - 1] = signature_raw.s[i];
        witness[13 + 32 + i - 1] = signature_raw.e[i];
    }
    for (size_t i = 0; i < 10; ++i) {
        witness[i] = message_string[i];
    }

    // TODO: actually sign a schnorr signature!
    auto builder = create_circuit(constraint_system, /*size_hint*/ 0, witness);

    auto composer = Composer();
    auto prover = composer.create_ultra_with_keccak_prover(builder);
    auto proof = prover.construct_proof();
    auto verifier = composer.create_ultra_with_keccak_verifier(builder);
    EXPECT_EQ(verifier.verify_proof(proof), true);
}

TEST_F(AcirFormatTests, TestVarKeccak)
{
    HashInput input1;
    input1.witness = 0;
    input1.num_bits = 8;
    HashInput input2;
    input2.witness = 1;
    input2.num_bits = 8;
    HashInput input3;
    input3.witness = 2;
    input3.num_bits = 8;
    KeccakVarConstraint keccak;
    keccak.inputs = { input1, input2, input3 };
    keccak.var_message_size = 3;
    keccak.result = { 4,  5,  6,  7,  8,  9,  10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
                      20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35 };

    RangeConstraint range_a{
        .witness = 0,
        .num_bits = 8,
    };
    RangeConstraint range_b{
        .witness = 1,
        .num_bits = 8,
    };
    RangeConstraint range_c{
        .witness = 2,
        .num_bits = 8,
    };
    RangeConstraint range_d{
        .witness = 3,
        .num_bits = 8,
    };

    auto dummy = poly_triple{
        .a = 0,
        .b = 0,
        .c = 0,
        .q_m = 0,
        .q_l = 1,
        .q_r = 0,
        .q_o = 0,
        .q_c = fr::neg_one() * fr(4),
    };

    AcirFormat constraint_system{
        .varnum = 36,
        .public_inputs = {},
        .logic_constraints = {},
        .range_constraints = { range_a, range_b, range_c, range_d },
        .sha256_constraints = {},
        .schnorr_constraints = {},
        .ecdsa_k1_constraints = {},
        .ecdsa_r1_constraints = {},
        .blake2s_constraints = {},
        .blake3_constraints = {},
        .keccak_constraints = {},
        .keccak_var_constraints = { keccak },
        .keccak_permutations = {},
        .pedersen_constraints = {},
        .pedersen_hash_constraints = {},
        .fixed_base_scalar_mul_constraints = {},
        .ec_add_constraints = {},
        .recursion_constraints = {},
        .bigint_from_le_bytes_constraints = {},
        .bigint_operations = {},
        .constraints = { dummy },
        .block_constraints = {},
    };

    WitnessVector witness{ 4, 2, 6, 2 };
    auto builder = create_circuit(constraint_system, /*size_hint*/ 0, witness);

    auto composer = Composer();
    auto prover = composer.create_ultra_with_keccak_prover(builder);
    auto proof = prover.construct_proof();
    auto verifier = composer.create_ultra_with_keccak_verifier(builder);
    EXPECT_EQ(verifier.verify_proof(proof), true);
}

TEST_F(AcirFormatTests, TestKeccakPermutation)
{
    Keccakf1600
        keccak_permutation{
            .state = { 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25 },
            .result = { 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38,
                        39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50 },
        };

    AcirFormat constraint_system{ .varnum = 51,
                                  .public_inputs = {},
                                  .logic_constraints = {},
                                  .range_constraints = {},
                                  .sha256_constraints = {},
                                  .schnorr_constraints = {},
                                  .ecdsa_k1_constraints = {},
                                  .ecdsa_r1_constraints = {},
                                  .blake2s_constraints = {},
                                  .blake3_constraints = {},
                                  .keccak_constraints = {},
                                  .keccak_var_constraints = {},
                                  .keccak_permutations = { keccak_permutation },
                                  .pedersen_constraints = {},
                                  .pedersen_hash_constraints = {},
                                  .fixed_base_scalar_mul_constraints = {},
                                  .ec_add_constraints = {},
                                  .recursion_constraints = {},
                                  .bigint_from_le_bytes_constraints = {},
                                  .bigint_operations = {},
                                  .constraints = {},
                                  .block_constraints = {} };

    WitnessVector witness{ 1,  2,  3,  4,  5,  6,  7,  8,  9,  10, 11, 12, 13, 14, 15, 16, 17,
                           18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34,
                           35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50 };

    auto builder = create_circuit(constraint_system, /*size_hint=*/0, witness);

    auto composer = Composer();
    auto prover = composer.create_ultra_with_keccak_prover(builder);
    auto proof = prover.construct_proof();

    auto verifier = composer.create_ultra_with_keccak_verifier(builder);

    EXPECT_EQ(verifier.verify_proof(proof), true);
}
