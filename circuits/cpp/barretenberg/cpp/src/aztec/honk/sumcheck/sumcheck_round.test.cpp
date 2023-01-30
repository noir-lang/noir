#include <proof_system/flavor/flavor.hpp>
#include "sumcheck_round.hpp"
#include "relations/arithmetic_relation.hpp"
#include "relations/grand_product_computation_relation.hpp"
#include "relations/grand_product_initialization_relation.hpp"
#include "polynomials/multivariates.hpp"
#include "polynomials/univariate.hpp"
#include <ecc/curves/bn254/fr.hpp>
#include <numeric/random/engine.hpp>

#include <tuple>

#include <common/mem.hpp>
#include <gtest/gtest.h>

using namespace honk;
using namespace honk::sumcheck;

namespace test_sumcheck_round {

TEST(SumcheckRound, ComputeUnivariateProver)
{
    const size_t num_polys(bonk::StandardArithmetization::NUM_POLYNOMIALS);
    // const size_t multivariate_d(1);
    const size_t max_relation_length = 6;

    using FF = barretenberg::fr;
    using Multivariates = ::Multivariates<FF, num_polys>;

    std::array<FF, 2> w_l = { 1, 2 };
    std::array<FF, 2> w_r = { 1, 2 };
    std::array<FF, 2> w_o = { 1, 2 };
    std::array<FF, 2> z_perm = { 1, 2 };
    std::array<FF, 2> z_perm_shift = { 0, 1 }; // chosen to reuse value computed in tests of grand product relations
    std::array<FF, 2> q_m = { 1, 2 };
    std::array<FF, 2> q_l = { 1, 2 };
    std::array<FF, 2> q_r = { 1, 2 };
    std::array<FF, 2> q_o = { 1, 2 };
    std::array<FF, 2> q_c = { 1, 2 };
    std::array<FF, 2> sigma_1 = { 1, 2 };
    std::array<FF, 2> sigma_2 = { 1, 2 };
    std::array<FF, 2> sigma_3 = { 1, 2 };
    std::array<FF, 2> id_1 = { 1, 2 };
    std::array<FF, 2> id_2 = { 1, 2 };
    std::array<FF, 2> id_3 = { 1, 2 };
    std::array<FF, 2> lagrange_first = { 1, 2 };
    std::array<FF, 2> lagrange_last = { 1, 2 };
    std::array<FF, 2> pow_zeta = { 1, 1 };

    std::array<std::span<FF>, bonk::StandardArithmetization::NUM_POLYNOMIALS> full_polynomials = {
        w_l,     w_r,  w_o,  z_perm, z_perm_shift,   q_m,           q_l,     q_r, q_o, q_c, sigma_1, sigma_2,
        sigma_3, id_1, id_2, id_3,   lagrange_first, lagrange_last, pow_zeta
    };

    size_t round_size = 1;

    auto relations = std::tuple(
        ArithmeticRelation<FF>(), GrandProductComputationRelation<FF>(), GrandProductInitializationRelation<FF>());

    // Improvement(Cody): This is ugly? Maye supply some/all of this data through "flavor" class?
    auto round = SumcheckRound<FF,
                               Multivariates::num,
                               ArithmeticRelation,
                               GrandProductComputationRelation,
                               GrandProductInitializationRelation>(round_size, relations);
    const RelationParameters<FF> relation_parameters =
        RelationParameters<FF>{ .alpha = 1, .beta = 1, .gamma = 1, .public_input_delta = 1 };
    Univariate<FF, max_relation_length> round_univariate =
        round.compute_univariate(full_polynomials, relation_parameters);
    Univariate<FF, max_relation_length> expected_round_univariate{ { 32, 149, 406, 857, 1556, 2557 } };
    EXPECT_EQ(round_univariate, expected_round_univariate);
}

TEST(SumcheckRound, ComputeUnivariateVerifier)
{
    const size_t num_polys(bonk::StandardArithmetization::NUM_POLYNOMIALS);
    // const size_t multivariate_d(1);
    // const size_t multivariate_n(1 << multivariate_d);
    // const size_t max_rezlation_length = 5;

    using FF = barretenberg::fr;
    using Multivariates = ::Multivariates<FF, num_polys>;

    FF w_l = { 1 };
    FF w_r = { 2 };
    FF w_o = { 3 };
    // TODO(Cody): compute permutation value?
    FF z_perm = { 0 };
    FF z_perm_shift = { 0 };
    FF q_m = { 4 };
    FF q_l = { 5 };
    FF q_r = { 6 };
    FF q_o = { 7 };
    FF q_c = { 8 };
    FF sigma_1 = { 0 };
    FF sigma_2 = { 0 };
    FF sigma_3 = { 0 };
    FF id_1 = { 0 };
    FF id_2 = { 0 };
    FF id_3 = { 0 };
    FF lagrange_first = { 0 };
    FF lagrange_last = { 0 };
    FF pow_zeta = { 1 };

    // pow_zeta(q_m * w_l * w_r + q_l * w_l + q_r * w_r + q_o * w_o + q_c)
    // = 1 * (4 * 1 * 2 + 5 * 1 + 6 * 2 + 7 * 3 + 8) = 54
    FF expected_full_purported_value = 54;
    std::vector<FF> purported_evaluations = { w_l,     w_r,  w_o,  z_perm, z_perm_shift,   q_m,
                                              q_l,     q_r,  q_o,  q_c,    sigma_1,        sigma_2,
                                              sigma_3, id_1, id_2, id_3,   lagrange_first, lagrange_last,
                                              pow_zeta };

    // size_t round_size = 1;
    auto relations = std::tuple(
        ArithmeticRelation<FF>(), GrandProductComputationRelation<FF>(), GrandProductInitializationRelation<FF>());
    auto round = SumcheckRound<FF,
                               Multivariates::num,
                               ArithmeticRelation,
                               GrandProductComputationRelation,
                               GrandProductInitializationRelation>(relations);
    const RelationParameters<FF> relation_parameters =
        RelationParameters<FF>{ .alpha = -1, .beta = 1, .gamma = 1, .public_input_delta = 1 };
    FF full_purported_value =
        round.compute_full_honk_relation_purported_value(purported_evaluations, relation_parameters);
    EXPECT_EQ(full_purported_value, expected_full_purported_value);
}

} // namespace test_sumcheck_round
