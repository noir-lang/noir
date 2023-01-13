#include "multivariates.hpp"
#include <ecc/curves/bn254/fr.hpp>

#include <gtest/gtest.h>
#include <numeric/random/engine.hpp>

using namespace honk::sumcheck;
namespace test_sumcheck_polynomials {

template <class FF> class MultivariatesTests : public testing::Test {
  public:
    // template <size_t num_polys, size_t multivariate_d>
    // using Multivariates = Multivariates<FF, num_polys, multivariate_d>;
    // TODO(Cody): reinstate this
    //     /*
    //         u2 = 1
    //         3 -------- 7         4 -------- 8       ~~>
    //         |          |         |          |       ~~>
    //         |    Y1    |         |    Y2    |       ~~>
    //         |          |         |          |       ~~>
    //         1 -------- 5         2 -------- 6       ~~> 3 -------- 7   4 -------- 8
    //         (3(1-X1)+7X1)  X2     (4(1-X1)+8X1)  X2      3(1-X1)+7X1    4(1-X1)+8X1
    //        +(1(1-X1)+5X1)(1-X2)  +(2(1-X1)+6X1)(1-X2)
    //      */
    //     static void test_fold_2()
    //     {
    //         const size_t num_polys(2);
    //         const size_t multivariate_d(2);
    //         const size_t multivariate_n(1 << multivariate_d);

    //         Edge Y11 = Edge({ 1, 3 });
    //         Edge Y12 = Edge({ 5, 7 });
    //         Edge Y21 = Edge({ 2, 4 });
    //         Edge Y22 = Edge({ 6, 8 });

    //         auto group_1 = EdgeGroup<num_polys>({ Y11, Y21 });
    //         auto group_2 = EdgeGroup<num_polys>({ Y12, Y22 });

    //         std::array<EdgeGroup<num_polys>, multivariate_d> groups{ group_1, group_2 };
    //         auto polys = Multivariates<num_polys, multivariate_d>(groups);

    //         FF u2 = 1;
    //         polys.fold(n, u2);

    //         EXPECT_EQ(polys.groups[0][0].at(0), 3);
    //         EXPECT_EQ(polys.groups[0][0].at(1), 7);

    //         EXPECT_EQ(polys.groups[0][1].at(0), 4);
    //         EXPECT_EQ(polys.groups[0][1].at(1), 8);
    //     }
};

using FieldTypes = testing::Types<barretenberg::fr>;
TYPED_TEST_SUITE(MultivariatesTests, FieldTypes);

#define MULTIVARIATES_TESTS_TYPE_ALIASES using FF = TypeParam;

TYPED_TEST(MultivariatesTests, Constructor)
{
    MULTIVARIATES_TESTS_TYPE_ALIASES

    const size_t num_polys(4);
    const size_t multivariate_d(2);
    // const size_t multivariate_n(1 << multivariate_d);

    std::array<FF, 3> f0 = { 0, 0, 1 };
    std::array<FF, 3> f1 = { 1, 1, 1 };
    std::array<FF, 3> f2 = { 3, 4, 1 };
    std::array<FF, 3> f3 = { -1, -1, 1 };

    auto full_polynomials = std::array<std::span<FF>, num_polys>({ f0, f1, f2, f3 });
    auto multivariates = Multivariates<FF, num_polys, multivariate_d>(full_polynomials);

    ASSERT_TRUE(span_arrays_equal(full_polynomials, multivariates.full_polynomials));
}

// IMPROVEMENT(Cody): rewrite or clarify this comment?
/*
               u2 = 1                  ~~>
           v01 ------ v11              ~~>
            |          |               ~~>
            |    Y     |               ~~>
            |          |               ~~>
           v00 ------ v10              ~~>     v00 * (1-u2) + v01 * u2 -------- (v11 * u2 + v10 * (1-u2))
    (v01 * (1-X1) + v11 * X1) *   X2   ~~>    (v00 * (1-u2) + v01 * u2) * (1-X1)
  + (v00 * (1-X1) + v10 * X1) * (1-X2) ~~>                                    + (v11 * u2 + v10 * (1-u2)) * X1
 */
TYPED_TEST(MultivariatesTests, FoldTwo)
{
    MULTIVARIATES_TESTS_TYPE_ALIASES

    const size_t num_polys(2);
    const size_t multivariate_d(1);
    const size_t multivariate_n(1 << multivariate_d);

    FF v00 = FF::random_element();
    FF v01 = FF::random_element();
    FF v10 = FF::random_element();
    FF v11 = FF::random_element();

    std::array<FF, 2> f0 = { v00, v10 };
    std::array<FF, 2> f1 = { v01, v11 };

    auto full_polynomials = std::array<std::span<FF>, 2>({ f0, f1 });
    auto multivariates = Multivariates<FF, num_polys, multivariate_d>(full_polynomials);

    FF round_challenge_2 = FF::random_element();
    FF expected_lo = v00 * (FF(1) - round_challenge_2) + v10 * round_challenge_2;
    FF expected_hi = v11 * round_challenge_2 + v01 * (FF(1) - round_challenge_2);

    multivariates.fold(multivariates.full_polynomials, multivariate_n, round_challenge_2);

    EXPECT_EQ(multivariates.folded_polynomials[0][0], expected_lo);
    EXPECT_EQ(multivariates.folded_polynomials[1][0], expected_hi);

    FF round_challenge_1 = FF::random_element();
    FF expected_val = expected_lo * (FF(1) - round_challenge_1) + expected_hi * round_challenge_1;

    multivariates.fold(multivariates.folded_polynomials, multivariate_n >> 1, round_challenge_1);
    EXPECT_EQ(multivariates.folded_polynomials[0][0], expected_val);
}
} // namespace test_sumcheck_polynomials