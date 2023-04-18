#include <gtest/gtest.h>
#include "barretenberg/plonk/composer/standard_composer.hpp"
#include "barretenberg/plonk/composer/turbo_composer.hpp"
#include "barretenberg/plonk/composer/ultra_composer.hpp"
#include "barretenberg/honk/composer/standard_honk_composer.hpp"
#include "barretenberg/stdlib/primitives/witness/witness.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "barretenberg/stdlib/primitives/group/group.hpp"
#include "barretenberg/numeric/random/engine.hpp"

#define STDLIB_TYPE_ALIASES                                                                                            \
    using Composer = TypeParam;                                                                                        \
    using witness_ct = stdlib::witness_t<Composer>;                                                                    \
    using field_ct = stdlib::field_t<Composer>;                                                                        \
    using group_ct = stdlib::group<Composer>;

namespace stdlib_group_tests {
using namespace barretenberg;
using namespace plonk;
namespace {
auto& engine = numeric::random::get_debug_engine();
}

template <class Composer> class GroupTest : public ::testing::Test {};

using ComposerTypes =
    ::testing::Types<honk::StandardHonkComposer, plonk::StandardComposer, plonk::TurboComposer, plonk::UltraComposer>;
TYPED_TEST_SUITE(GroupTest, ComposerTypes);

TYPED_TEST(GroupTest, TestFixedBaseScalarMul)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    auto scalar = uint256_t(123, 0, 0, 0);
    auto priv_key = grumpkin::fr(scalar);
    auto pub_key = crypto::generators::get_generator_data(crypto::generators::DEFAULT_GEN_1).generator * priv_key;

    auto priv_key_witness = field_ct(witness_ct(&composer, fr(scalar)));

    auto result = group_ct::template fixed_base_scalar_mul<128>(priv_key_witness, 0);

    EXPECT_EQ(result.x.get_value(), pub_key.x);
    EXPECT_EQ(result.y.get_value(), pub_key.y);

    auto native_result = crypto::generators::fixed_base_scalar_mul<128>(barretenberg::fr(scalar), 0);
    EXPECT_EQ(native_result.x, pub_key.x);
    EXPECT_EQ(native_result.y, pub_key.y);

    auto prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    plonk::proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TYPED_TEST(GroupTest, TestFixedBaseScalarMulZeroFails)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    auto scalar = uint256_t(0, 0, 0, 0);

    auto priv_key_witness = field_ct(witness_ct(&composer, fr(scalar)));
    group_ct::template fixed_base_scalar_mul<128>(priv_key_witness, 0);

    auto prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    plonk::proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, false);
    EXPECT_EQ(composer.err(), "input scalar to fixed_base_scalar_mul_internal cannot be 0");
}

TYPED_TEST(GroupTest, TestFixedBaseScalarMulWithTwoLimbs)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    const uint256_t scalar = engine.get_random_uint256();

    auto priv_key_low = (scalar.slice(0, 128));
    auto priv_key_high = (scalar.slice(128, 256));
    auto priv_key = grumpkin::fr(scalar);
    auto pub_key = grumpkin::g1::one * priv_key;
    pub_key = pub_key.normalize();
    auto priv_key_low_witness = field_ct(witness_ct(&composer, fr(priv_key_low)));
    auto priv_key_high_witness = field_ct(witness_ct(&composer, fr(priv_key_high)));

    auto result = group_ct::fixed_base_scalar_mul(priv_key_low_witness, priv_key_high_witness);

    EXPECT_EQ(result.x.get_value(), pub_key.x);
    EXPECT_EQ(result.y.get_value(), pub_key.y);

    auto prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    plonk::proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}
} // namespace stdlib_group_tests
