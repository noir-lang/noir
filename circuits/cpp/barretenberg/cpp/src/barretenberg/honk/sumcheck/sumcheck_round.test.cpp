#include "barretenberg/honk/flavor/flavor.hpp"
#include "sumcheck_round.hpp"
#include "relations/arithmetic_relation.hpp"
#include "relations/grand_product_computation_relation.hpp"
#include "relations/grand_product_initialization_relation.hpp"
#include "polynomials/univariate.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/numeric/random/engine.hpp"

#include <tuple>

#include "barretenberg/common/mem.hpp"
#include <gtest/gtest.h>
/**
 * We want to test if the univariate (S_l in the thesis) computed by the prover in a particular round is correct. We
 * also want to verify given the purported evaluations of all the relevant polynomials, the verifer can correctly verify
 * the purported evaluation of S_l. For the prover, we use a couple of methods to compute the univariate by the sumcheck
 * method `compute_univariate` and by step by step manual computation respectively. For the verifier, we follow a
 * similar approach.
 */

using namespace proof_system::honk;
using namespace proof_system::honk::sumcheck;
const size_t max_relation_length = 5;
const size_t input_polynomial_length = 2;
using FF = barretenberg::fr;
const size_t NUM_POLYNOMIALS = proof_system::honk::StandardArithmetization::NUM_POLYNOMIALS;
using POLYNOMIAL = proof_system::honk::StandardArithmetization::POLYNOMIAL;

