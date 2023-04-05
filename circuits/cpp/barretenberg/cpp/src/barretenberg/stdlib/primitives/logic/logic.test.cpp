#include "../bool/bool.hpp"
#include "logic.hpp"
#include "barretenberg/plonk/proof_system/constants.hpp"
#include <gtest/gtest.h>
#include "barretenberg/honk/composer/standard_honk_composer.hpp"
#include "barretenberg/plonk/composer/standard_composer.hpp"
#include "barretenberg/plonk/composer/ultra_composer.hpp"
#include "barretenberg/plonk/composer/turbo_composer.hpp"
#include "barretenberg/numeric/random/engine.hpp"

namespace test_stdlib_logic {

namespace {
auto& engine = numeric::random::get_debug_engine();
}

template <class T> void ignore_unused(T&) {} // use to ignore unused variables in lambdas

using namespace barretenberg;
using namespace proof_system::plonk;

template <typename Composer> class stdlib_logic : public testing::Test {
    typedef stdlib::bool_t<Composer> bool_ct;
    typedef stdlib::field_t<Composer> field_ct;
    typedef stdlib::witness_t<Composer> witness_ct;
    typedef stdlib::public_witness_t<Composer> public_witness_ct;

  public:
    /**
     * @brief Test logic
     */
    static void test_logic()
    {
        Composer composer;
        auto run_test = [&](size_t num_bits) {
            uint256_t mask = (uint256_t(1) << num_bits) - 1;

            uint256_t a = engine.get_random_uint256() & mask;
            uint256_t b = engine.get_random_uint256() & mask;

            uint256_t and_expected = a & b;
            uint256_t xor_expected = a ^ b;

            field_ct x = witness_ct(&composer, a);
            field_ct y = witness_ct(&composer, b);

            field_ct x_const(&composer, a);
            field_ct y_const(&composer, b);
            field_ct and_result = stdlib::logic<Composer>::create_logic_constraint(x, y, num_bits, false);
            field_ct xor_result = stdlib::logic<Composer>::create_logic_constraint(x, y, num_bits, true);

            field_ct and_result_left_constant =
                stdlib::logic<Composer>::create_logic_constraint(x_const, y, num_bits, false);
            field_ct xor_result_left_constant =
                stdlib::logic<Composer>::create_logic_constraint(x_const, y, num_bits, true);

            field_ct and_result_right_constant =
                stdlib::logic<Composer>::create_logic_constraint(x, y_const, num_bits, false);
            field_ct xor_result_right_constant =
                stdlib::logic<Composer>::create_logic_constraint(x, y_const, num_bits, true);

            field_ct and_result_both_constant =
                stdlib::logic<Composer>::create_logic_constraint(x_const, y_const, num_bits, false);
            field_ct xor_result_both_constant =
                stdlib::logic<Composer>::create_logic_constraint(x_const, y_const, num_bits, true);

            EXPECT_EQ(uint256_t(and_result.get_value()), and_expected);
            EXPECT_EQ(uint256_t(and_result_left_constant.get_value()), and_expected);
            EXPECT_EQ(uint256_t(and_result_right_constant.get_value()), and_expected);
            EXPECT_EQ(uint256_t(and_result_both_constant.get_value()), and_expected);

            EXPECT_EQ(uint256_t(xor_result.get_value()), xor_expected);
            EXPECT_EQ(uint256_t(xor_result_left_constant.get_value()), xor_expected);
            EXPECT_EQ(uint256_t(xor_result_right_constant.get_value()), xor_expected);
            EXPECT_EQ(uint256_t(xor_result_both_constant.get_value()), xor_expected);
        };

        for (size_t i = 8; i < 248; i += 8) {
            run_test(i);
        }
        auto prover = composer.create_prover();
        plonk::proof proof = prover.construct_proof();
        auto verifier = composer.create_verifier();
        bool result = verifier.verify_proof(proof);

        EXPECT_EQ(result, true);
    }
};

typedef testing::Types<plonk::UltraComposer, plonk::TurboComposer, plonk::StandardComposer, honk::StandardHonkComposer>
    ComposerTypes;

TYPED_TEST_SUITE(stdlib_logic, ComposerTypes);

TYPED_TEST(stdlib_logic, test_logic)
{
    TestFixture::test_logic();
}
} // namespace test_stdlib_logic