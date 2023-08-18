#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/honk/composer/standard_composer.hpp"
#include "barretenberg/honk/composer/ultra_composer.hpp"
#include "barretenberg/honk/flavor/standard.hpp"
#include "barretenberg/honk/proof_system/grand_product_library.hpp"
#include "barretenberg/honk/sumcheck/relations/auxiliary_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/elliptic_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/gen_perm_sort_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/lookup_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/ultra_arithmetic_relation.hpp"
#include "barretenberg/honk/transcript/transcript.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include "barretenberg/transcript/manifest.hpp"
#include "barretenberg/transcript/transcript_wrappers.hpp"
#include "relations/arithmetic_relation.hpp"
#include "relations/permutation_relation.hpp"
#include "sumcheck.hpp"
#include <array>
#include <cstddef>
#include <cstdint>
#include <gtest/internal/gtest-internal.h>

#include <gtest/gtest.h>
#include <initializer_list>
#include <optional>
#include <string>
#include <sys/types.h>
#include <vector>

using namespace proof_system::honk;
using namespace proof_system::honk::sumcheck;
using Flavor = honk::flavor::Standard; // TODO(Cody): Generalize this test.
using FF = typename Flavor::FF;
using ProverPolynomials = typename Flavor::ProverPolynomials;
const size_t NUM_POLYNOMIALS = Flavor::NUM_ALL_ENTITIES;

namespace test_sumcheck_round {

template <class FF, size_t N>
ProverPolynomials construct_full_polynomials(std::array<FF, N>& w_l,
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
    ProverPolynomials full_polynomials;
    full_polynomials.w_l = w_l;
    full_polynomials.w_r = w_r;
    full_polynomials.w_o = w_o;
    full_polynomials.z_perm = z_perm;
    full_polynomials.z_perm_shift = z_perm_shift;
    full_polynomials.q_m = q_m;
    full_polynomials.q_l = q_l;
    full_polynomials.q_r = q_r;
    full_polynomials.q_o = q_o;
    full_polynomials.q_c = q_c;
    full_polynomials.sigma_1 = sigma_1;
    full_polynomials.sigma_2 = sigma_2;
    full_polynomials.sigma_3 = sigma_3;
    full_polynomials.id_1 = id_1;
    full_polynomials.id_2 = id_2;
    full_polynomials.id_3 = id_3;
    full_polynomials.lagrange_first = lagrange_first;
    full_polynomials.lagrange_last = lagrange_last;

    return full_polynomials;
}

class SumcheckTests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { barretenberg::srs::init_crs_factory("../srs_db/ignition"); }
};

