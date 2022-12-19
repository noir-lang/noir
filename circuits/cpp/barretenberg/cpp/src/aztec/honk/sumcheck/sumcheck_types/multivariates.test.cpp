#include "./multivariates.hpp"
#include <common/mem.hpp>
#include <gtest/gtest.h>
#include <ecc/curves/bn254/fr.hpp>
#include <numeric/random/engine.hpp>

#pragma GCC diagnostic ignored "-Wunused-variable"
#pragma GCC diagnostic ignored "-Wunused-parameter"

using namespace honk::sumcheck;
namespace test_sumcheck_polynomials {

template <class Fr> class sumcheck_polynomials : public testing::Test {
    template <size_t num_polys, size_t multivariate_d>
    using Multivariates = Multivariates<Fr, num_polys, multivariate_d>;

  public:
    static void test_honk_polys_constructors()
    {
        const size_t num_polys(4);
        const size_t multivariate_d(2);
        // const size_t multivariate_n(1 << multivariate_d);

        Fr f0[3] = { 0, 0, 1 };
        Fr f1[3] = { 1, 1, 1 };
        Fr f2[3] = { 3, 4, 1 };
        Fr f3[3] = { -1, -1, 1 };

        auto full_polynomials = std::array<Fr*, num_polys>({ f0, f1, f2, f3 });
        auto multivariates = Multivariates<num_polys, multivariate_d>(full_polynomials);

        EXPECT_EQ(multivariates.full_polynomials, full_polynomials);
    }

    // TODO(cody): rewrite this comment
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
    static void test_fold_two()
    {
        const size_t num_polys(2);
        const size_t multivariate_d(1);
        const size_t multivariate_n(1 << multivariate_d);

        Fr v00 = Fr::random_element();
        Fr v01 = Fr::random_element();
        Fr v10 = Fr::random_element();
        Fr v11 = Fr::random_element();

        Fr f0[2] = { v00, v10 };
        Fr f1[2] = { v01, v11 };

        auto full_polynomials = std::array<Fr*, 2>({ f0, f1 });
        auto multivariates = Multivariates<num_polys, multivariate_d>(full_polynomials);

        Fr u2 = Fr::random_element();
        Fr expected_lo = v00 * (Fr(1) - u2) + v10 * u2;
        Fr expected_hi = v11 * u2 + v01 * (Fr(1) - u2);

        multivariates.fold_first_round(2, u2);

        EXPECT_EQ(multivariates.folded_polynomials[0][0], expected_lo);
        EXPECT_EQ(multivariates.folded_polynomials[1][0], expected_hi);

        Fr u1 = Fr::random_element();
        Fr expected_val = expected_lo * (Fr(1) - u1) + expected_hi * u1;

        multivariates.fold(multivariate_n >> 1, u1);
        // Seems the edge case is handled correctly?
        EXPECT_EQ(multivariates.folded_polynomials[0][0], expected_val);
    }

    // TODO(cody): reinstate this
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

    //         Fr u2 = 1;
    //         polys.fold(n, u2);

    //         EXPECT_EQ(polys.groups[0][0].at(0), 3);
    //         EXPECT_EQ(polys.groups[0][0].at(1), 7);

    //         EXPECT_EQ(polys.groups[0][1].at(0), 4);
    //         EXPECT_EQ(polys.groups[0][1].at(1), 8);
    //     }
};

typedef testing::Types<barretenberg::fr> FieldTypes;
TYPED_TEST_SUITE(sumcheck_polynomials, FieldTypes);

TYPED_TEST(sumcheck_polynomials, honk_polys_constructor)
{
    TestFixture::test_honk_polys_constructors();
}
TYPED_TEST(sumcheck_polynomials, fold_2)
{
    TestFixture::test_fold_two();
}
// TYPED_TEST(sumcheck_polynomials, fold_2)
// {
//     TestFixture::test_fold_2();
// }
} // namespace test_sumcheck_polynomials