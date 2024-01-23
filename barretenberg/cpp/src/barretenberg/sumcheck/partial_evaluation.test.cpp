#include "barretenberg/flavor/ultra.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"

#include <gtest/gtest.h>

using namespace bb::honk::sumcheck;

template <typename Flavor> class PartialEvaluationTests : public testing::Test {};

using Flavors = testing::Types<bb::honk::flavor::Ultra>;

TYPED_TEST_SUITE(PartialEvaluationTests, Flavors);

/*
 * We represent a bivariate f0 as f0(X0, X1). The indexing starts from 0 to match with the round number in sumcheck.
 * The idea is variable X0 (lsb) will be folded at round 2 (the first sumcheck round),
 * then the variable X1 (msb) will be folded at round 1 (the last rond in this case). Pictorially we have,
 *          v10 ------ v11
 *           |          |
 *   X0(lsb) |          |
 *           |  X1(msb) |
 *          v00 ------ v01
 * f0(X0, X1) = v00 * (1-X0) * (1-X1)
 *            + v10 *   X0   * (1-X1)
 *            + v01 * (1-X0) *   X1
 *            + v11 *   X0   *   X1.
 *
 * To effectively represent folding we write,
 * f0(X0, X1) = [v00 * (1-X0) + v10 * X0] * (1-X1)
 *            + [v01 * (1-X0) + v11 * X0] *   X1.
 *
 * After folding at round 0 (round challenge u0), we have,
 * f0(u0,X1) = (v00 * (1-u0) + v10 * u0) * (1-X1)
 *           + (v01 * (1-u0) + v11 * u0) *   X1.
 *
 * After folding at round 1 (round challenge u1), we have,
 * f0(u0,u1) = (v00 * (1-u0) + v10 * u0) * (1-u1)
 *           + (v01 * (1-u0) + v11 * u0) *   u1.
 */
TYPED_TEST(PartialEvaluationTests, TwoRoundsSpecial)
{
    using Flavor = TypeParam;
    using FF = typename Flavor::FF;
    using Transcript = typename Flavor::Transcript;

    // values here are chosen to check another test
    const size_t multivariate_d(2);
    const size_t multivariate_n(1 << multivariate_d);

    FF v00 = 0;
    FF v10 = 1;
    FF v01 = 0;
    FF v11 = 0;

    std::array<FF, 4> f0 = { v00, v10, v01, v11 };

    auto full_polynomials = std::array<std::span<FF>, 1>({ f0 });
    auto transcript = Transcript::prover_init_empty();
    auto sumcheck = SumcheckProver<Flavor>(multivariate_n, transcript);

    FF round_challenge_0 = { 0x6c7301b49d85a46c, 0x44311531e39c64f6, 0xb13d66d8d6c1a24c, 0x04410c360230a295 };
    round_challenge_0.self_to_montgomery_form();
    FF expected_lo = v00 * (FF(1) - round_challenge_0) + v10 * round_challenge_0;
    FF expected_hi = v01 * (FF(1) - round_challenge_0) + v11 * round_challenge_0;

    sumcheck.partially_evaluate(full_polynomials, multivariate_n, round_challenge_0);

    auto& first_polynomial = sumcheck.partially_evaluated_polynomials.get_all()[0];
    EXPECT_EQ(first_polynomial[0], round_challenge_0);
    EXPECT_EQ(first_polynomial[1], FF(0));

    FF round_challenge_1 = 2;
    FF expected_val = expected_lo * (FF(1) - round_challenge_1) + expected_hi * round_challenge_1;

    sumcheck.partially_evaluate(sumcheck.partially_evaluated_polynomials, multivariate_n >> 1, round_challenge_1);
    EXPECT_EQ(first_polynomial[0], expected_val);
}