TEST_F(SumcheckTests, PolynomialNormalization)
{
    // TODO(#225)(Cody): We should not use real constants like this in the tests, at least not in so many of them.
    const size_t multivariate_d(3);
    const size_t multivariate_n(1 << multivariate_d);

    std::array<FF, multivariate_n> w_l;
    std::array<FF, multivariate_n> w_r;
    std::array<FF, multivariate_n> w_o;
    std::array<FF, multivariate_n> z_perm;
    std::array<FF, multivariate_n> z_perm_shift;
    std::array<FF, multivariate_n> q_m;
    std::array<FF, multivariate_n> q_l;
    std::array<FF, multivariate_n> q_r;
    std::array<FF, multivariate_n> q_o;
    std::array<FF, multivariate_n> q_c;
    std::array<FF, multivariate_n> sigma_1;
    std::array<FF, multivariate_n> sigma_2;
    std::array<FF, multivariate_n> sigma_3;
    std::array<FF, multivariate_n> id_1;
    std::array<FF, multivariate_n> id_2;
    std::array<FF, multivariate_n> id_3;
    std::array<FF, multivariate_n> lagrange_first;
    std::array<FF, multivariate_n> lagrange_last;
    for (size_t i = 0; i < multivariate_n; i++) {
        w_l[i] = FF::random_element();
        w_r[i] = FF::random_element();
        w_o[i] = FF::random_element();
        z_perm[i] = FF::random_element();
        z_perm_shift[i] = FF::random_element();
        q_m[i] = FF::random_element();
        q_l[i] = FF::random_element();
        q_r[i] = FF::random_element();
        q_o[i] = FF::random_element();
        q_c[i] = FF::random_element();
        sigma_1[i] = FF::random_element();
        sigma_2[i] = FF::random_element();
        sigma_3[i] = FF::random_element();
        id_1[i] = FF::random_element();
        id_2[i] = FF::random_element();
        id_3[i] = FF::random_element();
        lagrange_first[i] = FF::random_element();
        lagrange_last[i] = FF::random_element();
    }

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

    auto transcript = ProverTranscript<FF>::init_empty();

    auto sumcheck = SumcheckProver<Flavor>(multivariate_n, transcript);

    auto [multivariate_challenge, evaluations] = sumcheck.prove(full_polynomials, {});

    FF u_0 = multivariate_challenge[0];
    FF u_1 = multivariate_challenge[1];
    FF u_2 = multivariate_challenge[2];

    /* sumcheck.prove() terminates with sumcheck.multivariates.folded_polynoimals as an array such that
     * sumcheck.multivariates.folded_polynoimals[i][0] is the evaluatioin of the i'th multivariate at the vector of
     challenges u_i. What does this mean?

     Here we show that if the multivariate is F(X0, X1, X2) defined as above, then what we get is F(u0, u1, u2) and
     not, say F(u2, u1, u0). This is in accordance with Adrian's thesis (cf page 9).
      */

    // Get the values of the Lagrange basis polys L_i defined
    // by: L_i(v) = 1 if i = v, 0 otherwise, for v from 0 to 7.
    FF one{ 1 };
    // clang-format off
    FF l_0 = (one - u_0) * (one - u_1) * (one - u_2);
    FF l_1 = (      u_0) * (one - u_1) * (one - u_2);
    FF l_2 = (one - u_0) * (      u_1) * (one - u_2);
    FF l_3 = (      u_0) * (      u_1) * (one - u_2);
    FF l_4 = (one - u_0) * (one - u_1) * (      u_2);
    FF l_5 = (      u_0) * (one - u_1) * (      u_2);
    FF l_6 = (one - u_0) * (      u_1) * (      u_2);
    FF l_7 = (      u_0) * (      u_1) * (      u_2);
    // clang-format on
    FF hand_computed_value;
    for (size_t i = 0; i < NUM_POLYNOMIALS; i++) {
        // full_polynomials[0][0] = w_l[0], full_polynomials[1][1] = w_r[1], and so on.
        hand_computed_value = l_0 * full_polynomials[i][0] + l_1 * full_polynomials[i][1] +
                              l_2 * full_polynomials[i][2] + l_3 * full_polynomials[i][3] +
                              l_4 * full_polynomials[i][4] + l_5 * full_polynomials[i][5] +
                              l_6 * full_polynomials[i][6] + l_7 * full_polynomials[i][7];
        EXPECT_EQ(hand_computed_value, sumcheck.partially_evaluated_polynomials[i][0]);
    }
}

