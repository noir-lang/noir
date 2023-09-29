#include "../../primitives/circuit_builders/circuit_builders.hpp"
#include "./pedersen_refactor.hpp"
#include "barretenberg/crypto/pedersen_hash/pedersen_refactor.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include <gtest/gtest.h>

#define STDLIB_TYPE_ALIASES using Composer = TypeParam;

namespace stdlib_pedersen_tests {
using namespace barretenberg;
using namespace proof_system::plonk;

namespace {
auto& engine = numeric::random::get_debug_engine();
}

template <class Composer> class PedersenTest : public ::testing::Test {
  public:
    static void SetUpTestSuite(){

    };
};

using CircuitTypes = ::testing::Types<proof_system::StandardCircuitBuilder, proof_system::UltraCircuitBuilder>;
TYPED_TEST_SUITE(PedersenTest, CircuitTypes);

TYPED_TEST(PedersenTest, TestHash)
{
    STDLIB_TYPE_ALIASES;
    using field_ct = stdlib::field_t<Composer>;
    using witness_ct = stdlib::witness_t<Composer>;
    auto composer = Composer();

    const size_t num_inputs = 10;

    std::vector<field_ct> inputs;
    std::vector<fr> inputs_native;

    for (size_t i = 0; i < num_inputs; ++i) {
        const auto element = fr::random_element(&engine);
        inputs_native.emplace_back(element);
        inputs.emplace_back(field_ct(witness_ct(&composer, element)));
    }

    auto result = stdlib::pedersen_hash_refactor<Composer>::hash(inputs);
    auto expected = crypto::pedersen_hash_refactor<curve::Grumpkin>::hash(inputs_native);

    EXPECT_EQ(result.get_value(), expected);

    bool proof_result = composer.check_circuit();
    EXPECT_EQ(proof_result, true);
}
} // namespace stdlib_pedersen_tests