namespace test_sumcheck_round {
/**
 * @brief Place polynomials into full_polynomials in the order determined by the StandardArithmetization enum.
 *
 */
template <class FF, size_t N>
std::array<std::span<FF>, NUM_POLYNOMIALS> construct_full_polynomials(std::array<FF, N>& w_l,
                                                                      std::array<FF, N>& w_r,
                                                                      std::array<FF, N>& w_o,
                                                                      std::array<FF, N>& z_perm,
                                                                      std::array<FF, N>& z_perm_shift,
                                                                      std::array<FF, N>& q_m,
                                                                      std::array<FF, N>& q_l,
                                                                      std::array<FF, N>& q_r,
                                                                      std::array<FF, N>& q_o,
                                                                      std::array<FF, N>& q_c,
                                                                      std::array<FF, N>& sigma_1,
                                                                      std::array<FF, N>& sigma_2,
                                                                      std::array<FF, N>& sigma_3,
                                                                      std::array<FF, N>& id_1,
                                                                      std::array<FF, N>& id_2,
                                                                      std::array<FF, N>& id_3,
                                                                      std::array<FF, N>& lagrange_first,
                                                                      std::array<FF, N>& lagrange_last)
{
    std::array<std::span<FF>, NUM_POLYNOMIALS> full_polynomials;
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

    return full_polynomials;
}

// The below two methods are used in the test ComputeUnivariateProver
static Univariate<FF, max_relation_length> compute_round_univariate(
    std::array<std::array<FF, input_polynomial_length>, NUM_POLYNOMIALS>& input_polynomials,
    const RelationParameters<FF>& relation_parameters,
    const FF alpha)
{
    size_t round_size = 1;
    auto relations = std::tuple(
        ArithmeticRelation<FF>(), GrandProductComputationRelation<FF>(), GrandProductInitializationRelation<FF>());
    // Improvement(Cody): This is ugly? Maye supply some/all of this data through "flavor" class?
    auto round = SumcheckRound<FF,
                               NUM_POLYNOMIALS,
                               ArithmeticRelation,
                               GrandProductComputationRelation,
                               GrandProductInitializationRelation>(round_size, relations);
    auto w_l = input_polynomials[0];
    auto w_r = input_polynomials[1];
    auto w_o = input_polynomials[2];
    auto z_perm = input_polynomials[3];
    auto z_perm_shift = input_polynomials[4];
    auto q_m = input_polynomials[5];
    auto q_l = input_polynomials[6];
    auto q_r = input_polynomials[7];
    auto q_o = input_polynomials[8];
    auto q_c = input_polynomials[9];
    auto sigma_1 = input_polynomials[10];
    auto sigma_2 = input_polynomials[11];
    auto sigma_3 = input_polynomials[12];
    auto id_1 = input_polynomials[13];
    auto id_2 = input_polynomials[14];
    auto id_3 = input_polynomials[15];
    auto lagrange_first = input_polynomials[16];
    auto lagrange_last = input_polynomials[17];

    auto full_polynomials = construct_full_polynomials(w_l,
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
                                                       lagrange_first,
                                                       lagrange_last);
    PowUnivariate<FF> pow_zeta(1);
    Univariate<FF, max_relation_length> round_univariate =
        round.compute_univariate(full_polynomials, relation_parameters, pow_zeta, alpha);
    return round_univariate;
}

static Univariate<FF, max_relation_length> compute_expected_round_univariate(
    std::array<Univariate<FF, input_polynomial_length>, NUM_POLYNOMIALS>& input_univariates,
    const RelationParameters<FF>& relation_parameters,
    const FF alpha)
{
    BarycentricData<FF, input_polynomial_length, max_relation_length> barycentric_2_to_max =
        BarycentricData<FF, input_polynomial_length, max_relation_length>();
    std::array<Univariate<FF, max_relation_length>, NUM_POLYNOMIALS> extended_univariates;
    for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
        extended_univariates[i] = barycentric_2_to_max.extend(input_univariates[i]);
    }
    auto w_l_univariate = Univariate<FF, max_relation_length>(extended_univariates[0]);
    auto w_r_univariate = Univariate<FF, max_relation_length>(extended_univariates[1]);
    auto w_o_univariate = Univariate<FF, max_relation_length>(extended_univariates[2]);
    auto z_perm_univariate = Univariate<FF, max_relation_length>(extended_univariates[3]);
    auto z_perm_shift_univariate =
        Univariate<FF, max_relation_length>(extended_univariates[4]); // this is not real shifted data
    auto q_m_univariate = Univariate<FF, max_relation_length>(extended_univariates[5]);
    auto q_l_univariate = Univariate<FF, max_relation_length>(extended_univariates[6]);
    auto q_r_univariate = Univariate<FF, max_relation_length>(extended_univariates[7]);
    auto q_o_univariate = Univariate<FF, max_relation_length>(extended_univariates[8]);
    auto q_c_univariate = Univariate<FF, max_relation_length>(extended_univariates[9]);
    auto sigma_1_univariate = Univariate<FF, max_relation_length>(extended_univariates[10]);
    auto sigma_2_univariate = Univariate<FF, max_relation_length>(extended_univariates[11]);
    auto sigma_3_univariate = Univariate<FF, max_relation_length>(extended_univariates[12]);
    auto id_1_univariate = Univariate<FF, max_relation_length>(extended_univariates[13]);
    auto id_2_univariate = Univariate<FF, max_relation_length>(extended_univariates[14]);
    auto id_3_univariate = Univariate<FF, max_relation_length>(extended_univariates[15]);
    auto lagrange_first_univariate = Univariate<FF, max_relation_length>(extended_univariates[16]);
    auto lagrange_last_univariate = Univariate<FF, max_relation_length>(extended_univariates[17]);

    auto expected_arithmetic_relation =
        ((q_m_univariate * w_r_univariate * w_l_univariate) + (q_r_univariate * w_r_univariate) +
         (q_l_univariate * w_l_univariate) + (q_o_univariate * w_o_univariate) + (q_c_univariate));
    auto expected_grand_product_computation_relation =
        ((z_perm_univariate + lagrange_first_univariate) *
         (w_l_univariate + id_1_univariate * relation_parameters.beta + relation_parameters.gamma) *
         (w_r_univariate + id_2_univariate * relation_parameters.beta + relation_parameters.gamma) *
         (w_o_univariate + id_3_univariate * relation_parameters.beta + relation_parameters.gamma));
    expected_grand_product_computation_relation -=
        ((z_perm_shift_univariate + lagrange_last_univariate * relation_parameters.public_input_delta) *
         (w_l_univariate + sigma_1_univariate * relation_parameters.beta + relation_parameters.gamma) *
         (w_r_univariate + sigma_2_univariate * relation_parameters.beta + relation_parameters.gamma) *
         (w_o_univariate + sigma_3_univariate * relation_parameters.beta + relation_parameters.gamma));
    auto expected_grand_product_initialization_relation = (z_perm_shift_univariate * lagrange_last_univariate);
    Univariate<FF, max_relation_length> expected_round_univariate =
        expected_arithmetic_relation + expected_grand_product_computation_relation * alpha +
        expected_grand_product_initialization_relation * alpha.sqr();
    return expected_round_univariate;
}