TEST_F(SumcheckTests, Prover)
{
    auto run_test = [](bool is_random_input) {
        const size_t multivariate_d(2);
        const size_t multivariate_n(1 << multivariate_d);
        std::array<std::array<FF, multivariate_n>, NUM_POLYNOMIALS> input_polynomials;
        if (is_random_input) {
            for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
                input_polynomials[i] = {
                    FF::random_element(), FF::random_element(), FF::random_element(), FF::random_element()
                };
            }
        } else {
            for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
                input_polynomials[i] = { 1, 2, 0, 0 };
            }
        };
        std::array<FF, multivariate_n> w_l = input_polynomials[0];
        std::array<FF, multivariate_n> w_r = input_polynomials[1];
        std::array<FF, multivariate_n> w_o = input_polynomials[2];
        std::array<FF, multivariate_n> z_perm = input_polynomials[3];
        std::array<FF, multivariate_n> z_perm_shift = input_polynomials[4];
        std::array<FF, multivariate_n> q_m = input_polynomials[5];
        std::array<FF, multivariate_n> q_l = input_polynomials[6];
        std::array<FF, multivariate_n> q_r = input_polynomials[7];
        std::array<FF, multivariate_n> q_o = input_polynomials[8];
        std::array<FF, multivariate_n> q_c = input_polynomials[9];
        std::array<FF, multivariate_n> sigma_1 = input_polynomials[10];
        std::array<FF, multivariate_n> sigma_2 = input_polynomials[11];
        std::array<FF, multivariate_n> sigma_3 = input_polynomials[12];
        std::array<FF, multivariate_n> id_1 = input_polynomials[13];
        std::array<FF, multivariate_n> id_2 = input_polynomials[14];
        std::array<FF, multivariate_n> id_3 = input_polynomials[15];
        std::array<FF, multivariate_n> lagrange_first = input_polynomials[16];
        std::array<FF, multivariate_n> lagrange_last = input_polynomials[17];
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

        auto transcript = ProverTranscript<FF>::init_empty();

        auto sumcheck = SumcheckProver<Flavor>(multivariate_n, transcript);

        auto [multivariate_challenge, evaluations] = sumcheck.prove(full_polynomials, {});
        FF u_0 = multivariate_challenge[0];
        FF u_1 = multivariate_challenge[1];
        std::vector<FF> expected_values;
        for (auto& polynomial : full_polynomials) {
            // using knowledge of inputs here to derive the evaluation
            FF expected_lo = polynomial[0] * (FF(1) - u_0) + polynomial[1] * u_0;
            expected_lo *= (FF(1) - u_1);
            FF expected_hi = polynomial[2] * (FF(1) - u_0) + polynomial[3] * u_0;
            expected_hi *= u_1;
            expected_values.emplace_back(expected_lo + expected_hi);
        }

        for (size_t poly_idx = 0; poly_idx < NUM_POLYNOMIALS; poly_idx++) {
            EXPECT_EQ(evaluations[poly_idx], expected_values[poly_idx]);
        }
    };
    run_test(/* is_random_input=*/false);
    run_test(/* is_random_input=*/true);
}

// TODO(#223)(Cody): write standalone test of the verifier.
// Note(luke): This test (and ProverAndVerifierLonger) are slighly misleading in that they include the grand product
// realtions but do not test their correctness due to the choice of zero polynomials for sigma, id etc.
TEST_F(SumcheckTests, ProverAndVerifier)
{
    const size_t multivariate_d(1);
    const size_t multivariate_n(1 << multivariate_d);

    std::array<FF, 2> w_l = { 0, 1 };
    std::array<FF, 2> w_r = { 0, 1 };
    std::array<FF, 2> w_o = { 0, 2 };
    std::array<FF, 2> z_perm = { 0, 0 };
    std::array<FF, 2> z_perm_shift = { 0, 0 }; // NOTE: Not set up to be valid.
    std::array<FF, 2> q_m = { 0, 0 };
    std::array<FF, 2> q_l = { 1, 1 };
    std::array<FF, 2> q_r = { 0, 1 };
    std::array<FF, 2> q_o = { 0, -1 };
    std::array<FF, 2> q_c = { 0, 0 };
    std::array<FF, 2> sigma_1 = { 0, 0 }; // NOTE: Not set up to be valid.
    std::array<FF, 2> sigma_2 = { 0, 0 }; // NOTE: Not set up to be valid.
    std::array<FF, 2> sigma_3 = { 0, 0 }; // NOTE: Not set up to be valid.
    std::array<FF, 2> id_1 = { 0, 0 };    // NOTE: Not set up to be valid.
    std::array<FF, 2> id_2 = { 0, 0 };    // NOTE: Not set up to be valid.
    std::array<FF, 2> id_3 = { 0, 0 };    // NOTE: Not set up to be valid.
    std::array<FF, 2> lagrange_first = { 0, 0 };
    std::array<FF, 2> lagrange_last = { 0, 0 }; // NOTE: Not set up to be valid.

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
    // Set aribitrary random relation parameters
    sumcheck::RelationParameters<FF> relation_parameters{
        .beta = FF::random_element(),
        .gamma = FF::random_element(),
        .public_input_delta = FF::one(),
    };

    auto prover_transcript = ProverTranscript<FF>::init_empty();

    auto sumcheck_prover = SumcheckProver<Flavor>(multivariate_n, prover_transcript);

    auto prover_output = sumcheck_prover.prove(full_polynomials, relation_parameters);

    auto verifier_transcript = VerifierTranscript<FF>::init_empty(prover_transcript);

    auto sumcheck_verifier = SumcheckVerifier<Flavor>(multivariate_n);

    std::optional verifier_output = sumcheck_verifier.verify(relation_parameters, verifier_transcript);

    ASSERT_TRUE(verifier_output.has_value());
    ASSERT_EQ(prover_output, *verifier_output);
}

