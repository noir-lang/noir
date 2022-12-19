#include "./constraint.hpp"
#include "./arithmetic_constraint.hpp"
#include "./grand_product_initialization_constraint.hpp"
#include "./grand_product_computation_constraint.hpp"
#include "./multivariates.hpp"
#include "./univariate.hpp"
#include "./challenge_container.hpp"
#include <ecc/curves/bn254/fr.hpp>
#include <numeric/random/engine.hpp>
#include "../transcript.hpp"

#include <common/mem.hpp>
#include <gtest/gtest.h>

#pragma GCC diagnostic ignored "-Wunused-variable"
#pragma GCC diagnostic ignored "-Wunused-parameter"

using namespace honk::sumcheck;

namespace honk {
namespace sumcheck {

// template <class Fr> class MockTranscript : public Transcript<Fr> {
//   public:
//     Fr get_challenge() { return mock_challenge; };
//     Fr mock_challenge = -1;
// };

// field is named Fscalar here because of clash with the Fr expected to be
// part of ConstraintTypes. Got impatient; should find better way
template <class Fscalar> class sumcheck_constraint : public testing::Test {
    template <size_t t> using Univariate = Univariate<Fscalar, t>;
    template <size_t t> using UnivariateView = UnivariateView<Fscalar, t>;
    // template <size_t t> using ChallengeContainer = ChallengeContainer<Fscalar, Transcript<Fscalar>, Univariate<t>>;

    template <size_t group_size, size_t univariate_degree> class ExampleConstraintTypes {
      public:
        using Fr = Fscalar;
        // using Univariate = Univariate<univariate_degree>;
        // using ChallengeContainer = ChallengeContainer<univariate_degree>;
    };

    // TODO(luke): may want to make this more flexible/generic
    static std::array<Univariate<5>, MULTIVARIATE::COUNT> compute_mock_edge_extensions()
    {
        // TODO(cody): build from Univariate<2>'s?
        auto w_l = Univariate<5>({ 1, 2, 3, 4, 5 });
        auto w_r = Univariate<5>({ 1, 2, 3, 4, 5 });
        auto w_o = Univariate<5>({ 1, 2, 3, 4, 5 });
        auto z_perm = Univariate<5>({ 1, 2, 3, 4, 5 });
        // Note: z_perm_shift can be any linear poly for the sake of the tests but should not be equal to z_perm to
        // avoid a trivial computation in the case of the grand_product_computation_constraint
        auto z_perm_shift = Univariate<5>({ 2, 1, 0, -1, -2 });
        auto q_m = Univariate<5>({ 1, 2, 3, 4, 5 });
        auto q_l = Univariate<5>({ 1, 2, 3, 4, 5 });
        auto q_r = Univariate<5>({ 1, 2, 3, 4, 5 });
        auto q_o = Univariate<5>({ 1, 2, 3, 4, 5 });
        auto q_c = Univariate<5>({ 1, 2, 3, 4, 5 });
        auto sigma_1 = Univariate<5>({ 1, 2, 3, 4, 5 });
        auto sigma_2 = Univariate<5>({ 1, 2, 3, 4, 5 });
        auto sigma_3 = Univariate<5>({ 1, 2, 3, 4, 5 });
        auto id_1 = Univariate<5>({ 1, 2, 3, 4, 5 });
        auto id_2 = Univariate<5>({ 1, 2, 3, 4, 5 });
        auto id_3 = Univariate<5>({ 1, 2, 3, 4, 5 });
        auto lagrange_1 = Univariate<5>({ 1, 2, 3, 4, 5 });

        std::array<Univariate<5>, MULTIVARIATE::COUNT> edge_extensions({ w_l,
                                                                         w_r,
                                                                         w_o,
                                                                         z_perm,
                                                                         z_perm_shift,
                                                                         q_m,
                                                                         q_l,
                                                                         q_r,
                                                                         q_o,
                                                                         q_c,
                                                                         sigma_1,
                                                                         sigma_2,
                                                                         sigma_3,
                                                                         id_1,
                                                                         id_2,
                                                                         id_3,
                                                                         lagrange_1 });
        return edge_extensions;
    }

  public:
    static void test_arithmetic_constraint()
    {
        auto edge_extensions = compute_mock_edge_extensions();

        auto constraint = ArithmeticConstraint<Fscalar>();
        using UnivariateView = UnivariateView<constraint.CONSTRAINT_LENGTH>;
        using Univariate = Univariate<constraint.CONSTRAINT_LENGTH>;

        // Manually compute the expected edge contribution
        auto w_l = UnivariateView(edge_extensions[MULTIVARIATE::W_L]);
        auto w_r = UnivariateView(edge_extensions[MULTIVARIATE::W_R]);
        auto w_o = UnivariateView(edge_extensions[MULTIVARIATE::W_O]);
        auto q_m = UnivariateView(edge_extensions[MULTIVARIATE::Q_M]);
        auto q_l = UnivariateView(edge_extensions[MULTIVARIATE::Q_L]);
        auto q_r = UnivariateView(edge_extensions[MULTIVARIATE::Q_R]);
        auto q_o = UnivariateView(edge_extensions[MULTIVARIATE::Q_O]);
        auto q_c = UnivariateView(edge_extensions[MULTIVARIATE::Q_C]);
        Univariate expected_evals = (q_m * w_r * w_l) + (q_r * w_r) + (q_l * w_l) + (q_o * w_o) + (q_c);
        // Univariate<constraint.CONSTRAINT_LENGTH> expected_evals{ { 5, 22, 57, 116 } };

        auto evals = Univariate();
        constraint.add_edge_contribution(edge_extensions, evals);

        EXPECT_EQ(evals, expected_evals);
    };

    static void test_grand_product_computation_constraint()
    {
        using ChallengeContainer = ChallengeContainer<Fscalar, Transcript<Fscalar>, Univariate<5>>;

        // TODO(luke): Write a test that illustrates the following?
        // Note: the below z_perm_shift = X^2 will fail because it results in a constraint of degree 2*1*1*1 = 5 which
        // cannot be represented by 5 points. Therefore when we do the calculation then barycentrically extend, we are
        // effectively exprapolating a 4th degree polynomial instead of the correct 5th degree poly
        // auto z_perm_shift = Univariate<5>({ 1, 4, 9, 16, 25 }); // X^2

        auto edge_extensions = compute_mock_edge_extensions();
        auto constraint = GrandProductComputationConstraint<Fscalar>();
        auto transcript = Transcript<Fscalar>();
        auto challenges = ChallengeContainer(transcript);
        using UnivariateView = UnivariateView<constraint.CONSTRAINT_LENGTH>;
        using Univariate = Univariate<constraint.CONSTRAINT_LENGTH>;

        // Manually compute the expected edge contribution
        auto w_1 = UnivariateView(edge_extensions[MULTIVARIATE::W_L]);
        auto w_2 = UnivariateView(edge_extensions[MULTIVARIATE::W_R]);
        auto w_3 = UnivariateView(edge_extensions[MULTIVARIATE::W_O]);
        auto sigma_1 = UnivariateView(edge_extensions[MULTIVARIATE::SIGMA_1]);
        auto sigma_2 = UnivariateView(edge_extensions[MULTIVARIATE::SIGMA_2]);
        auto sigma_3 = UnivariateView(edge_extensions[MULTIVARIATE::SIGMA_3]);
        auto id_1 = UnivariateView(edge_extensions[MULTIVARIATE::ID_1]);
        auto id_2 = UnivariateView(edge_extensions[MULTIVARIATE::ID_1]);
        auto id_3 = UnivariateView(edge_extensions[MULTIVARIATE::ID_1]);
        auto z_perm = UnivariateView(edge_extensions[MULTIVARIATE::Z_PERM]);
        auto z_perm_shift = UnivariateView(edge_extensions[MULTIVARIATE::Z_PERM_SHIFT]);
        // auto lagrange_1 = UnivariateView(edge_extensions[MULTIVARIATE::LAGRANGE_1]);
        // TODO(luke): use real transcript/challenges
        Fscalar beta = challenges.get_challenge_equals_one();
        Fscalar gamma = challenges.get_challenge_equals_one();

        auto expected_evals = Univariate();
        expected_evals +=
            z_perm * (w_1 + id_1 * beta + gamma) * (w_2 + id_2 * beta + gamma) * (w_3 + id_3 * beta + gamma);
        expected_evals -= z_perm_shift * (w_1 + sigma_1 * beta + gamma) * (w_2 + sigma_2 * beta + gamma) *
                          (w_3 + sigma_3 * beta + gamma);

        auto evals = Univariate();
        constraint.add_edge_contribution(edge_extensions, evals, challenges);

        EXPECT_EQ(evals, expected_evals);
    };

    static void test_grand_product_initialization_constraint()
    {
        auto edge_extensions = compute_mock_edge_extensions();
        auto constraint = GrandProductInitializationConstraint<Fscalar>();
        using UnivariateView = UnivariateView<constraint.CONSTRAINT_LENGTH>;
        using Univariate = Univariate<constraint.CONSTRAINT_LENGTH>;

        // Manually compute the expected edge contribution
        auto z_perm = UnivariateView(edge_extensions[MULTIVARIATE::Z_PERM]);
        auto lagrange_1 = UnivariateView(edge_extensions[MULTIVARIATE::LAGRANGE_1]);
        auto expected_evals = lagrange_1 * (z_perm - Fscalar(1));

        // Compute the edge contribution using add_edge_contribution
        auto evals = Univariate();
        constraint.add_edge_contribution(edge_extensions, evals);

        EXPECT_EQ(evals, expected_evals);
    };
};

typedef testing::Types<barretenberg::fr> FieldTypes;
TYPED_TEST_SUITE(sumcheck_constraint, FieldTypes);

TYPED_TEST(sumcheck_constraint, arithmetic_constraint)
{
    TestFixture::test_arithmetic_constraint();
}

TYPED_TEST(sumcheck_constraint, grand_product_computation_constraint)
{
    TestFixture::test_grand_product_computation_constraint();
}

TYPED_TEST(sumcheck_constraint, grand_product_initialization_constraint)
{
    TestFixture::test_grand_product_initialization_constraint();
}
} // namespace sumcheck
} // namespace honk
