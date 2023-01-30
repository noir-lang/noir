#include "relation.hpp"
#include <proof_system/flavor/flavor.hpp>
#include "arithmetic_relation.hpp"
#include "grand_product_initialization_relation.hpp"
#include "grand_product_computation_relation.hpp"
#include "../polynomials/multivariates.hpp"
#include "../polynomials/univariate.hpp"
#include "../polynomials/barycentric_data.hpp"

#include <ecc/curves/bn254/fr.hpp>
#include <numeric/random/engine.hpp>

#include <gtest/gtest.h>

using namespace honk::sumcheck;

namespace honk_relation_tests {

template <class FF> class SumcheckRelation : public testing::Test {
  public:
    template <size_t t> using Univariate = Univariate<FF, t>;
    template <size_t t> using UnivariateView = UnivariateView<FF, t>;

    // TODO(luke): may want to make this more flexible/genericzs
    static std::array<Univariate<6>, bonk::StandardArithmetization::NUM_POLYNOMIALS> compute_mock_extended_edges()
    {
        // TODO(Cody): build from Univariate<2>'s?
        // evaluation form, i.e. w_l(0) = 1, w_l(1) = 2,.. The poly is x+1.
        auto w_l = Univariate<6>({ 1, 2, 3, 4, 5, 6 });
        auto w_r = Univariate<6>({ 1, 2, 3, 4, 5, 6 });
        auto w_o = Univariate<6>({ 1, 2, 3, 4, 5, 6 });
        auto z_perm = Univariate<6>({ 1, 2, 3, 4, 5, 6 });
        // Note: z_perm and z_perm_shift can be any linear poly for the sake of the tests but should not be be equal to
        // each other In order to avoid a trivial computation in the case of the grand_product_computation_relation.
        // Values here were chosen so that output univariates in tests are small positive numbers.
        auto z_perm_shift = Univariate<6>({ 0, 1, 2, 3, 4, 5 }); // this is not real shifted data
        auto q_m = Univariate<6>({ 1, 2, 3, 4, 5, 6 });
        auto q_l = Univariate<6>({ 1, 2, 3, 4, 5, 6 });
        auto q_r = Univariate<6>({ 1, 2, 3, 4, 5, 6 });
        auto q_o = Univariate<6>({ 1, 2, 3, 4, 5, 6 });
        auto q_c = Univariate<6>({ 1, 2, 3, 4, 5, 6 });
        auto sigma_1 = Univariate<6>({ 1, 2, 3, 4, 5, 6 });
        auto sigma_2 = Univariate<6>({ 1, 2, 3, 4, 5, 6 });
        auto sigma_3 = Univariate<6>({ 1, 2, 3, 4, 5, 6 });
        auto id_1 = Univariate<6>({ 1, 2, 3, 4, 5, 6 });
        auto id_2 = Univariate<6>({ 1, 2, 3, 4, 5, 6 });
        auto id_3 = Univariate<6>({ 1, 2, 3, 4, 5, 6 });
        auto lagrange_first = Univariate<6>({ 1, 2, 3, 4, 5, 6 });
        auto lagrange_last = Univariate<6>({ 1, 2, 3, 4, 5, 6 });
        auto pow_zeta = Univariate<6>({ 1, 1, 1, 1, 1, 1 });

        std::array<Univariate<6>, bonk::StandardArithmetization::NUM_POLYNOMIALS> extended_edges = {
            w_l,     w_r,  w_o,  z_perm, z_perm_shift,   q_m,           q_l,     q_r, q_o, q_c, sigma_1, sigma_2,
            sigma_3, id_1, id_2, id_3,   lagrange_first, lagrange_last, pow_zeta
        };
        return extended_edges;
    }
};

using FieldTypes = testing::Types<barretenberg::fr>;
TYPED_TEST_SUITE(SumcheckRelation, FieldTypes);

#define SUMCHECK_RELATION_TYPE_ALIASES using FF = TypeParam;

TYPED_TEST(SumcheckRelation, ArithmeticRelation)
{
    SUMCHECK_RELATION_TYPE_ALIASES

    auto extended_edges = TestFixture::compute_mock_extended_edges();

    auto relation = ArithmeticRelation<FF>();
    using MULTIVARIATE = bonk::StandardArithmetization::POLYNOMIAL;

    // Manually compute the expected edge contribution
    auto w_l = UnivariateView<FF, relation.RELATION_LENGTH>(extended_edges[MULTIVARIATE::W_L]);
    auto w_r = UnivariateView<FF, relation.RELATION_LENGTH>(extended_edges[MULTIVARIATE::W_R]);
    auto w_o = UnivariateView<FF, relation.RELATION_LENGTH>(extended_edges[MULTIVARIATE::W_O]);
    auto q_m = UnivariateView<FF, relation.RELATION_LENGTH>(extended_edges[MULTIVARIATE::Q_M]);
    auto q_l = UnivariateView<FF, relation.RELATION_LENGTH>(extended_edges[MULTIVARIATE::Q_L]);
    auto q_r = UnivariateView<FF, relation.RELATION_LENGTH>(extended_edges[MULTIVARIATE::Q_R]);
    auto q_o = UnivariateView<FF, relation.RELATION_LENGTH>(extended_edges[MULTIVARIATE::Q_O]);
    auto q_c = UnivariateView<FF, relation.RELATION_LENGTH>(extended_edges[MULTIVARIATE::Q_C]);
    auto pow_zeta = UnivariateView<FF, relation.RELATION_LENGTH>(extended_edges[MULTIVARIATE::POW_ZETA]);
    // expected_evals, length 4, extends to { { 5, 22, 57, 116, 205} };
    Univariate expected_evals = pow_zeta * ((q_m * w_r * w_l) + (q_r * w_r) + (q_l * w_l) + (q_o * w_o) + (q_c));

    auto evals = Univariate<FF, relation.RELATION_LENGTH>();
    relation.add_edge_contribution(extended_edges, evals, 0);

    EXPECT_EQ(evals, expected_evals);
};

TYPED_TEST(SumcheckRelation, GrandProductComputationRelation)
{
    SUMCHECK_RELATION_TYPE_ALIASES

    // TODO(luke): Write a test that illustrates the following?
    // Note: the below z_perm_shift = X^2 will fail because it results in a relation of degree 2*1*1*1 = 5 which
    // cannot be represented by 5 points. Therefore when we do the calculation then barycentrically extend, we are
    // effectively exprapolating a 4th degree polynomial instead of the correct 5th degree poly
    // auto z_perm_shift = Univariate<FF, 5>({ 1, 4, 9, 16, 25 }); // X^2

    auto extended_edges = TestFixture::compute_mock_extended_edges();
    auto relation = GrandProductComputationRelation<FF>();
    using UnivariateView = UnivariateView<FF, relation.RELATION_LENGTH>;
    using Univariate = Univariate<FF, relation.RELATION_LENGTH>;

    // Manually compute the expected edge contribution
    using MULTIVARIATE = bonk::StandardArithmetization::POLYNOMIAL;

    auto w_1 = UnivariateView(extended_edges[MULTIVARIATE::W_L]);
    auto w_2 = UnivariateView(extended_edges[MULTIVARIATE::W_R]);
    auto w_3 = UnivariateView(extended_edges[MULTIVARIATE::W_O]);
    auto sigma_1 = UnivariateView(extended_edges[MULTIVARIATE::SIGMA_1]);
    auto sigma_2 = UnivariateView(extended_edges[MULTIVARIATE::SIGMA_2]);
    auto sigma_3 = UnivariateView(extended_edges[MULTIVARIATE::SIGMA_3]);
    auto id_1 = UnivariateView(extended_edges[MULTIVARIATE::ID_1]);
    auto id_2 = UnivariateView(extended_edges[MULTIVARIATE::ID_1]);
    auto id_3 = UnivariateView(extended_edges[MULTIVARIATE::ID_1]);
    auto z_perm = UnivariateView(extended_edges[MULTIVARIATE::Z_PERM]);
    auto z_perm_shift = UnivariateView(extended_edges[MULTIVARIATE::Z_PERM_SHIFT]);
    auto lagrange_first = UnivariateView(extended_edges[MULTIVARIATE::LAGRANGE_FIRST]);
    auto lagrange_last = UnivariateView(extended_edges[MULTIVARIATE::LAGRANGE_LAST]);
    auto pow_zeta = UnivariateView(extended_edges[MULTIVARIATE::POW_ZETA]);

    // TODO(luke): use real transcript/challenges once manifest is done
    FF beta = FF::random_element();
    FF gamma = FF::random_element();
    FF public_input_delta = FF::random_element();
    const RelationParameters<FF> relation_parameters = RelationParameters<FF>{
        .alpha = FF ::zero(), .beta = beta, .gamma = gamma, .public_input_delta = public_input_delta
    };

    auto expected_evals = Univariate();
    // expected_evals in the below step { { 27, 250, 1029, 2916, 6655 } }
    expected_evals += pow_zeta * ((z_perm + lagrange_first) * (w_1 + id_1 * beta + gamma) *
                                  (w_2 + id_2 * beta + gamma) * (w_3 + id_3 * beta + gamma));
    // expected_evals below is { { 27, 125, 343, 729, 1331 } }
    expected_evals -= pow_zeta * ((z_perm_shift + lagrange_last * public_input_delta) * (w_1 + sigma_1 * beta + gamma) *
                                  (w_2 + sigma_2 * beta + gamma) * (w_3 + sigma_3 * beta + gamma));

    auto evals = Univariate();
    relation.add_edge_contribution(extended_edges, evals, relation_parameters);

    EXPECT_EQ(evals, expected_evals);
};

TYPED_TEST(SumcheckRelation, GrandProductInitializationRelation)
{
    SUMCHECK_RELATION_TYPE_ALIASES

    auto extended_edges = TestFixture::compute_mock_extended_edges();
    auto relation = GrandProductInitializationRelation<FF>();
    using UnivariateView = UnivariateView<FF, relation.RELATION_LENGTH>;
    using Univariate = Univariate<FF, relation.RELATION_LENGTH>;

    // Manually compute the expected edge contribution
    using MULTIVARIATE = bonk::StandardArithmetization::POLYNOMIAL;

    auto z_perm_shift = UnivariateView(extended_edges[MULTIVARIATE::Z_PERM_SHIFT]);
    auto lagrange_last = UnivariateView(extended_edges[MULTIVARIATE::LAGRANGE_LAST]);
    auto pow_zeta = UnivariateView(extended_edges[MULTIVARIATE::POW_ZETA]);
    auto expected_evals = pow_zeta * (z_perm_shift * lagrange_last);

    // Compute the edge contribution using add_edge_contribution
    auto evals = Univariate();
    relation.add_edge_contribution(extended_edges, evals, 0);

    EXPECT_EQ(evals, expected_evals);
};

} // namespace honk_relation_tests