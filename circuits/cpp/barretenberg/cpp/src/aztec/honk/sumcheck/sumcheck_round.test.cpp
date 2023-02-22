#include <proof_system/flavor/flavor.hpp>
#include "sumcheck_round.hpp"
#include "relations/arithmetic_relation.hpp"
#include "relations/grand_product_computation_relation.hpp"
#include "relations/grand_product_initialization_relation.hpp"
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
    const size_t NUM_POLYNOMIALS(bonk::StandardArithmetization::NUM_POLYNOMIALS);
    // const size_t multivariate_d(1);
    const size_t max_relation_length = 5;

    using FF = barretenberg::fr;

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

    std::array<std::span<FF>, NUM_POLYNOMIALS> full_polynomials;
    using POLYNOMIAL = bonk::StandardArithmetization::POLYNOMIAL;
    full_polynomials[POLYNOMIAL::W_L] = w_l;
    full_polynomials[POLYNOMIAL::W_R] = w_r;
    full_polynomials[POLYNOMIAL::W_O] = w_o;
    full_polynomials[POLYNOMIAL::Z_PERM] = z_perm;
    full_polynomials[POLYNOMIAL::Z_PERM_SHIFT] = z_perm_shift;
    full_polynomials[POLYNOMIAL::Q_M] = q_m;
    full_polynomials[POLYNOMIAL::Q_L] = q_l;
    full_polynomials[POLYNOMIAL::Q_R] = q_r;
    full_polynomials[POLYNOMIAL::Q_O] = q_o;
    full_polynomials[POLYNOMIAL::Q_C] = q_c;
    full_polynomials[POLYNOMIAL::SIGMA_1] = sigma_1;
    full_polynomials[POLYNOMIAL::SIGMA_2] = sigma_2;
    full_polynomials[POLYNOMIAL::SIGMA_3] = sigma_3;
    full_polynomials[POLYNOMIAL::ID_1] = id_1;
    full_polynomials[POLYNOMIAL::ID_2] = id_2;
    full_polynomials[POLYNOMIAL::ID_3] = id_3;
    full_polynomials[POLYNOMIAL::LAGRANGE_FIRST] = lagrange_first;
    full_polynomials[POLYNOMIAL::LAGRANGE_LAST] = lagrange_last;

    size_t round_size = 1;

    auto relations = std::tuple(
        ArithmeticRelation<FF>(), GrandProductComputationRelation<FF>(), GrandProductInitializationRelation<FF>());

    // Improvement(Cody): This is ugly? Maye supply some/all of this data through "flavor" class?
    auto round = SumcheckRound<FF,
                               NUM_POLYNOMIALS,
                               ArithmeticRelation,
                               GrandProductComputationRelation,
                               GrandProductInitializationRelation>(round_size, relations);
    const RelationParameters<FF> relation_parameters =
        RelationParameters<FF>{ .zeta = 2, .alpha = 1, .beta = 1, .gamma = 1, .public_input_delta = 1 };

    PowUnivariate<FF> pow_univariate(relation_parameters.zeta);
    Univariate<FF, max_relation_length> round_univariate =
        round.compute_univariate(full_polynomials, relation_parameters, pow_univariate);
    Univariate<FF, max_relation_length> expected_round_univariate{ { 32, 149, 406, 857, 1556 } };
    EXPECT_EQ(round_univariate, expected_round_univariate);
}

TEST(SumcheckRound, ComputeUnivariateVerifier)
{
    const size_t NUM_POLYNOMIALS(bonk::StandardArithmetization::NUM_POLYNOMIALS);
    // const size_t multivariate_d(1);
    // const size_t multivariate_n(1 << multivariate_d);
    // const size_t max_rezlation_length = 5;

    using FF = barretenberg::fr;

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

    // q_m * w_l * w_r + q_l * w_l + q_r * w_r + q_o * w_o + q_c
    // = 1 * (4 * 1 * 2 + 5 * 1 + 6 * 2 + 7 * 3 + 8) = 54
    FF expected_full_purported_value = 54;

    std::vector<FF> purported_evaluations;
    purported_evaluations.resize(NUM_POLYNOMIALS);

    using POLYNOMIAL = bonk::StandardArithmetization::POLYNOMIAL;
    purported_evaluations[POLYNOMIAL::W_L] = w_l;
    purported_evaluations[POLYNOMIAL::W_R] = w_r;
    purported_evaluations[POLYNOMIAL::W_O] = w_o;
    purported_evaluations[POLYNOMIAL::Z_PERM] = z_perm;
    purported_evaluations[POLYNOMIAL::Z_PERM_SHIFT] = z_perm_shift;
    purported_evaluations[POLYNOMIAL::Q_M] = q_m;
    purported_evaluations[POLYNOMIAL::Q_L] = q_l;
    purported_evaluations[POLYNOMIAL::Q_R] = q_r;
    purported_evaluations[POLYNOMIAL::Q_O] = q_o;
    purported_evaluations[POLYNOMIAL::Q_C] = q_c;
    purported_evaluations[POLYNOMIAL::SIGMA_1] = sigma_1;
    purported_evaluations[POLYNOMIAL::SIGMA_2] = sigma_2;
    purported_evaluations[POLYNOMIAL::SIGMA_3] = sigma_3;
    purported_evaluations[POLYNOMIAL::ID_1] = id_1;
    purported_evaluations[POLYNOMIAL::ID_2] = id_2;
    purported_evaluations[POLYNOMIAL::ID_3] = id_3;
    purported_evaluations[POLYNOMIAL::LAGRANGE_FIRST] = lagrange_first;
    purported_evaluations[POLYNOMIAL::LAGRANGE_LAST] = lagrange_last;

    // size_t round_size = 1;
    auto relations = std::tuple(
        ArithmeticRelation<FF>(), GrandProductComputationRelation<FF>(), GrandProductInitializationRelation<FF>());
    auto round = SumcheckRound<FF,
                               NUM_POLYNOMIALS,
                               ArithmeticRelation,
                               GrandProductComputationRelation,
                               GrandProductInitializationRelation>(relations);
    const RelationParameters<FF> relation_parameters =
        RelationParameters<FF>{ .zeta = 2, .alpha = -1, .beta = 1, .gamma = 1, .public_input_delta = 1 };
    PowUnivariate<FF> pow_univariate(relation_parameters.zeta);
    FF full_purported_value =
        round.compute_full_honk_relation_purported_value(purported_evaluations, relation_parameters, pow_univariate);
    EXPECT_EQ(full_purported_value, expected_full_purported_value);
}

} // namespace test_sumcheck_round
