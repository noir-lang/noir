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
using G1 = bb::g1;
using Fr = bb::fr;

class ECCVMComposerTests : public ::testing::Test {
  protected:
    // TODO(640): The Standard Honk on Grumpkin test suite fails unless the SRS is initialized for every test.
    void SetUp() override { srs::init_grumpkin_crs_factory("../srs_db/grumpkin"); };
};
namespace {
auto& engine = numeric::get_debug_randomness();
}
ECCVMCircuitBuilder generate_circuit(numeric::RNG* engine = nullptr)
{
    std::shared_ptr<ECCOpQueue> op_queue = std::make_shared<ECCOpQueue>();

    auto generators = G1::derive_generators("test generators", 3);

    typename G1::element a = generators[0];
    typename G1::element b = generators[1];
    typename G1::element c = generators[2];
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

TEST_F(ECCVMComposerTests, BaseCase)
{
    ECCVMCircuitBuilder builder = generate_circuit(&engine);
    ECCVMProver prover(builder);
    auto proof = prover.construct_proof();
    ECCVMVerifier verifier(prover.key);
    bool verified = verifier.verify_proof(proof);

    ASSERT_TRUE(verified);
}

TEST_F(ECCVMComposerTests, EqFails)
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
