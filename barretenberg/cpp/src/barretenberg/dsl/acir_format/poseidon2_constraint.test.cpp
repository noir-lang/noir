#include "poseidon2_constraint.hpp"
#include "acir_format.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/plonk/proof_system/verification_key/verification_key.hpp"

#include <cstdint>
#include <gtest/gtest.h>
#include <vector>

namespace acir_format::tests {

class Poseidon2Tests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }
};
using fr = field<Bn254FrParams>;

/**
 * @brief Create a circuit testing the Poseidon2 permutation function
 *
 */
TEST_F(Poseidon2Tests, TestPoseidon2Permutation)
{
    Poseidon2Constraint
        poseidon2_constraint{
            .state = { 1, 2, 3, 4, },
            .result = { 5, 6, 7, 8, },
            .len = 4,
        };

    AcirFormat constraint_system{ .varnum = 9,
                                  .recursive = false,
                                  .public_inputs = {},
                                  .logic_constraints = {},
                                  .range_constraints = {},
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
                                  .poseidon2_constraints = { poseidon2_constraint },
                                  .fixed_base_scalar_mul_constraints = {},
                                  .ec_add_constraints = {},
                                  .recursion_constraints = {},
                                  .bigint_from_le_bytes_constraints = {},
                                  .bigint_to_le_bytes_constraints = {},
                                  .bigint_operations = {},
                                  .constraints = {},
                                  .block_constraints = {} };

    WitnessVector witness{
        1,
        0,
        1,
        2,
        3,
        bb::fr(std::string("0x01bd538c2ee014ed5141b29e9ae240bf8db3fe5b9a38629a9647cf8d76c01737")),
        bb::fr(std::string("0x239b62e7db98aa3a2a8f6a0d2fa1709e7a35959aa6c7034814d9daa90cbac662")),
        bb::fr(std::string("0x04cbb44c61d928ed06808456bf758cbf0c18d1e15a7b6dbc8245fa7515d5e3cb")),
        bb::fr(std::string("0x2e11c5cff2a22c64d01304b778d78f6998eff1ab73163a35603f54794c30847a")),
    };

    auto builder = create_circuit(constraint_system, /*size_hint=*/0, witness);

    auto composer = Composer();
    auto prover = composer.create_ultra_with_keccak_prover(builder);
    auto proof = prover.construct_proof();

    auto verifier = composer.create_ultra_with_keccak_verifier(builder);

    EXPECT_EQ(verifier.verify_proof(proof), true);
}

} // namespace acir_format::tests