// TODO(#225): make the inputs to this test more interesting, e.g. num_public_inputs > 0 and non-trivial permutations
TEST_F(SumcheckTests, ProverAndVerifierLonger)
{
    auto run_test = [](bool expect_verified) {
        const size_t multivariate_d(2);
        const size_t multivariate_n(1 << multivariate_d);

        // clang-format off
    std::array<FF, multivariate_n> w_l;
    if (expect_verified) {         w_l =            { 0,  1,  2, 0 };
    } else {                       w_l =            { 0,  0,  2, 0 };
    }
    std::array<FF, multivariate_n> w_r            = { 0,  1,  2, 0 };
    std::array<FF, multivariate_n> w_o            = { 0,  2,  4, 0 };
    std::array<FF, multivariate_n> z_perm         = { 0,  0,  0, 0 };
    std::array<FF, multivariate_n> z_perm_shift   = { 0,  0,  0, 0 };
    std::array<FF, multivariate_n> q_m            = { 0,  0,  1, 0 };
    std::array<FF, multivariate_n> q_l            = { 1,  1,  0, 0 };
    std::array<FF, multivariate_n> q_r            = { 0,  1,  0, 0 };
    std::array<FF, multivariate_n> q_o            = { 0, -1,  -1, 0 };
    std::array<FF, multivariate_n> q_c            = { 0,  0,  0, 0 };
    // Setting all of these to 0 ensures the GrandProductRelation is satisfied
    std::array<FF, multivariate_n> sigma_1        = { 0,  0,  0, 0 };
    std::array<FF, multivariate_n> sigma_2        = { 0,  0,  0, 0 };
    std::array<FF, multivariate_n> sigma_3        = { 0,  0,  0, 0 };
    std::array<FF, multivariate_n> id_1           = { 0,  0,  0, 0 };
    std::array<FF, multivariate_n> id_2           = { 0,  0,  0, 0 };
    std::array<FF, multivariate_n> id_3           = { 0,  0,  0, 0 };
    std::array<FF, multivariate_n> lagrange_first = { 0,  0,  0, 0 };
    std::array<FF, multivariate_n> lagrange_last  = { 0,  0,  0, 0 };
        // clang-format on

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

        // Set aribitrary random relation parameters
        sumcheck::RelationParameters<FF> relation_parameters{
            .beta = FF::random_element(),
            .gamma = FF::random_element(),
            .public_input_delta = FF::one(),
        };

        auto prover_transcript = ProverTranscript<FF>::init_empty();

        auto sumcheck_prover = SumcheckProver<Flavor>(multivariate_n, prover_transcript);

        auto prover_output = sumcheck_prover.prove(full_polynomials, relation_parameters);

        auto verifier_transcript = VerifierTranscript<FF>::init_empty(prover_transcript);

        auto sumcheck_verifier = SumcheckVerifier<Flavor>(multivariate_n);

        std::optional verifier_output = sumcheck_verifier.verify(relation_parameters, verifier_transcript);

        EXPECT_EQ(verifier_output.has_value(), expect_verified);
    };

    run_test(/* expect_verified=*/true);
    run_test(/* expect_verified=*/false);
}

