#include <cstddef>
#include <cstdint>
#include <gtest/gtest.h>
#include <vector>

#include "barretenberg/eccvm/eccvm_circuit_builder.hpp"
#include "barretenberg/eccvm/eccvm_prover.hpp"
#include "barretenberg/eccvm/eccvm_verifier.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/plonk_honk_shared/library/grand_product_delta.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/relations/permutation_relation.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/sumcheck/sumcheck_round.hpp"

using namespace bb;

class ECCVMTests : public ::testing::Test {
  protected:
    void SetUp() override { srs::init_grumpkin_crs_factory("../srs_db/grumpkin"); };
};
namespace {
auto& engine = numeric::get_debug_randomness();
}

/**
 * @brief Adds operations in BN254 to the op_queue and then constructs and ECCVM circuit from the op_queue.
 *
 * @param engine
 * @return ECCVMCircuitBuilder
 */
ECCVMCircuitBuilder generate_circuit(numeric::RNG* engine = nullptr)
{
    using Curve = curve::BN254;
    using G1 = Curve::Element;
    using Fr = Curve::ScalarField;

    std::shared_ptr<ECCOpQueue> op_queue = std::make_shared<ECCOpQueue>();
    G1 a = G1::random_element(engine);
    G1 b = G1::random_element(engine);
    G1 c = G1::random_element(engine);
    Fr x = Fr::random_element(engine);
    Fr y = Fr::random_element(engine);

    op_queue->add_accumulate(a);
    op_queue->mul_accumulate(a, x);
    op_queue->mul_accumulate(b, x);
    op_queue->mul_accumulate(b, y);
    op_queue->add_accumulate(a);
    op_queue->mul_accumulate(b, x);
    op_queue->eq_and_reset();
    op_queue->add_accumulate(c);
    op_queue->mul_accumulate(a, x);
    op_queue->mul_accumulate(b, x);
    op_queue->eq_and_reset();
    op_queue->mul_accumulate(a, x);
    op_queue->mul_accumulate(b, x);
    op_queue->mul_accumulate(c, x);
    ECCVMCircuitBuilder builder{ op_queue };
    return builder;
}

TEST_F(ECCVMTests, BaseCase)
{
    ECCVMCircuitBuilder builder = generate_circuit(&engine);
    ECCVMProver prover(builder);
    auto proof = prover.construct_proof();
    ECCVMVerifier verifier(prover.key);
    bool verified = verifier.verify_proof(proof);

    ASSERT_TRUE(verified);
}

TEST_F(ECCVMTests, EqFails)
{
    auto builder = generate_circuit(&engine);
    // Tamper with the eq op such that the expected value is incorect
    builder.op_queue->add_erroneous_equality_op_for_testing();

    builder.op_queue->num_transcript_rows++;
    ECCVMProver prover(builder);

    auto proof = prover.construct_proof();
    ECCVMVerifier verifier(prover.key);
    bool verified = verifier.verify_proof(proof);
    ASSERT_FALSE(verified);
}
