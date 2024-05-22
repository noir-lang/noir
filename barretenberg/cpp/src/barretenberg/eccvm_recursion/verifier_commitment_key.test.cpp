
#include "barretenberg/eccvm_recursion/verifier_commitment_key.hpp"
#include <gtest/gtest.h>
namespace bb {
template <typename Builder> class RecursiveVeriferCommitmentKeyTest : public testing::Test {
  public:
    using native_VK = VerifierCommitmentKey<curve::Grumpkin>;
    using VK = VerifierCommitmentKey<stdlib::bn254<Builder>>;
    static void SetUpTestSuite()
    {
        srs::init_crs_factory("../srs_db/ignition");
        srs::init_grumpkin_crs_factory("../srs_db/grumpkin");
    }

    /**
     * @brief Instantiante a recursive verifier commitment key from a Grumpkin native key and check consistency.
     *
     */
    static void test_equality()
    {
        size_t num_points = 4096;
        Builder builder;
        auto native_vk = std::make_shared<native_VK>(num_points);
        auto recursive_vk = std::make_shared<VK>(&builder, num_points, native_vk);
        EXPECT_EQ(native_vk->get_first_g1(), recursive_vk->get_first_g1().get_value());
        auto* native_monomial_points = native_vk->get_monomial_points();
        auto recursive_monomial_points = recursive_vk->get_monomial_points();
        for (size_t i = 0; i < num_points; i++) {
            EXPECT_EQ(native_monomial_points[i], recursive_monomial_points[i].get_value());
        }
    }
};

using Builders = testing::Types<UltraCircuitBuilder, GoblinUltraCircuitBuilder>;

TYPED_TEST_SUITE(RecursiveVeriferCommitmentKeyTest, Builders);

TYPED_TEST(RecursiveVeriferCommitmentKeyTest, EqualityTest)
{
    TestFixture::test_equality();
};
} // namespace bb