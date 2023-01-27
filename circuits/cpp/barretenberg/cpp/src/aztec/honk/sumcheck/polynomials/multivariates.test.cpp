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

    std::array<FF, 3> f0 = { 0, 0, 1 };
    std::array<FF, 3> f1 = { 1, 1, 1 };
    std::array<FF, 3> f2 = { 3, 4, 1 };
    std::array<FF, 3> f3 = { -1, -1, 1 };

    auto full_polynomials = std::array<std::span<FF>, num_polys>({ f0, f1, f2, f3 });
    auto multivariates = Multivariates<FF, num_polys>(full_polynomials);

    ASSERT_TRUE(span_arrays_equal(full_polynomials, multivariates.full_polynomials));
}

/*
 * We represent a bivariate f0 as f0(X1, X2). The indexing starts from 1 to match with the round number in sumcheck.
 * The idea is variable X2 (lsb) will be folded at round 2 (the first sumcheck round),
 * then the variable X1 (msb) will be folded at round 1 (the last rond in this case). Pictorially we have,
 *          v10 ------ v11
 *           |          |
 *   X1(msb) |          |
 *           |  X2(lsb) |
 *          v00 ------ v01
 * f0(X1, X2) = v00 * (1-X1)(1-X2) + v01 * (1-X1) * X2 + v10 * X1)(1-X2) + v11 * X1 * X2.
 *
 * To effectively represent folding we write,
 * f0(X1, X2) = [v00 * (1-X2) + v01 * X2] * (1-X1) + [v10 * (1-X2) + v11 * X2] * X1.
 *
 * After folding at round 2 (round challenge u2), we have,
 * f0(X1,u2) = (v00 * (1-u2) + v01 * u2) * (1-X1) + (v11 * u2 + v10 * (1-u2)) * X1.
 *
 * After folding at round 1 (round challenge u1), we have,
 * f0(u1,u2) = (v00 * (1-u2) + v01 * u2) * (1-u1) + (v11 * u2 + v10 * (1-u2)) * u1.
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

/*
 * Similarly for a trivariate polynomial f0(X1, X2, X3), we have
 * f0(X1, X2, X3) = v000 * (1-X1) * (1-X2) * (1-X3) + v001 * (1-X1) * (1-X2) * X3 + v010 * (1-X1) * X2 * (1-X3) +
 * v011(1-X1)* X2 * X3 + v100 * X1 * (1-X2) * (1-X3) + v101 * X1 * (1-X2) * X3 + v110 * X1 * X2 * (1-X3) + v111 * X1 *
 * X2 * X3.
 * After the third round (round challenge u3), we have
 *  f0(X1, X2, u3) = [v000 * (1-u3) + v001 * u3] * (1-X1) * (1-X2) + [v010 * (1-u3) + v011 * u3] * (1-X1) * X2
 *                  + [v100 * (1-u3) + v101 * u3] * X1 * (1-X2) + [v110 * (1-u3) + v111 * u3] * X1 * X2.
 * After the second round (round challenge u2), we have
 * f0(X1, u2, u3) = [(v000 * (1-u3) + v001 * u3) * (1-u2) + (v010 * (1-u3) + v011 * u3) * u2] * (1-X1)
 *                  + [(v100 * (1-u3) + v101 * u3) * (1-u2) + (v110 * (1-u3) + v111 * u3) * u2] * X1.
 * After the first round (round challenge u1), we have
 * f0(u1, u2, u3) = [v000 * (1-u3) * (1-u2) + v001 * u3 * (1-u2) + v010 * (1-u3) * u2 + v011 * u3 * u2] * (1-u3)
 *                  + [v100 * (1-u3) * (1-u2) + v101 * u3 * (1-u2) + v110 * (1-u3) * u2 + v111 * u3 * u2] * u3.
 */
