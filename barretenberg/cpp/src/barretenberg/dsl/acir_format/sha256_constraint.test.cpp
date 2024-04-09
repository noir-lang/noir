#include "sha256_constraint.hpp"
#include "acir_format.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/plonk/proof_system/verification_key/verification_key.hpp"

#include <gtest/gtest.h>
#include <vector>

namespace acir_format::tests {
using curve_ct = bb::stdlib::secp256k1<Builder>;

class Sha256Tests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }
};

TEST_F(Sha256Tests, TestSha256Compression)
{

    std::vector<Sha256Input> inputs;
    for (uint32_t i = 1; i < 17; ++i) {
        inputs.push_back({ .witness = i, .num_bits = 32 });
    }
    std::vector<Sha256Input> hash_values;
    for (uint32_t i = 17; i < 25; ++i) {
        hash_values.push_back({ .witness = i, .num_bits = 32 });
    }
    Sha256Compression sha256_compression{
        .inputs = inputs,
        .hash_values = hash_values,
        .result = { 25, 26, 27, 28, 29, 30, 31, 32 },
    };

    AcirFormat constraint_system{ .varnum = 34,
                                  .recursive = false,
                                  .public_inputs = {},
                                  .logic_constraints = {},
                                  .range_constraints = {},
                                  .sha256_constraints = {},
                                  .sha256_compression = { sha256_compression },
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
                                  .fixed_base_scalar_mul_constraints = {},
                                  .ec_add_constraints = {},
                                  .recursion_constraints = {},
                                  .bigint_from_le_bytes_constraints = {},
                                  .bigint_to_le_bytes_constraints = {},
                                  .bigint_operations = {},
                                  .constraints = {},
                                  .block_constraints = {} };

    WitnessVector witness{ 0,
                           0,
                           1,
                           2,
                           3,
                           4,
                           5,
                           6,
                           7,
                           8,
                           9,
                           10,
                           11,
                           12,
                           13,
                           14,
                           15,
                           0,
                           1,
                           2,
                           3,
                           4,
                           5,
                           6,
                           7,
                           static_cast<uint32_t>(3349900789),
                           1645852969,
                           static_cast<uint32_t>(3630270619),
                           1004429770,
                           739824817,
                           static_cast<uint32_t>(3544323979),
                           557795688,
                           static_cast<uint32_t>(3481642555) };

    auto builder = create_circuit(constraint_system, /*size_hint=*/0, witness);

    auto composer = Composer();
    auto prover = composer.create_ultra_with_keccak_prover(builder);
    auto proof = prover.construct_proof();

    auto verifier = composer.create_ultra_with_keccak_verifier(builder);

    EXPECT_EQ(verifier.verify_proof(proof), true);
}
} // namespace acir_format::tests