TYPED_TEST(PartialEvaluationTests, TwoRoundsGeneric)
{
    using Flavor = TypeParam;
    using FF = typename Flavor::FF;
    using Transcript = typename Flavor::Transcript;

    const size_t multivariate_d(2);
    const size_t multivariate_n(1 << multivariate_d);

    FF v00 = FF::random_element();
    FF v10 = FF::random_element();
    FF v01 = FF::random_element();
    FF v11 = FF::random_element();

    std::array<FF, 4> f0 = { v00, v10, v01, v11 };

    auto full_polynomials = std::array<std::span<FF>, 1>({ f0 });
    auto transcript = Transcript::prover_init_empty();
    auto sumcheck = SumcheckProver<Flavor>(multivariate_n, transcript);

    FF round_challenge_0 = FF::random_element();
    FF expected_lo = v00 * (FF(1) - round_challenge_0) + v10 * round_challenge_0;
    FF expected_hi = v01 * (FF(1) - round_challenge_0) + v11 * round_challenge_0;

    sumcheck.partially_evaluate(full_polynomials, multivariate_n, round_challenge_0);
    auto& first_polynomial = sumcheck.partially_evaluated_polynomials.get_all()[0];

    EXPECT_EQ(first_polynomial[0], expected_lo);
    EXPECT_EQ(first_polynomial[1], expected_hi);

    FF round_challenge_1 = FF::random_element();
    FF expected_val = expected_lo * (FF(1) - round_challenge_1) + expected_hi * round_challenge_1;
    sumcheck.partially_evaluate(sumcheck.partially_evaluated_polynomials, multivariate_n >> 1, round_challenge_1);
    EXPECT_EQ(first_polynomial[0], expected_val);
}

/*
 * Similarly for a trivariate polynomial f0(X0, X1, X2), we have
 * f0(X0, X1, X2) = v000 * (1-X0) * (1-X1) * (1-X2)
 *                + v100 *   X0   * (1-X1) * (1-X2)
 *                + v010 * (1-X0) *   X1   * (1-X2)
 *                + v110 *   X0   *   X1   * (1-X2)
 *                + v001 * (1-X0) * (1-X1) *   X2
 *                + v101 *   X0   * (1-X1) *   X2
 *                + v011 * (1-X0) *   X1   *   X2
 *                + v111 *   X0   *   X1   *   X2.
 * After round 0 (round challenge u0), we have
 *  f0(u0, X1, X2) = [v000 * (1-u0) + v100 * u0] * (1-X1) * (1-X2)
 *                 + [v010 * (1-u0) + v110 * u0] *   X1   * (1-X2)
 *                 + [v001 * (1-u0) + v101 * u0] * (1-X1) *   X2
 *                 + [v011 * (1-u0) + v111 * u0] *   X1   *   X2.
 * After round 1 (round challenge u1), we have
 * f0(u0, u1, X2) = [(v000 * (1-u0) + v100 * u0) * (1-u1) + (v010 * (1-u0) + v110 * u0) * u1] * (1-X2)
 *                + [(v001 * (1-u0) + v101 * u0) * (1-u1) + (v011 * (1-u0) + v111 * u0) * u1] *   X2.
 * After round 2 (round challenge u2), we have
 * f0(u0, u1, u2) = [(v000 * (1-u0) + v100 * u0) * (1-u1) + (v010 * (1-u0) + v110 * u0) * u1] * (1-u2)
 *                + [(v001 * (1-u0) + v101 * u0) * (1-u1) + (v011 * (1-u0) + v111 * u0) * u1] *   u2.
 */