TYPED_TEST(MultivariatesTests, FoldThreeRoundsSpecial)
{
    MULTIVARIATES_TESTS_TYPE_ALIASES

    const size_t num_polys(1);
    const size_t multivariate_d(3);
    const size_t multivariate_n(1 << multivariate_d);

    FF v000 = 1;
    FF v001 = 2;
    FF v010 = 3;
    FF v011 = 4;
    FF v100 = 5;
    FF v101 = 6;
    FF v110 = 7;
    FF v111 = 8;

    std::array<FF, 8> f0 = { v000, v001, v010, v011, v100, v101, v110, v111 };

    auto full_polynomials = std::array<std::span<FF>, 1>({ f0 });
    auto multivariates = Multivariates<FF, num_polys>(full_polynomials);

    FF round_challenge_3 = 1;
    FF expected_q1 = v000 * (FF(1) - round_challenge_3) + v001 * round_challenge_3; // 2
    FF expected_q2 = v010 * (FF(1) - round_challenge_3) + v011 * round_challenge_3; // 4
    FF expected_q3 = v100 * (FF(1) - round_challenge_3) + v101 * round_challenge_3; // 6
    FF expected_q4 = v110 * (FF(1) - round_challenge_3) + v111 * round_challenge_3; // 8

    multivariates.fold(multivariates.full_polynomials, multivariate_n, round_challenge_3);

    EXPECT_EQ(multivariates.folded_polynomials[0][0], expected_q1);
    EXPECT_EQ(multivariates.folded_polynomials[0][1], expected_q2);
    EXPECT_EQ(multivariates.folded_polynomials[0][2], expected_q3);
    EXPECT_EQ(multivariates.folded_polynomials[0][3], expected_q4);

    FF round_challenge_2 = 2;
    FF expected_lo = expected_q1 * (FF(1) - round_challenge_2) + expected_q2 * round_challenge_2; // 6
    FF expected_hi = expected_q3 * (FF(1) - round_challenge_2) + expected_q4 * round_challenge_2; // 10

    multivariates.fold(multivariates.folded_polynomials, multivariate_n >> 1, round_challenge_2);
    EXPECT_EQ(multivariates.folded_polynomials[0][0], expected_lo);
    EXPECT_EQ(multivariates.folded_polynomials[0][1], expected_hi);

    FF round_challenge_1 = 3;
    FF expected_val = expected_lo * (FF(1) - round_challenge_1) + expected_hi * round_challenge_1; // 18
    multivariates.fold(multivariates.folded_polynomials, multivariate_n >> 2, round_challenge_1);
    EXPECT_EQ(multivariates.folded_polynomials[0][0], expected_val);
}

TYPED_TEST(MultivariatesTests, FoldThreeRoundsGeneric)
{
    MULTIVARIATES_TESTS_TYPE_ALIASES

    const size_t num_polys(1);
    const size_t multivariate_d(3);
    const size_t multivariate_n(1 << multivariate_d);

    FF v000 = FF::random_element();
    FF v001 = FF::random_element();
    FF v010 = FF::random_element();
    FF v011 = FF::random_element();
    FF v100 = FF::random_element();
    FF v101 = FF::random_element();
    FF v110 = FF::random_element();
    FF v111 = FF::random_element();

    std::array<FF, 8> f0 = { v000, v001, v010, v011, v100, v101, v110, v111 };

    auto full_polynomials = std::array<std::span<FF>, 1>({ f0 });
    auto multivariates = Multivariates<FF, num_polys>(full_polynomials);

    FF round_challenge_3 = FF::random_element();
    FF expected_q1 = v000 * (FF(1) - round_challenge_3) + v001 * round_challenge_3;
    FF expected_q2 = v010 * (FF(1) - round_challenge_3) + v011 * round_challenge_3;
    FF expected_q3 = v100 * (FF(1) - round_challenge_3) + v101 * round_challenge_3;
    FF expected_q4 = v110 * (FF(1) - round_challenge_3) + v111 * round_challenge_3;

    multivariates.fold(multivariates.full_polynomials, multivariate_n, round_challenge_3);

    EXPECT_EQ(multivariates.folded_polynomials[0][0], expected_q1);
    EXPECT_EQ(multivariates.folded_polynomials[0][1], expected_q2);
    EXPECT_EQ(multivariates.folded_polynomials[0][2], expected_q3);
    EXPECT_EQ(multivariates.folded_polynomials[0][3], expected_q4);

    FF round_challenge_2 = FF::random_element();
    FF expected_lo = expected_q1 * (FF(1) - round_challenge_2) + expected_q2 * round_challenge_2;
    FF expected_hi = expected_q3 * (FF(1) - round_challenge_2) + expected_q4 * round_challenge_2;

    multivariates.fold(multivariates.folded_polynomials, multivariate_n >> 1, round_challenge_2);
    EXPECT_EQ(multivariates.folded_polynomials[0][0], expected_lo);
    EXPECT_EQ(multivariates.folded_polynomials[0][1], expected_hi);

    FF round_challenge_1 = FF::random_element();
    FF expected_val = expected_lo * (FF(1) - round_challenge_1) + expected_hi * round_challenge_1;
    multivariates.fold(multivariates.folded_polynomials, multivariate_n >> 2, round_challenge_1);
    EXPECT_EQ(multivariates.folded_polynomials[0][0], expected_val);
}