/**
 * @brief Test the Standard Sumcheck Prover and Verifier for a real circuit
 *
 */
TEST_F(SumcheckTests, RealCircuitStandard)
{
    using Flavor = honk::flavor::Standard;
    using FF = typename Flavor::FF;
    using ProverPolynomials = typename Flavor::ProverPolynomials;

    // Create a composer and a dummy circuit with a few gates
    auto circuit_constructor = StandardCircuitBuilder();
    fr a = fr::one();
    // Using the public variable to check that public_input_delta is computed and added to the relation correctly
    uint32_t a_idx = circuit_constructor.add_public_variable(a);
    fr b = fr::one();
    fr c = a + b;
    fr d = a + c;
    uint32_t b_idx = circuit_constructor.add_variable(b);
    uint32_t c_idx = circuit_constructor.add_variable(c);
    uint32_t d_idx = circuit_constructor.add_variable(d);
    for (size_t i = 0; i < 16; i++) {
        circuit_constructor.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
        circuit_constructor.create_add_gate(
            { d_idx, c_idx, a_idx, fr::one(), fr::neg_one(), fr::neg_one(), fr::zero() });
    }
    // Create a prover (it will compute proving key and witness)
    auto composer = StandardComposer();
    auto prover = composer.create_prover(circuit_constructor);

    // Generate beta and gamma
    fr beta = fr::random_element();
    fr gamma = fr::random_element();

    // Compute public input delta
    const auto public_inputs = circuit_constructor.get_public_inputs();
    auto public_input_delta =
        honk::compute_public_input_delta<Flavor>(public_inputs, beta, gamma, prover.key->circuit_size);

    sumcheck::RelationParameters<FF> relation_parameters{
        .beta = beta,
        .gamma = gamma,
        .public_input_delta = public_input_delta,
    };

    ProverPolynomials prover_polynomials;

    prover_polynomials.w_l = prover.key->w_l;
    prover_polynomials.w_r = prover.key->w_r;
    prover_polynomials.w_o = prover.key->w_o;
    prover_polynomials.q_m = prover.key->q_m;
    prover_polynomials.q_l = prover.key->q_l;
    prover_polynomials.q_r = prover.key->q_r;
    prover_polynomials.q_o = prover.key->q_o;
    prover_polynomials.q_c = prover.key->q_c;
    prover_polynomials.sigma_1 = prover.key->sigma_1;
    prover_polynomials.sigma_2 = prover.key->sigma_2;
    prover_polynomials.sigma_3 = prover.key->sigma_3;
    prover_polynomials.id_1 = prover.key->id_1;
    prover_polynomials.id_2 = prover.key->id_2;
    prover_polynomials.id_3 = prover.key->id_3;
    prover_polynomials.lagrange_first = prover.key->lagrange_first;
    prover_polynomials.lagrange_last = prover.key->lagrange_last;

    // Compute grand product polynomial
    grand_product_library::compute_grand_products<Flavor>(prover.key, prover_polynomials, relation_parameters);

    auto prover_transcript = ProverTranscript<FF>::init_empty();

    auto sumcheck_prover = SumcheckProver<Flavor>(prover.key->circuit_size, prover_transcript);

    auto prover_output = sumcheck_prover.prove(prover_polynomials, relation_parameters);

    auto verifier_transcript = VerifierTranscript<FF>::init_empty(prover_transcript);

    auto sumcheck_verifier = SumcheckVerifier<Flavor>(prover.key->circuit_size);

    std::optional verifier_output = sumcheck_verifier.verify(relation_parameters, verifier_transcript);

    ASSERT_TRUE(verifier_output.has_value());
}

