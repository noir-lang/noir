#include "sumcheck.hpp"
#include "barretenberg/proof_system/flavor/flavor.hpp"
#include "barretenberg/transcript/transcript_wrappers.hpp"
#include "relations/arithmetic_relation.hpp"
#include "relations/grand_product_computation_relation.hpp"
#include "relations/grand_product_initialization_relation.hpp"
#include "barretenberg/transcript/manifest.hpp"
#include <array>
#include <cstddef>
#include <cstdint>
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include <gtest/internal/gtest-internal.h>
#include "barretenberg/numeric/random/engine.hpp"

#include <initializer_list>
#include <gtest/gtest.h>
#include <string>
#include <sys/types.h>
#include <vector>

using namespace honk;
using namespace honk::sumcheck;
using Transcript = transcript::StandardTranscript;
using FF = barretenberg::fr;
const size_t NUM_POLYNOMIALS = bonk::StandardArithmetization::NUM_POLYNOMIALS;
using POLYNOMIAL = bonk::StandardArithmetization::POLYNOMIAL;

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

Transcript produce_mocked_transcript(size_t multivariate_d, size_t num_public_inputs)
{
    // Create a mock manifest containing only elements needed for testing Sumcheck
    constexpr size_t fr_size = 32;
    const size_t multivariate_n(1 << multivariate_d);
    const size_t public_input_size = fr_size * num_public_inputs;
    std::vector<transcript::Manifest::RoundManifest> manifest_rounds;
    manifest_rounds.emplace_back(transcript::Manifest::RoundManifest(
        { { .name = "circuit_size", .num_bytes = 4, .derived_by_verifier = true },
          { .name = "public_input_size", .num_bytes = 4, .derived_by_verifier = true } },
        /* challenge_name = */ "init",
        /* num_challenges_in = */ 1));

    manifest_rounds.emplace_back(transcript::Manifest::RoundManifest({ /* this is a noop */ },
                                                                     /* challenge_name = */ "alpha",
                                                                     /* num_challenges_in = */ 2));
    manifest_rounds.emplace_back(transcript::Manifest::RoundManifest(
        { { .name = "public_inputs", .num_bytes = public_input_size, .derived_by_verifier = false } },
        /* challenge_name = */ "beta",
        /* num_challenges_in = */ 2) // also produce "gamma"
    );

    for (size_t i = 0; i < multivariate_d; i++) {
        auto label = std::to_string(i);
        manifest_rounds.emplace_back(
            transcript::Manifest::RoundManifest({ { .name = "univariate_" + label,
                                                    .num_bytes = fr_size * honk::StandardHonk::MAX_RELATION_LENGTH,
                                                    .derived_by_verifier = false } },
                                                /* challenge_name = */ "u_" + label,
                                                /* num_challenges_in = */ 1));
    }

    // Create a transcript from the mock manifest
    auto transcript = Transcript(transcript::Manifest(manifest_rounds));

    transcript.add_element("circuit_size",
                           { static_cast<uint8_t>(multivariate_n >> 24),
                             static_cast<uint8_t>(multivariate_n >> 16),
                             static_cast<uint8_t>(multivariate_n >> 8),
                             static_cast<uint8_t>(multivariate_n) });

    transcript.add_element("public_input_size",
                           { static_cast<uint8_t>(num_public_inputs >> 24),
                             static_cast<uint8_t>(num_public_inputs >> 16),
                             static_cast<uint8_t>(num_public_inputs >> 8),
                             static_cast<uint8_t>(num_public_inputs) });

    transcript.apply_fiat_shamir("init");
    transcript.apply_fiat_shamir("alpha");
    std::vector<uint8_t> public_inputs_buf(public_input_size, 1); // arbitrary buffer of 1's
    transcript.add_element("public_inputs", public_inputs_buf);
    transcript.apply_fiat_shamir("beta");

    return transcript;
}

