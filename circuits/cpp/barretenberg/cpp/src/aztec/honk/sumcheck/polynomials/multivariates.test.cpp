#include "multivariates.hpp"
#include <ecc/curves/bn254/fr.hpp>

#include <gtest/gtest.h>
#include <numeric/random/engine.hpp>

#pragma GCC diagnostic ignored "-Wunused-variable"

using namespace honk::sumcheck;
namespace test_sumcheck_polynomials {

template <class FF> class MultivariatesTests : public testing::Test {};

using FieldTypes = testing::Types<barretenberg::fr>;
TYPED_TEST_SUITE(MultivariatesTests, FieldTypes);

#define MULTIVARIATES_TESTS_TYPE_ALIASES using FF = TypeParam;

TYPED_TEST(MultivariatesTests, Constructor)
{
    MULTIVARIATES_TESTS_TYPE_ALIASES

    const size_t num_polys(4);
    // const size_t multivariate_d(2);
    // const size_t multivariate_n(1 << multivariate_d);

    std::array<FF, 3> f0 = { 0, 0, 1 };
    std::array<FF, 3> f1 = { 1, 1, 1 };
    std::array<FF, 3> f2 = { 3, 4, 1 };
    std::array<FF, 3> f3 = { -1, -1, 1 };

    auto full_polynomials = std::array<std::span<FF>, num_polys>({ f0, f1, f2, f3 });
    auto multivariates = Multivariates<FF, num_polys>(full_polynomials);

    ASSERT_TRUE(span_arrays_equal(full_polynomials, multivariates.full_polynomials));
}

// IMPROVEMENT(Cody): rewrite or clarify this comment?
/*
           v01 ------ v11              ~~>
            |          |               ~~>
            |    Y     |               ~~>
            |          |               ~~>
           v00 ------ v10              ~~>     v00 * (1-u2) + v01 * u2 -------- (v11 * u2 + v10 * (1-u2))
    (v01 * (1-X1) + v11 * X1) *   X2   ~~>    (v00 * (1-u2) + v01 * u2) * (1-X1)
  + (v00 * (1-X1) + v10 * X1) * (1-X2) ~~>                                    + (v11 * u2 + v10 * (1-u2)) * X1
 */
TYPED_TEST(MultivariatesTests, FoldTwoRoundsSpecial)
{
    MULTIVARIATES_TESTS_TYPE_ALIASES

    // values here are chosen to check another test
    const size_t num_polys(1);
    const size_t multivariate_d(2);
    const size_t multivariate_n(1 << multivariate_d);

    FF v00 = 0;
    FF v01 = 1;
    FF v10 = 0;
    FF v11 = 0;

    std::array<FF, 4> f0 = { v00, v01, v10, v11 };

    auto full_polynomials = std::array<std::span<FF>, 1>({ f0 });
    auto multivariates = Multivariates<FF, num_polys>(full_polynomials);

    FF round_challenge_2 = { 0x6c7301b49d85a46c, 0x44311531e39c64f6, 0xb13d66d8d6c1a24c, 0x04410c360230a295 };
    round_challenge_2.self_to_montgomery_form();
    FF expected_lo = v00 * (FF(1) - round_challenge_2) + v01 * round_challenge_2;
    FF expected_hi = v11 * round_challenge_2 + v10 * (FF(1) - round_challenge_2);

    multivariates.fold(multivariates.full_polynomials, multivariate_n, round_challenge_2);

    EXPECT_EQ(multivariates.folded_polynomials[0][0], round_challenge_2);
    EXPECT_EQ(multivariates.folded_polynomials[0][1], FF(0));

    FF round_challenge_1 = 2;
    FF expected_val = expected_lo * (FF(1) - round_challenge_1) + expected_hi * round_challenge_1;

    multivariates.fold(multivariates.folded_polynomials, multivariate_n >> 1, round_challenge_1);
    EXPECT_EQ(multivariates.folded_polynomials[0][0], expected_val);
}

TYPED_TEST(MultivariatesTests, FoldTwoRoundsGeneric)
{
    MULTIVARIATES_TESTS_TYPE_ALIASES

    const size_t num_polys(1);
    const size_t multivariate_d(2);
    const size_t multivariate_n(1 << multivariate_d);

    FF v00 = FF::random_element();
    FF v01 = FF::random_element();
    FF v10 = FF::random_element();
    FF v11 = FF::random_element();

    std::array<FF, 4> f0 = { v00, v01, v10, v11 };

    auto full_polynomials = std::array<std::span<FF>, 1>({ f0 });
    auto multivariates = Multivariates<FF, num_polys>(full_polynomials);

    FF round_challenge_2 = FF::random_element();
    FF expected_lo = v00 * (FF(1) - round_challenge_2) + v01 * round_challenge_2;
    FF expected_hi = v11 * round_challenge_2 + v10 * (FF(1) - round_challenge_2);

    multivariates.fold(multivariates.full_polynomials, multivariate_n, round_challenge_2);

    EXPECT_EQ(multivariates.folded_polynomials[0][0], expected_lo);
    EXPECT_EQ(multivariates.folded_polynomials[0][1], expected_hi);

    FF round_challenge_1 = FF::random_element();
    FF expected_val = expected_lo * (FF(1) - round_challenge_1) + expected_hi * round_challenge_1;

    multivariates.fold(multivariates.folded_polynomials, multivariate_n >> 1, round_challenge_1);
    EXPECT_EQ(multivariates.folded_polynomials[0][0], expected_val);
}
} // namespace test_sumcheck_polynomials