/**
 * @brief Test the Ultra Sumcheck Prover and Verifier for a real circuit
 *
 */
TEST_F(SumcheckTests, RealCircuitUltra)
{
    using Flavor = honk::flavor::Ultra;
    using FF = typename Flavor::FF;
    using ProverPolynomials = typename Flavor::ProverPolynomials;

    // Create a composer and a dummy circuit with a few gates
    auto circuit_constructor = UltraCircuitBuilder();
    fr a = fr::one();

    // Add some basic add gates, with a public input for good measure
    uint32_t a_idx = circuit_constructor.add_public_variable(a);
    fr b = fr::one();
    fr c = a + b;
    fr d = a + c;
    uint32_t b_idx = circuit_constructor.add_variable(b);
    uint32_t c_idx = circuit_constructor.add_variable(c);
    uint32_t d_idx = circuit_constructor.add_variable(d);
    for (size_t i = 0; i < 16; i++) {
        circuit_constructor.create_add_gate({ a_idx, b_idx, c_idx, 1, 1, -1, 0 });
        circuit_constructor.create_add_gate({ d_idx, c_idx, a_idx, 1, -1, -1, 0 });
    }

    // Add a big add gate with use of next row to test q_arith = 2
    fr e = a + b + c + d;
    uint32_t e_idx = circuit_constructor.add_variable(e);

    uint32_t zero_idx = circuit_constructor.zero_idx;
    circuit_constructor.create_big_add_gate({ a_idx, b_idx, c_idx, d_idx, -1, -1, -1, -1, 0 }, true); // use next row
    circuit_constructor.create_big_add_gate({ zero_idx, zero_idx, zero_idx, e_idx, 0, 0, 0, 0, 0 }, false);

    // Add some lookup gates (related to pedersen hashing)
    barretenberg::fr pedersen_input_value = fr::random_element();
    const fr input_hi = uint256_t(pedersen_input_value).slice(126, 256);
    const fr input_lo = uint256_t(pedersen_input_value).slice(0, 126);
    const auto input_hi_index = circuit_constructor.add_variable(input_hi);
    const auto input_lo_index = circuit_constructor.add_variable(input_lo);

    const auto sequence_data_hi = plookup::get_lookup_accumulators(plookup::MultiTableId::PEDERSEN_LEFT_HI, input_hi);
    const auto sequence_data_lo = plookup::get_lookup_accumulators(plookup::MultiTableId::PEDERSEN_LEFT_LO, input_lo);

    circuit_constructor.create_gates_from_plookup_accumulators(
        plookup::MultiTableId::PEDERSEN_LEFT_HI, sequence_data_hi, input_hi_index);
    circuit_constructor.create_gates_from_plookup_accumulators(
        plookup::MultiTableId::PEDERSEN_LEFT_LO, sequence_data_lo, input_lo_index);

    // Add a sort gate (simply checks that consecutive inputs have a difference of < 4)
    a_idx = circuit_constructor.add_variable(FF(0));
    b_idx = circuit_constructor.add_variable(FF(1));
    c_idx = circuit_constructor.add_variable(FF(2));
    d_idx = circuit_constructor.add_variable(FF(3));
    circuit_constructor.create_sort_constraint({ a_idx, b_idx, c_idx, d_idx });

    // Add an elliptic curve addition gate
    grumpkin::g1::affine_element p1 = crypto::generators::get_generator_data({ 0, 0 }).generator;
    grumpkin::g1::affine_element p2 = crypto::generators::get_generator_data({ 0, 1 }).generator;

    grumpkin::fq beta_scalar = grumpkin::fq::cube_root_of_unity();
    grumpkin::g1::affine_element p2_endo = p2;
    p2_endo.x *= beta_scalar;

    grumpkin::g1::affine_element p3(grumpkin::g1::element(p1) - grumpkin::g1::element(p2_endo));

    uint32_t x1 = circuit_constructor.add_variable(p1.x);
    uint32_t y1 = circuit_constructor.add_variable(p1.y);
    uint32_t x2 = circuit_constructor.add_variable(p2.x);
    uint32_t y2 = circuit_constructor.add_variable(p2.y);
    uint32_t x3 = circuit_constructor.add_variable(p3.x);
    uint32_t y3 = circuit_constructor.add_variable(p3.y);

    circuit_constructor.create_ecc_add_gate({ x1, y1, x2, y2, x3, y3, beta_scalar, -1 });

    // Add some RAM gates
    uint32_t ram_values[8]{
        circuit_constructor.add_variable(fr::random_element()), circuit_constructor.add_variable(fr::random_element()),
        circuit_constructor.add_variable(fr::random_element()), circuit_constructor.add_variable(fr::random_element()),
        circuit_constructor.add_variable(fr::random_element()), circuit_constructor.add_variable(fr::random_element()),
        circuit_constructor.add_variable(fr::random_element()), circuit_constructor.add_variable(fr::random_element()),
    };

    size_t ram_id = circuit_constructor.create_RAM_array(8);

    for (size_t i = 0; i < 8; ++i) {
        circuit_constructor.init_RAM_element(ram_id, i, ram_values[i]);
    }

    a_idx = circuit_constructor.read_RAM_array(ram_id, circuit_constructor.add_variable(5));
    EXPECT_EQ(a_idx != ram_values[5], true);

    b_idx = circuit_constructor.read_RAM_array(ram_id, circuit_constructor.add_variable(4));
    c_idx = circuit_constructor.read_RAM_array(ram_id, circuit_constructor.add_variable(1));

    circuit_constructor.write_RAM_array(
        ram_id, circuit_constructor.add_variable(4), circuit_constructor.add_variable(500));
    d_idx = circuit_constructor.read_RAM_array(ram_id, circuit_constructor.add_variable(4));

    EXPECT_EQ(circuit_constructor.get_variable(d_idx), 500);

    // ensure these vars get used in another arithmetic gate
    const auto e_value = circuit_constructor.get_variable(a_idx) + circuit_constructor.get_variable(b_idx) +
                         circuit_constructor.get_variable(c_idx) + circuit_constructor.get_variable(d_idx);
    e_idx = circuit_constructor.add_variable(e_value);

    circuit_constructor.create_big_add_gate({ a_idx, b_idx, c_idx, d_idx, -1, -1, -1, -1, 0 }, true);
    circuit_constructor.create_big_add_gate(
        {
            circuit_constructor.zero_idx,
            circuit_constructor.zero_idx,
            circuit_constructor.zero_idx,
            e_idx,
            0,
            0,
            0,
            0,
            0,
        },
        false);

    // Create a prover (it will compute proving key and witness)
    auto composer = UltraComposer();
    auto prover = composer.create_prover(circuit_constructor);

    // Generate eta, beta and gamma
    fr eta = fr::random_element();
    fr beta = fr::random_element();
    fr gamma = fr::random_element();

    // Compute public input delta
    const auto public_inputs = circuit_constructor.get_public_inputs();
    size_t pub_inputs_offset = Flavor::has_zero_row ? 1 : 0;
    auto public_input_delta = honk::compute_public_input_delta<Flavor>(
        public_inputs, beta, gamma, prover.key->circuit_size, pub_inputs_offset);
    auto lookup_grand_product_delta =
        honk::compute_lookup_grand_product_delta<FF>(beta, gamma, prover.key->circuit_size);

    sumcheck::RelationParameters<FF> relation_parameters{
        .eta = eta,
        .beta = beta,
        .gamma = gamma,
        .public_input_delta = public_input_delta,
        .lookup_grand_product_delta = lookup_grand_product_delta,
    };

    // Compute sorted witness-table accumulator
    prover.key->sorted_accum = prover_library::compute_sorted_list_accumulator<Flavor>(prover.key, eta);

    // Add RAM/ROM memory records to wire four
    prover_library::add_plookup_memory_records_to_wire_4<Flavor>(prover.key, eta);

    ProverPolynomials prover_polynomials;

    prover_polynomials.w_l = prover.key->w_l;
    prover_polynomials.w_r = prover.key->w_r;
    prover_polynomials.w_o = prover.key->w_o;
    prover_polynomials.w_4 = prover.key->w_4;
    prover_polynomials.w_l_shift = prover.key->w_l.shifted();
    prover_polynomials.w_r_shift = prover.key->w_r.shifted();
    prover_polynomials.w_o_shift = prover.key->w_o.shifted();
    prover_polynomials.w_4_shift = prover.key->w_4.shifted();
    prover_polynomials.sorted_accum = prover.key->sorted_accum;
    prover_polynomials.sorted_accum_shift = prover.key->sorted_accum.shifted();
    prover_polynomials.table_1 = prover.key->table_1;
    prover_polynomials.table_2 = prover.key->table_2;
    prover_polynomials.table_3 = prover.key->table_3;
    prover_polynomials.table_4 = prover.key->table_4;
    prover_polynomials.table_1_shift = prover.key->table_1.shifted();
    prover_polynomials.table_2_shift = prover.key->table_2.shifted();
    prover_polynomials.table_3_shift = prover.key->table_3.shifted();
    prover_polynomials.table_4_shift = prover.key->table_4.shifted();
    prover_polynomials.z_lookup = prover.key->z_lookup;
    prover_polynomials.z_lookup_shift = prover.key->z_lookup.shifted();
    prover_polynomials.q_m = prover.key->q_m;
    prover_polynomials.q_l = prover.key->q_l;
    prover_polynomials.q_r = prover.key->q_r;
    prover_polynomials.q_o = prover.key->q_o;
    prover_polynomials.q_c = prover.key->q_c;
    prover_polynomials.q_4 = prover.key->q_4;
    prover_polynomials.q_arith = prover.key->q_arith;
    prover_polynomials.q_sort = prover.key->q_sort;
    prover_polynomials.q_elliptic = prover.key->q_elliptic;
    prover_polynomials.q_aux = prover.key->q_aux;
    prover_polynomials.q_lookup = prover.key->q_lookup;
    prover_polynomials.sigma_1 = prover.key->sigma_1;
    prover_polynomials.sigma_2 = prover.key->sigma_2;
    prover_polynomials.sigma_3 = prover.key->sigma_3;
    prover_polynomials.sigma_4 = prover.key->sigma_4;
    prover_polynomials.id_1 = prover.key->id_1;
    prover_polynomials.id_2 = prover.key->id_2;
    prover_polynomials.id_3 = prover.key->id_3;
    prover_polynomials.id_4 = prover.key->id_4;
    prover_polynomials.lagrange_first = prover.key->lagrange_first;
    prover_polynomials.lagrange_last = prover.key->lagrange_last;

    grand_product_library::compute_grand_products<Flavor>(prover.key, prover_polynomials, relation_parameters);

    auto prover_transcript = ProverTranscript<FF>::init_empty();

    auto sumcheck_prover = SumcheckProver<Flavor>(prover.key->circuit_size, prover_transcript);

    auto prover_output = sumcheck_prover.prove(prover_polynomials, relation_parameters);

    auto verifier_transcript = VerifierTranscript<FF>::init_empty(prover_transcript);

    auto sumcheck_verifier = SumcheckVerifier<Flavor>(prover.key->circuit_size);

    std::optional verifier_output = sumcheck_verifier.verify(relation_parameters, verifier_transcript);

    ASSERT_TRUE(verifier_output.has_value());
}

} // namespace test_sumcheck_round
