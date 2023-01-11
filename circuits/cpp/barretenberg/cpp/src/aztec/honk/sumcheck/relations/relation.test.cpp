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

#pragma GCC diagnostic ignored "-Wunused-variable"
#pragma GCC diagnostic ignored "-Wunused-parameter"

using namespace honk::sumcheck;

namespace honk_relation_tests {

template <class FF> class SumcheckRelation : public testing::Test {
  public:
    template <size_t t> using Univariate = Univariate<FF, t>;
    template <size_t t> using UnivariateView = UnivariateView<FF, t>;

    // TODO(luke): may want to make this more flexible/genericzs
    static std::array<Univariate<5>, proving_system::StandardArithmetization::NUM_POLYNOMIALS>
    compute_mock_extended_edges()
    {
        // TODO(Cody): build from Univariate<2>'s?
        auto w_l = Univariate<5>({ 1, 2, 3, 4, 5 });
        auto w_r = Univariate<5>({ 1, 2, 3, 4, 5 });
        auto w_o = Univariate<5>({ 1, 2, 3, 4, 5 });
        auto z_perm = Univariate<5>({ 1, 2, 3, 4, 5 });
        // Note: z_perm and z_perm_shift can be any linear poly for the sake of the tests but should not be be equal to
        // each other In order to avoid a trivial computation in the case of the grand_product_computation_relation.
        // Values here were chosen so that output univariates in tests are small positive numbers.
        auto z_perm_shift = Univariate<5>({ 0, 1, 2, 3, 4 }); // this is not real shifted data
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

        std::array<Univariate<5>, proving_system::StandardArithmetization::NUM_POLYNOMIALS> extended_edges = {
            w_l, w_r,     w_o,     z_perm,  z_perm_shift, q_m,  q_l,  q_r,       q_o,
            q_c, sigma_1, sigma_2, sigma_3, id_1,         id_2, id_3, lagrange_1
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
    using MULTIVARIATE = proving_system::StandardArithmetization::POLYNOMIAL;

    // Manually compute the expected edge contribution
    auto w_l = UnivariateView<FF, relation.RELATION_LENGTH>(extended_edges[MULTIVARIATE::W_L]);
    auto w_r = UnivariateView<FF, relation.RELATION_LENGTH>(extended_edges[MULTIVARIATE::W_R]);
    auto w_o = UnivariateView<FF, relation.RELATION_LENGTH>(extended_edges[MULTIVARIATE::W_O]);
    auto q_m = UnivariateView<FF, relation.RELATION_LENGTH>(extended_edges[MULTIVARIATE::Q_M]);
    auto q_l = UnivariateView<FF, relation.RELATION_LENGTH>(extended_edges[MULTIVARIATE::Q_L]);
    auto q_r = UnivariateView<FF, relation.RELATION_LENGTH>(extended_edges[MULTIVARIATE::Q_R]);
    auto q_o = UnivariateView<FF, relation.RELATION_LENGTH>(extended_edges[MULTIVARIATE::Q_O]);
    auto q_c = UnivariateView<FF, relation.RELATION_LENGTH>(extended_edges[MULTIVARIATE::Q_C]);
    // expected_evals, length 4, extends to { { 5, 22, 57, 116, 205} };
    Univariate expected_evals = (q_m * w_r * w_l) + (q_r * w_r) + (q_l * w_l) + (q_o * w_o) + (q_c);

    auto evals = Univariate<FF, relation.RELATION_LENGTH>();
    relation.add_edge_contribution(extended_edges, evals);

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
    using MULTIVARIATE = proving_system::StandardArithmetization::POLYNOMIAL;

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
    // auto lagrange_1 = UnivariateView(extended_edges[MULTIVARIATE::LAGRANGE_1]);
    // TODO(luke): use real transcript/challenges once manifest is done
    FF beta = FF::one();
    FF gamma = FF::one();

    auto expected_evals = Univariate();
    // expected_evals is { { 27, 125, 343, 729, 1331 } }
    expected_evals += z_perm * (w_1 + id_1 * beta + gamma) * (w_2 + id_2 * beta + gamma) * (w_3 + id_3 * beta + gamma);
    expected_evals -=
        z_perm_shift * (w_1 + sigma_1 * beta + gamma) * (w_2 + sigma_2 * beta + gamma) * (w_3 + sigma_3 * beta + gamma);

    auto evals = Univariate();
    relation.add_edge_contribution(extended_edges, evals);

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
    using MULTIVARIATE = proving_system::StandardArithmetization::POLYNOMIAL;

    auto z_perm = UnivariateView(extended_edges[MULTIVARIATE::Z_PERM]);
    auto lagrange_1 = UnivariateView(extended_edges[MULTIVARIATE::LAGRANGE_1]);
    // expectede_evals, lenght 3, extends to { { 0, 2, 6, 12, 20 } }
    auto expected_evals = lagrange_1 * (z_perm - FF(1));

    // Compute the edge contribution using add_edge_contribution
    auto evals = Univariate();
    relation.add_edge_contribution(extended_edges, evals);

    EXPECT_EQ(evals, expected_evals);
};

} // namespace honk_relation_tests