TYPED_TEST(MultivariatesTests, FoldThreeRoundsGenericMultiplePolys)
{
    MULTIVARIATES_TESTS_TYPE_ALIASES
    const size_t num_polys(3);
    const size_t multivariate_d(3);
    const size_t multivariate_n(1 << multivariate_d);
    std::array<FF, 3> v000;
    std::array<FF, 3> v001;
    std::array<FF, 3> v010;
    std::array<FF, 3> v011;
    std::array<FF, 3> v100;
    std::array<FF, 3> v101;
    std::array<FF, 3> v110;
    std::array<FF, 3> v111;

    for (size_t i = 0; i < 3; i++) {
        v000[i] = FF::random_element();
        v001[i] = FF::random_element();
        v010[i] = FF::random_element();
        v011[i] = FF::random_element();
        v100[i] = FF::random_element();
        v101[i] = FF::random_element();
        v110[i] = FF::random_element();
        v111[i] = FF::random_element();
    }
    std::array<FF, 8> f0 = { v000[0], v001[0], v010[0], v011[0], v100[0], v101[0], v110[0], v111[0] };
    std::array<FF, 8> f1 = { v000[1], v001[1], v010[1], v011[1], v100[1], v101[1], v110[1], v111[1] };
    std::array<FF, 8> f2 = { v000[2], v001[2], v010[2], v011[2], v100[2], v101[2], v110[2], v111[2] };

    auto full_polynomials = std::array<std::span<FF>, 3>{ f0, f1, f2 };
    auto multivariates = Multivariates<FF, num_polys>(full_polynomials);

    std::array<FF, 3> expected_q1;
    std::array<FF, 3> expected_q2;
    std::array<FF, 3> expected_q3;
    std::array<FF, 3> expected_q4;
    FF round_challenge_3 = FF::random_element();
    for (size_t i = 0; i < 3; i++) {
        expected_q1[i] = v000[i] * (FF(1) - round_challenge_3) + v001[i] * round_challenge_3;
        expected_q2[i] = v010[i] * (FF(1) - round_challenge_3) + v011[i] * round_challenge_3;
        expected_q3[i] = v100[i] * (FF(1) - round_challenge_3) + v101[i] * round_challenge_3;
        expected_q4[i] = v110[i] * (FF(1) - round_challenge_3) + v111[i] * round_challenge_3;
    }

    multivariates.fold(multivariates.full_polynomials, multivariate_n, round_challenge_3);
    for (size_t i = 0; i < 3; i++) {
        EXPECT_EQ(multivariates.folded_polynomials[i][0], expected_q1[i]);
        EXPECT_EQ(multivariates.folded_polynomials[i][1], expected_q2[i]);
        EXPECT_EQ(multivariates.folded_polynomials[i][2], expected_q3[i]);
        EXPECT_EQ(multivariates.folded_polynomials[i][3], expected_q4[i]);
    }

    FF round_challenge_2 = FF::random_element();
    std::array<FF, 3> expected_lo;
    std::array<FF, 3> expected_hi;
    for (size_t i = 0; i < 3; i++) {
        expected_lo[i] = expected_q1[i] * (FF(1) - round_challenge_2) + expected_q2[i] * round_challenge_2;
        expected_hi[i] = expected_q3[i] * (FF(1) - round_challenge_2) + expected_q4[i] * round_challenge_2;
    }
    multivariates.fold(multivariates.folded_polynomials, multivariate_n >> 1, round_challenge_2);
    for (size_t i = 0; i < 3; i++) {
        EXPECT_EQ(multivariates.folded_polynomials[i][0], expected_lo[i]);
        EXPECT_EQ(multivariates.folded_polynomials[i][1], expected_hi[i]);
    }
    FF round_challenge_1 = FF::random_element();
    std::array<FF, 3> expected_val;
    for (size_t i = 0; i < 3; i++) {
        expected_val[i] = expected_lo[i] * (FF(1) - round_challenge_1) + expected_hi[i] * round_challenge_1;
    }
    multivariates.fold(multivariates.folded_polynomials, multivariate_n >> 2, round_challenge_1);
    for (size_t i = 0; i < 3; i++) {
        EXPECT_EQ(multivariates.folded_polynomials[i][0], expected_val[i]);
    }
}

} // namespace test_sumcheck_polynomials