// The below two methods are used in the test ComputeUnivariateVerifier
static FF compute_full_purported_value(std::array<FF, NUM_POLYNOMIALS>& input_values,
                                       const RelationParameters<FF>& relation_parameters,
                                       const FF alpha)
{
    std::vector<FF> purported_evaluations;
    purported_evaluations.resize(NUM_POLYNOMIALS);
    purported_evaluations[POLYNOMIAL::W_L] = input_values[0];
    purported_evaluations[POLYNOMIAL::W_R] = input_values[1];
    purported_evaluations[POLYNOMIAL::W_O] = input_values[2];
    purported_evaluations[POLYNOMIAL::Z_PERM] = input_values[3];
    purported_evaluations[POLYNOMIAL::Z_PERM_SHIFT] = input_values[4];
    purported_evaluations[POLYNOMIAL::Q_M] = input_values[5];
    purported_evaluations[POLYNOMIAL::Q_L] = input_values[6];
    purported_evaluations[POLYNOMIAL::Q_R] = input_values[7];
    purported_evaluations[POLYNOMIAL::Q_O] = input_values[8];
    purported_evaluations[POLYNOMIAL::Q_C] = input_values[9];
    purported_evaluations[POLYNOMIAL::SIGMA_1] = input_values[10];
    purported_evaluations[POLYNOMIAL::SIGMA_2] = input_values[11];
    purported_evaluations[POLYNOMIAL::SIGMA_3] = input_values[12];
    purported_evaluations[POLYNOMIAL::ID_1] = input_values[13];
    purported_evaluations[POLYNOMIAL::ID_2] = input_values[14];
    purported_evaluations[POLYNOMIAL::ID_3] = input_values[15];
    purported_evaluations[POLYNOMIAL::LAGRANGE_FIRST] = input_values[16];
    purported_evaluations[POLYNOMIAL::LAGRANGE_LAST] = input_values[17];
    auto relations = std::tuple(
        ArithmeticRelation<FF>(), GrandProductComputationRelation<FF>(), GrandProductInitializationRelation<FF>());
    auto round = SumcheckRound<FF,
                               NUM_POLYNOMIALS,
                               ArithmeticRelation,
                               GrandProductComputationRelation,
                               GrandProductInitializationRelation>(relations);
    PowUnivariate<FF> pow_univariate(1);
    FF full_purported_value = round.compute_full_honk_relation_purported_value(
        purported_evaluations, relation_parameters, pow_univariate, alpha);
    return full_purported_value;
}

static FF compute_full_purported_value_expected(std::array<FF, NUM_POLYNOMIALS>& input_values,
                                                const RelationParameters<FF>& relation_parameters,
                                                const FF alpha)
{
    FF w_l = input_values[0];
    FF w_r = input_values[1];
    FF w_o = input_values[2];
    FF z_perm = input_values[3];
    FF z_perm_shift = input_values[4];
    FF q_m = input_values[5];
    FF q_l = input_values[6];
    FF q_r = input_values[7];
    FF q_o = input_values[8];
    FF q_c = input_values[9];
    FF sigma_1 = input_values[10];
    FF sigma_2 = input_values[11];
    FF sigma_3 = input_values[12];
    FF id_1 = input_values[13];
    FF id_2 = input_values[14];
    FF id_3 = input_values[15];
    FF lagrange_first = input_values[16];
    FF lagrange_last = input_values[17];
    auto expected_arithmetic_relation = (q_m * w_r * w_l) + (q_r * w_r) + (q_l * w_l) + (q_o * w_o) + q_c;
    auto expected_grand_product_computation_relation =
        (z_perm + lagrange_first) * (w_l + id_1 * relation_parameters.beta + relation_parameters.gamma) *
        (w_r + id_2 * relation_parameters.beta + relation_parameters.gamma) *
        (w_o + id_3 * relation_parameters.beta + relation_parameters.gamma);
    expected_grand_product_computation_relation -=
        (z_perm_shift + lagrange_last * relation_parameters.public_input_delta) *
        (w_l + sigma_1 * relation_parameters.beta + relation_parameters.gamma) *
        (w_r + sigma_2 * relation_parameters.beta + relation_parameters.gamma) *
        (w_o + sigma_3 * relation_parameters.beta + relation_parameters.gamma);
    auto expected_grand_product_initialization_relation = z_perm_shift * lagrange_last;
    auto expected_full_purported_value = expected_arithmetic_relation +
                                         expected_grand_product_computation_relation * alpha +
                                         expected_grand_product_initialization_relation * alpha.sqr();
    return expected_full_purported_value;
}

