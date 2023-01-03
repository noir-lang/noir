#include "../flavor/flavor.hpp"
#include "sumcheck_round.hpp"
#include "relations/relation.hpp"
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
    const size_t num_polys(StandardArithmetization::NUM_POLYNOMIALS);
    const size_t multivariate_d(1);
    const size_t max_relation_length = 5;

    using FF = barretenberg::fr;
    using Multivariates = ::Multivariates<FF, num_polys, multivariate_d>;

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
    std::array<FF, 2> lagrange_1 = { 1, 2 };

    std::array<std::span<FF>, StandardArithmetization::NUM_POLYNOMIALS> full_polynomials = {
        w_l, w_r,     w_o,     z_perm,  z_perm_shift, q_m,  q_l,  q_r,       q_o,
        q_c, sigma_1, sigma_2, sigma_3, id_1,         id_2, id_3, lagrange_1
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
    FF relation_separator_challenge = 1;
    Univariate<FF, max_relation_length> round_univariate =
        round.compute_univariate(full_polynomials, relation_separator_challenge);
    Univariate<FF, max_relation_length> expected_round_univariate{ { 32, 149, 406, 857, 1556 } };

    EXPECT_EQ(round_univariate, expected_round_univariate);
}

TEST(SumcheckRound, ComputeUnivariateVerifier)
{
    const size_t num_polys(StandardArithmetization::NUM_POLYNOMIALS);
    const size_t multivariate_d(1);
    const size_t multivariate_n(1 << multivariate_d);
    const size_t max_relation_length = 5;

    using FF = barretenberg::fr;
    using Multivariates = ::Multivariates<FF, num_polys, multivariate_d>;

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
    FF lagrange_1 = { 0 };

    // 4 * 1 * 2 + 5 * 1 + 6 * 2 + 7 * 3 + 8 = 54
    FF expected_full_purported_value = 54;
    std::vector<FF> purported_evaluations = { w_l, w_r,     w_o,     z_perm,  z_perm_shift, q_m,  q_l,  q_r,       q_o,
                                              q_c, sigma_1, sigma_2, sigma_3, id_1,         id_2, id_3, lagrange_1 };

    size_t round_size = 1;
    auto relations = std::tuple(
        ArithmeticRelation<FF>(), GrandProductComputationRelation<FF>(), GrandProductInitializationRelation<FF>());
    auto round = SumcheckRound<FF,
                               Multivariates::num,
                               ArithmeticRelation,
                               GrandProductComputationRelation,
                               GrandProductInitializationRelation>(relations);
    FF relation_separator_challenge = -1;
    FF full_purported_value =
        round.compute_full_honk_relation_purported_value(purported_evaluations, relation_separator_challenge);
    EXPECT_EQ(full_purported_value, expected_full_purported_value);
}

// TODO(Cody): Implement this and better tests.
// TEST(sumcheck, round)
// {
//     // arithmetic relation G is deegree 3 in 8 variables
//     // G(Y1, ..., Y8) = Y4Y1Y2 + Y5Y1 + Y6Y2 + Y7Y3 + Y8
//     const size_t num_polys(StandardArithmetization::NUM_POLYNOMIALS);
//     const size_t multivariate_d(2);
//     const size_t multivariate_n(1 << multivariate_d);
//     const size_t max_relation_length = 5;

//     using Fr = barretenberg::fr;
//     using Edge = Edge<Fr>;
//     using EdgeGroup = EdgeGroup<Fr, num_polys>;
//     using Multivariates = Multivariates<Fr, num_polys, multivariate_d>;
//     using Univariate = Univariate<Fr, max_relation_length>;
//     // TODO(Cody): move this out of round.
//     EdgeGroup group0({ Edge({ 1, 2 }),
//                        Edge({ 1, 2 }),
//                        Edge({ 1, 2 }),
//                        Edge({ 1, 2 }),
//                        Edge({ 1, 2 }),
//                        Edge({ 1, 2 }),
//                        Edge({ 1, 2 }),
//                        Edge({ 1, 2 }) });

//     EdgeGroup group1({ Edge({ 7, 8 }),
//                        Edge({ 7, 8 }),
//                        Edge({ 7, 8 }),
//                        Edge({ 7, 8 }),
//                        Edge({ 7, 8 }),
//                        Edge({ 7, 8 }),
//                        Edge({ 7, 8 }),
//                        Edge({ 7, 8 }) });

//     auto polynomials = Multivariates({ group0, group1 });
//     auto relations = std::make_tuple(ArithmeticRelation<Fr>());

//     auto round = SumcheckRound<..., ArithmeticRelation>(polynomials, relations);
//     // The values of the univariate restriction S2 created in the first round
//     // are the sum of a contribution form group0 and a contribution from group1.
//     // Using Sage;
//     //    group0 contributes: [5, 22, 57, 116]
//     //    group1 contributes: [497, 712, 981, 1310]
//     // Therefore the values of S2 on {0, 1, 2, 3} are: [502, 734, 1038, 1426]
//     // and S2(0) + S2(1) = 502 + 734 = 1236
//     round.target_total_sum = 1236;
//     EXPECT_EQ(round.round_size, 2);
//     /*
//     Folding with u2 = -1
//     2 -------- 8
//     |          |
//     |    Yi    |
//     | i=1...8  |
//     1 -------- 7        ~~>  0 -------- 6
//     (2(1-X1)+8X1)  X2         0(1-X1)+6X1
//    +(1(1-X1)+7X1)(1-X2)
//  */
//     round.execute();
//     EdgeGroup expected_group0({ Edge({ 0, 6 }),
//                                 Edge({ 0, 6 }),
//                                 Edge({ 0, 6 }),
//                                 Edge({ 0, 6 }),
//                                 Edge({ 0, 6 }),
//                                 Edge({ 0, 6 }),
//                                 Edge({ 0, 6 }),
//                                 Edge({ 0, 6 }) });
//     EXPECT_EQ(expected_group0, round.polynomials.groups[0]);
//     ASSERT_FALSE(round.failed);
//     EXPECT_EQ(round.round_size, 1);
// }

} // namespace test_sumcheck_round
