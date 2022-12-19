#include "./transcript.hpp"
#include "./sumcheck_round.hpp"
#include "./sumcheck_types/constraint.hpp"
#include "./sumcheck_types/arithmetic_constraint.hpp"
#include "./sumcheck_types/multivariates.hpp"
#include "./sumcheck_types/univariate.hpp"
#include "./sumcheck_types/challenge_container.hpp"
#include <ecc/curves/bn254/fr.hpp>
#include <numeric/random/engine.hpp>

#include <tuple>

#include <common/mem.hpp>
#include <gtest/gtest.h>

#pragma GCC diagnostic ignored "-Wunused-variable"
#pragma GCC diagnostic ignored "-Wunused-parameter"

using namespace honk;
using namespace honk::sumcheck;

template <class Fr> class MockTranscript : public Transcript<Fr> {
  public:
    Fr get_challenge() { return mock_challenge; };
    Fr mock_challenge = -1;
};

namespace test_sumcheck_round {

TEST(Sumcheck, ComputeUnivariateProverMock)
{
    // arithmetic constraint G is deegree 3 in 8 variables
    // G(Y1, ..., Y8) = Y4Y1Y2 + Y5Y1 + Y6Y2 + Y7Y3 + Y8
    const size_t num_polys(MULTIVARIATE::COUNT);
    const size_t multivariate_d(1);
    const size_t multivariate_n(1 << multivariate_d);
    const size_t constraint_degree_plus_one = 5; // TODO(cody) extract from widget

    class SumcheckTypes {
      public:
        using Fr = barretenberg::fr;
        // using Univariate = Univariate;
        using Multivariates = ::Multivariates<Fr, num_polys, multivariate_d>;
        using ChallengeContainer =
            ::ChallengeContainer<Fr, MockTranscript<Fr>, Univariate<Fr, constraint_degree_plus_one>>;
    };

    SumcheckTypes::Fr w_l[2] = { 1, 2 };
    SumcheckTypes::Fr w_r[2] = { 1, 2 };
    SumcheckTypes::Fr w_o[2] = { 1, 2 };
    SumcheckTypes::Fr z_perm[2] = { 1, 2 };
    SumcheckTypes::Fr z_perm_shift[2] = { 1, 2 };
    SumcheckTypes::Fr q_m[2] = { 1, 2 };
    SumcheckTypes::Fr q_l[2] = { 1, 2 };
    SumcheckTypes::Fr q_r[2] = { 1, 2 };
    SumcheckTypes::Fr q_o[2] = { 1, 2 };
    SumcheckTypes::Fr q_c[2] = { 1, 2 };
    SumcheckTypes::Fr sigma_1[2] = { 1, 2 };
    SumcheckTypes::Fr sigma_2[2] = { 1, 2 };
    SumcheckTypes::Fr sigma_3[2] = { 1, 2 };
    SumcheckTypes::Fr id_1[2] = { 1, 2 };
    SumcheckTypes::Fr id_2[2] = { 1, 2 };
    SumcheckTypes::Fr id_3[2] = { 1, 2 };
    SumcheckTypes::Fr lagrange_1[2] = { 1, 2 };

    std::array<SumcheckTypes::Fr*, SumcheckTypes::Multivariates::num> full_polynomials({ w_l,
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

    auto multivariates = SumcheckTypes::Multivariates(full_polynomials);

    auto transcript = MockTranscript<SumcheckTypes::Fr>(); // actually a shared pointer to a transcript?
    auto challenges = SumcheckTypes::ChallengeContainer(transcript);

    size_t round_size = 1;
    // call SumcheckRound with one constraint
    auto round = SumcheckRound<SumcheckTypes, ArithmeticConstraint>(multivariates, challenges);
    Univariate<SumcheckTypes::Fr, constraint_degree_plus_one> restriction =
        round.compute_initial_univariate_restriction(multivariates, challenges);
    Univariate<SumcheckTypes::Fr, constraint_degree_plus_one> expected_restriction{ { 5, 22, 57, 116, 205 } };
    EXPECT_EQ(restriction, expected_restriction);
}

TEST(Sumcheck, ComputeUnivariateVerifierMock)
{
    // arithmetic constraint G is deegree 3 in 8 variables
    // G(Y1, ..., Y8) = Y4Y1Y2 + Y5Y1 + Y6Y2 + Y7Y3 + Y8
    const size_t num_polys(MULTIVARIATE::COUNT);
    const size_t multivariate_d(1);
    const size_t multivariate_n(1 << multivariate_d);
    const size_t constraint_degree_plus_one = 5; // TODO(cody) extract from widget

    class SumcheckTypes {
      public:
        using Fr = barretenberg::fr;
        // using Univariate = Univariate;
        using Multivariates = ::Multivariates<Fr, num_polys, multivariate_d>;
        using ChallengeContainer =
            ::ChallengeContainer<Fr, MockTranscript<Fr>, Univariate<Fr, constraint_degree_plus_one>>;
    };

    SumcheckTypes::Fr w_l = { 1 };
    SumcheckTypes::Fr w_r = { 2 };
    SumcheckTypes::Fr w_o = { 3 };
    SumcheckTypes::Fr z_perm = { 0 };
    SumcheckTypes::Fr z_perm_shift = { 0 };
    SumcheckTypes::Fr q_m = { 4 };
    SumcheckTypes::Fr q_l = { 5 };
    SumcheckTypes::Fr q_r = { 6 };
    SumcheckTypes::Fr q_o = { 7 };
    SumcheckTypes::Fr q_c = { 8 };
    SumcheckTypes::Fr sigma_1 = { 0 };
    SumcheckTypes::Fr sigma_2 = { 0 };
    SumcheckTypes::Fr sigma_3 = { 0 };
    SumcheckTypes::Fr id_1 = { 0 };
    SumcheckTypes::Fr id_2 = { 0 };
    SumcheckTypes::Fr id_3 = { 0 };
    SumcheckTypes::Fr lagrange_1 = { 0 };

    // 4 * 1 * 2 + 5 * 1 + 6 * 2 + 7 * 3 + 8 = 54
    SumcheckTypes::Fr expected_full_purported_value = 54;
    std::array<SumcheckTypes::Fr, SumcheckTypes::Multivariates::num> purported_evaluations({ w_l,
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

    auto transcript = MockTranscript<SumcheckTypes::Fr>(); // actually a shared pointer to a transcript?
    auto challenges = SumcheckTypes::ChallengeContainer(transcript);

    size_t round_size = 1;
    // call SumcheckRound with one constraint
    auto round = SumcheckRound<SumcheckTypes, ArithmeticConstraint>(purported_evaluations, challenges);
    SumcheckTypes::Fr full_purported_value = round.compute_full_honk_constraint_purported_value(challenges);
    EXPECT_EQ(full_purported_value, expected_full_purported_value);
}

// TEST(sumcheck, round)
// {
//     // arithmetic constraint G is deegree 3 in 8 variables
//     // G(Y1, ..., Y8) = Y4Y1Y2 + Y5Y1 + Y6Y2 + Y7Y3 + Y8
//     const size_t num_polys(MULTIVARIATE::COUNT);
//     const size_t multivariate_d(2);
//     const size_t multivariate_n(1 << multivariate_d);
//     const size_t constraint_degree_plus_one = 5;

//     using Fr = barretenberg::fr;
//     using Edge = Edge<Fr>;
//     using EdgeGroup = EdgeGroup<Fr, num_polys>;
//     using Multivariates = Multivariates<Fr, num_polys, multivariate_d>;
//     using Univariate = Univariate<Fr, constraint_degree_plus_one>;
//     using ChallengeContainer = ChallengeContainer<Fr, MockTranscript<Fr>, Univariate>;

//     class SumcheckTypes {
//       public:
//         using Fr = Fr;
//         using EdgeGroup = EdgeGroup;
//         using Multivariates = Multivariates;
//         using ChallengeContainer = ChallengeContainer;
//         using Univariate = Univariate;
//     };

//     // TODO(cody): move this out of round.
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
//     auto constraints = std::make_tuple(ArithmeticConstraint<Fr>());
//     auto transcript = MockTranscript<Fr>(); // actually a shared pointer to a transcript?
//     auto challenges = ChallengeContainer(transcript);

//     auto round = SumcheckRound<SumcheckTypes, ArithmeticConstraint>(polynomials, constraints, challenges);
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

// TEST(sumcheck, round_from_pointers)
// {
//     // arithmetic constraint G is deegree 3 in 8 variables
//     // G(Y1, ..., Y8) = Y4Y1Y2 + Y5Y1 + Y6Y2 + Y7Y3 + Y8
//     const size_t num_polys(MULTIVARIATE::COUNT);
//     const size_t multivariate_d(2);
//     const size_t multivariate_n(1 << multivariate_d);
//     const size_t constraint_degree_plus_one = 5;

//     using Fr = barretenberg::fr;
//     using Edge = Edge<Fr>;
//     using EdgeGroup = EdgeGroup<Fr, num_polys>;
//     using Multivariates = Multivariates<Fr, num_polys, multivariate_d>;
//     using ChallengeContainer = ChallengeContainer<Fr, MockTranscript<Fr>, Univariate<Fr,
//     constraint_degree_plus_one>>;

//     class SumcheckTypes {
//       public:
//         using Fr = Fr;
//         using EdgeGroup = EdgeGroup;
//         using Multivariates = Multivariates;
//         using ChallengeContainer = ChallengeContainer;
//         using Univariate = Univariate<Fr, constraint_degree_plus_one>;
//     };

//     Fr w_l[4] = { 1, 2, 7, 8 };
//     Fr w_r[4] = { 1, 2, 7, 8 };
//     Fr w_o[4] = { 1, 2, 7, 8 };
//     Fr q_l[4] = { 1, 2, 7, 8 };
//     Fr q_m[4] = { 1, 2, 7, 8 };
//     Fr q_r[4] = { 1, 2, 7, 8 };
//     Fr q_o[4] = { 1, 2, 7, 8 };
//     Fr q_c[4] = { 1, 2, 7, 8 };

//     std::array<Fr*, num_polys> input_polys = { w_l, w_r, w_o, q_l, q_m, q_r, q_o, q_c };

//     auto polynomials = Multivariates(input_polys);
//     auto constraints = std::make_tuple(ArithmeticConstraint<Fr>());
//     auto transcript = MockTranscript<Fr>(); // actually a shared pointer to a transcript?
//     auto challenges = ChallengeContainer(transcript);

//     auto round = SumcheckRound<SumcheckTypes, ArithmeticConstraint>(polynomials, constraints, challenges);
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
//     //     EdgeGroup expected_group0({ Edge({ 0, 6 }),
//     //                                 Edge({ 0, 6 }),
//     //                                 Edge({ 0, 6 }),
//     //                                 Edge({ 0, 6 }),
//     //                                 Edge({ 0, 6 }),
//     //                                 Edge({ 0, 6 }),
//     //                                 Edge({ 0, 6 }),
//     //                                 Edge({ 0, 6 }) });
//     //     EXPECT_EQ(expected_group0, round.polynomials.groups[0]);
//     //     ASSERT_FALSE(round.failed);
//     //     EXPECT_EQ(round.round_size, 1);
// }
} // namespace test_sumcheck_round