TYPED_TEST(PartialEvaluationTests, ThreeRoundsSpecial)
{
    using Flavor = TypeParam;
    using FF = typename Flavor::FF;
    using Transcript = typename Flavor::Transcript;

    const size_t multivariate_d(3);
    const size_t multivariate_n(1 << multivariate_d);

    FF v000 = 1;
    FF v100 = 2;
    FF v010 = 3;
    FF v110 = 4;
    FF v001 = 5;
    FF v101 = 6;
    FF v011 = 7;
    FF v111 = 8;

    std::array<FF, 8> f0 = { v000, v100, v010, v110, v001, v101, v011, v111 };

    auto full_polynomials = std::array<std::span<FF>, 1>({ f0 });
    auto transcript = Transcript::prover_init_empty();
    auto sumcheck = SumcheckProver<Flavor>(multivariate_n, transcript);

    FF round_challenge_0 = 1;
    FF expected_q1 = v000 * (FF(1) - round_challenge_0) + v100 * round_challenge_0; // 2
    FF expected_q2 = v010 * (FF(1) - round_challenge_0) + v110 * round_challenge_0; // 4
    FF expected_q3 = v001 * (FF(1) - round_challenge_0) + v101 * round_challenge_0; // 6
    FF expected_q4 = v011 * (FF(1) - round_challenge_0) + v111 * round_challenge_0; // 8

    sumcheck.partially_evaluate(full_polynomials, multivariate_n, round_challenge_0);

    auto& first_polynomial = sumcheck.partially_evaluated_polynomials.get_all()[0];
    EXPECT_EQ(first_polynomial[0], expected_q1);
    EXPECT_EQ(first_polynomial[1], expected_q2);
    EXPECT_EQ(first_polynomial[2], expected_q3);
    EXPECT_EQ(first_polynomial[3], expected_q4);

    FF round_challenge_1 = 2;
    FF expected_lo = expected_q1 * (FF(1) - round_challenge_1) + expected_q2 * round_challenge_1; // 6
    FF expected_hi = expected_q3 * (FF(1) - round_challenge_1) + expected_q4 * round_challenge_1; // 10

    sumcheck.partially_evaluate(sumcheck.partially_evaluated_polynomials, multivariate_n >> 1, round_challenge_1);
    EXPECT_EQ(first_polynomial[0], expected_lo);
    EXPECT_EQ(first_polynomial[1], expected_hi);

    FF round_challenge_2 = 3;
    FF expected_val = expected_lo * (FF(1) - round_challenge_2) + expected_hi * round_challenge_2; // 18
    sumcheck.partially_evaluate(sumcheck.partially_evaluated_polynomials, multivariate_n >> 2, round_challenge_2);
    EXPECT_EQ(first_polynomial[0], expected_val);
}

TYPED_TEST(PartialEvaluationTests, ThreeRoundsGeneric)
{
    using Flavor = TypeParam;
    using FF = typename Flavor::FF;
    using Transcript = typename Flavor::Transcript;

    const size_t multivariate_d(3);
    const size_t multivariate_n(1 << multivariate_d);

    FF v000 = FF::random_element();
    FF v100 = FF::random_element();
    FF v010 = FF::random_element();
    FF v110 = FF::random_element();
    FF v001 = FF::random_element();
    FF v101 = FF::random_element();
    FF v011 = FF::random_element();
    FF v111 = FF::random_element();

    std::array<FF, 8> f0 = { v000, v100, v010, v110, v001, v101, v011, v111 };

    auto full_polynomials = std::array<std::span<FF>, 1>({ f0 });
    auto transcript = Transcript::prover_init_empty();
    auto sumcheck = SumcheckProver<Flavor>(multivariate_n, transcript);

    FF round_challenge_0 = FF::random_element();
    FF expected_q1 = v000 * (FF(1) - round_challenge_0) + v100 * round_challenge_0;
    FF expected_q2 = v010 * (FF(1) - round_challenge_0) + v110 * round_challenge_0;
    FF expected_q3 = v001 * (FF(1) - round_challenge_0) + v101 * round_challenge_0;
    FF expected_q4 = v011 * (FF(1) - round_challenge_0) + v111 * round_challenge_0;

    auto& first_polynomial = sumcheck.partially_evaluated_polynomials.get_all()[0];
    sumcheck.partially_evaluate(full_polynomials, multivariate_n, round_challenge_0);

    EXPECT_EQ(first_polynomial[0], expected_q1);
    EXPECT_EQ(first_polynomial[1], expected_q2);
    EXPECT_EQ(first_polynomial[2], expected_q3);
    EXPECT_EQ(first_polynomial[3], expected_q4);

    FF round_challenge_1 = FF::random_element();
    FF expected_lo = expected_q1 * (FF(1) - round_challenge_1) + expected_q2 * round_challenge_1;
    FF expected_hi = expected_q3 * (FF(1) - round_challenge_1) + expected_q4 * round_challenge_1;

    sumcheck.partially_evaluate(sumcheck.partially_evaluated_polynomials, multivariate_n >> 1, round_challenge_1);
    EXPECT_EQ(first_polynomial[0], expected_lo);
    EXPECT_EQ(first_polynomial[1], expected_hi);

    FF round_challenge_2 = FF::random_element();
    FF expected_val = expected_lo * (FF(1) - round_challenge_2) + expected_hi * round_challenge_2;
    sumcheck.partially_evaluate(sumcheck.partially_evaluated_polynomials, multivariate_n >> 2, round_challenge_2);
    EXPECT_EQ(first_polynomial[0], expected_val);
}

