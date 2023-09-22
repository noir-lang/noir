#include <cstddef>
#include <cstdint>
#include <gtest/gtest.h>
#include <vector>

#include "barretenberg/honk/composer/eccvm_composer.hpp"
#include "barretenberg/honk/sumcheck/sumcheck_round.hpp"
#include "barretenberg/honk/utils/grand_product_delta.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/proof_system/circuit_builder/eccvm/eccvm_circuit_builder.hpp"
#include "barretenberg/proof_system/relations/permutation_relation.hpp"
#include "barretenberg/proof_system/relations/relation_parameters.hpp"

using namespace proof_system::honk;

namespace test_eccvm_composer {

template <typename Flavor> class ECCVMComposerTests : public ::testing::Test {
  protected:
    // TODO(640): The Standard Honk on Grumpkin test suite fails unless the SRS is initialised for every test.
    void SetUp() override
    {
        if constexpr (std::is_same<Flavor, flavor::ECCVMGrumpkin>::value) {
            barretenberg::srs::init_grumpkin_crs_factory("../srs_db/grumpkin");
        } else {
            barretenberg::srs::init_crs_factory("../srs_db/ignition");
        }
    };
};

using FlavorTypes = ::testing::Types<flavor::ECCVM, flavor::ECCVMGrumpkin>;
TYPED_TEST_SUITE(ECCVMComposerTests, FlavorTypes);

namespace {
auto& engine = numeric::random::get_debug_engine();
}
template <typename Flavor>
proof_system::ECCVMCircuitBuilder<Flavor> generate_trace(numeric::random::Engine* engine = nullptr)
{
    proof_system::ECCVMCircuitBuilder<Flavor> result;
    using G1 = typename Flavor::CycleGroup;
    using Fr = typename G1::Fr;

    auto generators = G1::template derive_generators<3>();

    typename G1::element a = generators[0];
    typename G1::element b = generators[1];
    typename G1::element c = generators[2];
    Fr x = Fr::random_element(engine);
    Fr y = Fr::random_element(engine);

    typename G1::element expected_1 = (a * x) + a + a + (b * y) + (b * x) + (b * x);
    typename G1::element expected_2 = (a * x) + c + (b * x);

    result.add_accumulate(a);
    result.mul_accumulate(a, x);
    result.mul_accumulate(b, x);
    result.mul_accumulate(b, y);
    result.add_accumulate(a);
    result.mul_accumulate(b, x);
    result.eq_and_reset(expected_1);
    result.add_accumulate(c);
    result.mul_accumulate(a, x);
    result.mul_accumulate(b, x);
    result.eq_and_reset(expected_2);
    result.mul_accumulate(a, x);
    result.mul_accumulate(b, x);
    result.mul_accumulate(c, x);

    return result;
}

TYPED_TEST(ECCVMComposerTests, BaseCase)
{
    using Flavor = TypeParam;

    auto circuit_constructor = generate_trace<Flavor>(&engine);

    auto composer = ECCVMComposer_<Flavor>();
    auto prover = composer.create_prover(circuit_constructor);

    auto proof = prover.construct_proof();
    auto verifier = composer.create_verifier(circuit_constructor);
    bool verified = verifier.verify_proof(proof);
    ASSERT_TRUE(verified);
}

TYPED_TEST(ECCVMComposerTests, EqFails)
{
    using Flavor = TypeParam;

    using G1 = typename Flavor::CycleGroup;
    auto circuit_constructor = generate_trace<Flavor>(&engine);
    // create an eq opcode that is not satisfied
    circuit_constructor.eq_and_reset(G1::affine_one);
    auto composer = ECCVMComposer_<Flavor>();
    auto prover = composer.create_prover(circuit_constructor);

    auto proof = prover.construct_proof();
    auto verifier = composer.create_verifier(circuit_constructor);
    bool verified = verifier.verify_proof(proof);
    ASSERT_FALSE(verified);
}
} // namespace test_eccvm_composer
