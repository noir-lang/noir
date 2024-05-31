
#include "barretenberg/eccvm_recursion/verifier_commitment_key.hpp"
#include "barretenberg/stdlib/primitives/curves/grumpkin.hpp"
#include <gtest/gtest.h>
namespace bb {
template <typename Curve> class RecursiveVeriferCommitmentKeyTest : public testing::Test {
  public:
    using Builder = typename Curve::Builder;
    using NativeEmbeddedCurve = Builder::EmbeddedCurve;
    using native_VK = VerifierCommitmentKey<NativeEmbeddedCurve>;
    using VK = VerifierCommitmentKey<Curve>;
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

        // The recursive verifier commitment key only stores the SRS so we verify against the even indices of the native
        // key (the odd containt elements produced after applying the pippenger point table).
        for (size_t i = 0; i < num_points * 2; i += 2) {
            EXPECT_EQ(native_monomial_points[i], recursive_monomial_points[i >> 1].get_value());
        }
    }
};

using Curves = testing::Types<stdlib::grumpkin<UltraCircuitBuilder>, stdlib::grumpkin<MegaCircuitBuilder>>;

TYPED_TEST_SUITE(RecursiveVeriferCommitmentKeyTest, Curves);

TYPED_TEST(RecursiveVeriferCommitmentKeyTest, EqualityTest)
{
    TestFixture::test_equality();
};
} // namespace bb