TYPED_TEST(PartialEvaluationTests, ThreeRoundsGenericMultiplePolys)
{
    using Flavor = TypeParam;
    using FF = typename Flavor::FF;
    using Transcript = typename Flavor::Transcript;

    const size_t multivariate_d(3);
    const size_t multivariate_n(1 << multivariate_d);
    std::array<FF, 3> v000;
    std::array<FF, 3> v100;
    std::array<FF, 3> v010;
    std::array<FF, 3> v110;
    std::array<FF, 3> v001;
    std::array<FF, 3> v101;
    std::array<FF, 3> v011;
    std::array<FF, 3> v111;

    for (size_t i = 0; i < 3; i++) {
        v000[i] = FF::random_element();
        v100[i] = FF::random_element();
        v010[i] = FF::random_element();
        v110[i] = FF::random_element();
        v001[i] = FF::random_element();
        v101[i] = FF::random_element();
        v011[i] = FF::random_element();
        v111[i] = FF::random_element();
    }
    std::array<FF, 8> f0 = { v000[0], v100[0], v010[0], v110[0], v001[0], v101[0], v011[0], v111[0] };
    std::array<FF, 8> f1 = { v000[1], v100[1], v010[1], v110[1], v001[1], v101[1], v011[1], v111[1] };
    std::array<FF, 8> f2 = { v000[2], v100[2], v010[2], v110[2], v001[2], v101[2], v011[2], v111[2] };

    auto full_polynomials = std::array<std::span<FF>, 3>{ f0, f1, f2 };
    auto transcript = Transcript::prover_init_empty();
    auto sumcheck = SumcheckProver<Flavor>(multivariate_n, transcript);

    std::array<FF, 3> expected_q1;
    std::array<FF, 3> expected_q2;
    std::array<FF, 3> expected_q3;
    std::array<FF, 3> expected_q4;
    FF round_challenge_0 = FF::random_element();
    for (size_t i = 0; i < 3; i++) {
        expected_q1[i] = v000[i] * (FF(1) - round_challenge_0) + v100[i] * round_challenge_0;
        expected_q2[i] = v010[i] * (FF(1) - round_challenge_0) + v110[i] * round_challenge_0;
        expected_q3[i] = v001[i] * (FF(1) - round_challenge_0) + v101[i] * round_challenge_0;
        expected_q4[i] = v011[i] * (FF(1) - round_challenge_0) + v111[i] * round_challenge_0;
    }

    sumcheck.partially_evaluate(full_polynomials, multivariate_n, round_challenge_0);
    auto polynomial_get_all = sumcheck.partially_evaluated_polynomials.get_all();
    for (size_t i = 0; i < 3; i++) {
        EXPECT_EQ((polynomial_get_all[i])[0], expected_q1[i]);
        EXPECT_EQ((polynomial_get_all[i])[1], expected_q2[i]);
        EXPECT_EQ((polynomial_get_all[i])[2], expected_q3[i]);
        EXPECT_EQ((polynomial_get_all[i])[3], expected_q4[i]);
    }

    FF round_challenge_1 = FF::random_element();
    std::array<FF, 3> expected_lo;
    std::array<FF, 3> expected_hi;
    for (size_t i = 0; i < 3; i++) {
        expected_lo[i] = expected_q1[i] * (FF(1) - round_challenge_1) + expected_q2[i] * round_challenge_1;
        expected_hi[i] = expected_q3[i] * (FF(1) - round_challenge_1) + expected_q4[i] * round_challenge_1;
    }
    sumcheck.partially_evaluate(sumcheck.partially_evaluated_polynomials, multivariate_n >> 1, round_challenge_1);
    for (size_t i = 0; i < 3; i++) {
        EXPECT_EQ((polynomial_get_all[i])[0], expected_lo[i]);
        EXPECT_EQ((polynomial_get_all[i])[1], expected_hi[i]);
    }
    FF round_challenge_2 = FF::random_element();
    std::array<FF, 3> expected_val;
    for (size_t i = 0; i < 3; i++) {
        expected_val[i] = expected_lo[i] * (FF(1) - round_challenge_2) + expected_hi[i] * round_challenge_2;
    }
    sumcheck.partially_evaluate(sumcheck.partially_evaluated_polynomials, multivariate_n >> 2, round_challenge_2);
    for (size_t i = 0; i < 3; i++) {
        EXPECT_EQ((polynomial_get_all[i])[0], expected_val[i]);
    }
}