TEST(SumcheckRound, ComputeUnivariateProver)
{
    auto run_test = [](bool is_random_input) {
        if (is_random_input) {
            std::array<std::array<FF, input_polynomial_length>, NUM_POLYNOMIALS> input_polynomials;
            for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
                input_polynomials[i] = { FF::random_element(), FF::random_element() };
            }

            const FF alpha = FF::random_element();
            const RelationParameters<FF> relation_parameters = RelationParameters<FF>{
                .beta = FF::random_element(), .gamma = FF::random_element(), .public_input_delta = FF::random_element()
            };

            auto round_univariate = compute_round_univariate(input_polynomials, relation_parameters, alpha);

            // Compute round_univariate manually
            std::array<Univariate<FF, input_polynomial_length>, NUM_POLYNOMIALS> input_univariates;
            for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
                input_univariates[i] = Univariate<FF, input_polynomial_length>(input_polynomials[i]);
            }
            auto expected_round_univariate =
                compute_expected_round_univariate(input_univariates, relation_parameters, alpha);
            EXPECT_EQ(round_univariate, expected_round_univariate);
        } else {
            std::array<std::array<FF, input_polynomial_length>, NUM_POLYNOMIALS> input_polynomials;
            for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
                input_polynomials[i] = { 1, 2 };
            }
            const FF alpha = 1;
            const RelationParameters<FF> relation_parameters =
                RelationParameters<FF>{ .beta = 1, .gamma = 1, .public_input_delta = 1 };
            auto round_univariate = compute_round_univariate(input_polynomials, relation_parameters, alpha);
            // Compute round_univariate manually
            std::array<Univariate<FF, input_polynomial_length>, NUM_POLYNOMIALS> input_univariates;
            for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
                input_univariates[i] = Univariate<FF, input_polynomial_length>(input_polynomials[i]);
            }
            // expected_round_univariate = { 6, 26, 66, 132, 230, 366 }
            auto expected_round_univariate =
                compute_expected_round_univariate(input_univariates, relation_parameters, alpha);
            EXPECT_EQ(round_univariate, expected_round_univariate);
        };
    };
    run_test(/* is_random_input=*/false);
    run_test(/* is_random_input=*/true);
}

TEST(SumcheckRound, ComputeUnivariateVerifier)
{
    auto run_test = [](bool is_random_input) {
        if (is_random_input) {
            std::array<FF, NUM_POLYNOMIALS> input_values;
            for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
                input_values[i] = FF::random_element();
            }
            const FF alpha = FF::random_element();
            const RelationParameters<FF> relation_parameters = RelationParameters<FF>{
                .beta = FF::random_element(), .gamma = FF::random_element(), .public_input_delta = FF::random_element()
            };
            auto full_purported_value = compute_full_purported_value(input_values, relation_parameters, alpha);
            // Compute round_univariate manually
            auto expected_full_purported_value =
                compute_full_purported_value_expected(input_values, relation_parameters, alpha);
            EXPECT_EQ(full_purported_value, expected_full_purported_value);
        } else {
            std::array<FF, NUM_POLYNOMIALS> input_values;
            for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
                input_values[i] = FF(2);
            }
            const FF alpha = 1;
            const RelationParameters<FF> relation_parameters =
                RelationParameters<FF>{ .beta = 1, .gamma = 1, .public_input_delta = 1 };
            auto full_purported_value = compute_full_purported_value(input_values, relation_parameters, alpha);
            // Compute round_univariate manually
            auto expected_full_purported_value =
                compute_full_purported_value_expected(input_values, relation_parameters, alpha);
            EXPECT_EQ(full_purported_value, expected_full_purported_value);
        };
    };
    run_test(/* is_random_input=*/false);
    run_test(/* is_random_input=*/true);
}

} // namespace test_sumcheck_round
