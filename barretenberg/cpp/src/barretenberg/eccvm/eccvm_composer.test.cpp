#include <cstddef>
#include <cstdint>
#include <gtest/gtest.h>
#include <vector>

#include "barretenberg/eccvm/eccvm_circuit_builder.hpp"
#include "barretenberg/eccvm/eccvm_composer.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/plonk_honk_shared/library/grand_product_delta.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/relations/permutation_relation.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/sumcheck/sumcheck_round.hpp"

using namespace bb;

template <typename Flavor> class ECCVMComposerTests : public ::testing::Test {
  protected:
    // TODO(640): The Standard Honk on Grumpkin test suite fails unless the SRS is initialized for every test.
    void SetUp() override
    {
        if constexpr (std::is_same<Flavor, ECCVMFlavor>::value) {
            srs::init_grumpkin_crs_factory("../srs_db/grumpkin");
        } else {
            srs::init_crs_factory("../srs_db/ignition");
        }
    };
};

using FlavorTypes = ::testing::Types<ECCVMFlavor>;
TYPED_TEST_SUITE(ECCVMComposerTests, FlavorTypes);

namespace {
auto& engine = numeric::get_debug_randomness();
}
template <typename Flavor> ECCVMCircuitBuilder generate_circuit(numeric::RNG* engine = nullptr)
{
    std::shared_ptr<ECCOpQueue> op_queue = std::make_shared<ECCOpQueue>();
    using G1 = typename Flavor::CycleGroup;
    using Fr = typename G1::Fr;

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
    op_queue->eq();
    op_queue->add_accumulate(c);
    op_queue->mul_accumulate(a, x);
    op_queue->mul_accumulate(b, x);
    op_queue->eq();
    op_queue->mul_accumulate(a, x);
    op_queue->mul_accumulate(b, x);
    op_queue->mul_accumulate(c, x);
    ECCVMCircuitBuilder builder{ op_queue };
    return builder;
}

TYPED_TEST(ECCVMComposerTests, BaseCase)
{
    using Flavor = TypeParam;

    auto builder = generate_circuit<Flavor>(&engine);

    auto composer = ECCVMComposer_<Flavor>();
    auto prover = composer.create_prover(builder);

    auto proof = prover.construct_proof();
    auto verifier = composer.create_verifier(builder);
    bool verified = verifier.verify_proof(proof);

    ASSERT_TRUE(verified);
}

TYPED_TEST(ECCVMComposerTests, EqFails)
{
    using Flavor = TypeParam;
    using G1 = typename Flavor::CycleGroup;
    using ECCVMOperation = eccvm::VMOperation<G1>;
    auto builder = generate_circuit<Flavor>(&engine);
    // Tamper with the eq op such that the expected value is incorect
    builder.op_queue->raw_ops.emplace_back(ECCVMOperation{ .add = false,
                                                           .mul = false,
                                                           .eq = true,
                                                           .reset = true,
                                                           .base_point = G1::affine_one,
                                                           .z1 = 0,
                                                           .z2 = 0,
                                                           .mul_scalar_full = 0 });
    builder.op_queue->num_transcript_rows++;
    auto composer = ECCVMComposer_<Flavor>();
    auto prover = composer.create_prover(builder);

    auto proof = prover.construct_proof();
    auto verifier = composer.create_verifier(builder);
    bool verified = verifier.verify_proof(proof);
    ASSERT_FALSE(verified);
}