TEST(Sumcheck, PolynomialNormalization)
{
    // Todo(Cody): We should not use real constants like this in the tests, at least not in so many of them.
    const size_t multivariate_d(3);
    const size_t multivariate_n(1 << multivariate_d);
    const size_t num_public_inputs(1);

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
    std::array<FF, multivariate_n> pow_zeta;
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

    auto transcript = produce_mocked_transcript(multivariate_d, num_public_inputs);

    auto sumcheck = Sumcheck<FF,
                             Transcript,
                             ArithmeticRelation,
                             GrandProductComputationRelation,
                             GrandProductInitializationRelation>(multivariate_n, transcript);

    sumcheck.execute_prover(full_polynomials);

    FF u_0 = transcript.get_challenge_field_element("u_0");
    FF u_1 = transcript.get_challenge_field_element("u_1");
    FF u_2 = transcript.get_challenge_field_element("u_2");

    /* sumcheck.execute_prover() terminates with sumcheck.multivariates.folded_polynoimals as an array such that
     * sumcheck.multivariates.folded_polynoimals[i][0] is the evaluatioin of the i'th multivariate at the vector of
     challenges u_i. What does this mean?

     Here we show that if the multivariate is F(X0, X1, X2) defined as above, then what we get is F(u0, u1, u2) and not,
     say F(u2, u1, u0). This is in accordance with Adrian's thesis (cf page 9).
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
        EXPECT_EQ(hand_computed_value, sumcheck.folded_polynomials[i][0]);
    }
}

TEST(Sumcheck, Prover)
{
    auto run_test = [](bool is_random_input) {
        const size_t multivariate_d(2);
        const size_t multivariate_n(1 << multivariate_d);
        const size_t num_public_inputs(1);
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

        auto transcript = produce_mocked_transcript(multivariate_d, num_public_inputs);
        auto sumcheck = Sumcheck<FF,
                                 Transcript,
                                 ArithmeticRelation,
                                 GrandProductComputationRelation,
                                 GrandProductInitializationRelation>(multivariate_n, transcript);

        sumcheck.execute_prover(full_polynomials);
        FF u_0 = transcript.get_challenge_field_element("u_0");
        FF u_1 = transcript.get_challenge_field_element("u_1");
        std::vector<FF> expected_values;
        for (auto& polynomial : full_polynomials) {
            // using knowledge of inputs here to derive the evaluation
            FF expected_lo = polynomial[0] * (FF(1) - u_0) + polynomial[1] * u_0;
            expected_lo *= (FF(1) - u_1);
            FF expected_hi = polynomial[2] * (FF(1) - u_0) + polynomial[3] * u_0;
            expected_hi *= u_1;
            expected_values.emplace_back(expected_lo + expected_hi);
        }
        // pull the sumcheck-produced multivariate evals out of the transcript
        auto sumcheck_evaluations = transcript.get_field_element_vector("multivariate_evaluations");
        for (size_t poly_idx = 0; poly_idx < NUM_POLYNOMIALS; poly_idx++) {
            EXPECT_EQ(sumcheck_evaluations[poly_idx], expected_values[poly_idx]);
        }
    };
    run_test(/* is_random_input=*/false);
    run_test(/* is_random_input=*/true);
}

// TODO(Cody): write standalone test of the verifier.
// TODO(luke): test possibly made obsolete by test ProverAndVerifierLonger
TEST(Sumcheck, ProverAndVerifier)
{
    const size_t multivariate_d(1);
    const size_t multivariate_n(1 << multivariate_d);
    const size_t num_public_inputs(1);

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

    auto transcript = produce_mocked_transcript(multivariate_d, num_public_inputs);

    auto sumcheck_prover = Sumcheck<FF,
                                    Transcript,
                                    ArithmeticRelation,
                                    GrandProductComputationRelation,
                                    GrandProductInitializationRelation>(multivariate_n, transcript);

    sumcheck_prover.execute_prover(full_polynomials);

    auto sumcheck_verifier = Sumcheck<FF,
                                      Transcript,
                                      ArithmeticRelation,
                                      GrandProductComputationRelation,
                                      GrandProductInitializationRelation>(transcript);

    bool verified = sumcheck_verifier.execute_verifier();
    ASSERT_TRUE(verified);
}

// TODO: make the inputs to this test more interesting, e.g. num_public_inputs > 0 and non-trivial permutations
TEST(Sumcheck, ProverAndVerifierLonger)
{
    auto run_test = [](bool expect_verified) {
        const size_t multivariate_d(2);
        const size_t multivariate_n(1 << multivariate_d);
        const size_t num_public_inputs(0);

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

        auto transcript = produce_mocked_transcript(multivariate_d, num_public_inputs);

        auto sumcheck_prover = Sumcheck<FF,
                                        Transcript,
                                        ArithmeticRelation,
                                        GrandProductComputationRelation,
                                        GrandProductInitializationRelation>(multivariate_n, transcript);

        sumcheck_prover.execute_prover(full_polynomials);

        auto sumcheck_verifier = Sumcheck<FF,
                                          Transcript,
                                          ArithmeticRelation,
                                          GrandProductComputationRelation,
                                          GrandProductInitializationRelation>(transcript);

        bool verified = sumcheck_verifier.execute_verifier();
        EXPECT_EQ(verified, expect_verified);
    };

    run_test(/* expect_verified=*/true);
    run_test(/* expect_verified=*/false);
}

} // namespace test_sumcheck_round
