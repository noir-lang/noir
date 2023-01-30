#include "sumcheck.hpp"
#include "proof_system/flavor/flavor.hpp"
#include "transcript/transcript_wrappers.hpp"
#include "polynomials/multivariates.hpp"
#include "relations/arithmetic_relation.hpp"
#include "relations/grand_product_computation_relation.hpp"
#include "relations/grand_product_initialization_relation.hpp"
#include "transcript/manifest.hpp"
#include <array>
#include <cstddef>
#include <cstdint>
#include <ecc/curves/bn254/fr.hpp>
#include <gtest/internal/gtest-internal.h>
#include <numeric/random/engine.hpp>

#include <initializer_list>
#include <gtest/gtest.h>
#include <string>
#include <sys/types.h>
#include <vector>

#pragma GCC diagnostic ignored "-Wunused-variable"

using namespace honk;
using namespace honk::sumcheck;

namespace test_sumcheck_round {

using Transcript = transcript::StandardTranscript;
using FF = barretenberg::fr;

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
                                                                     /* num_challenges_in = */ 1));
    manifest_rounds.emplace_back(transcript::Manifest::RoundManifest(
        { { .name = "public_inputs", .num_bytes = public_input_size, .derived_by_verifier = false } },
        /* challenge_name = */ "beta",
        /* num_challenges_in = */ 2) // also produce "gamma"
    );

    for (size_t i = 0; i < multivariate_d; i++) {
        auto label = std::to_string(multivariate_d - i);
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
    const size_t num_polys(bonk::StandardArithmetization::NUM_POLYNOMIALS);
    const size_t multivariate_d(3);
    const size_t multivariate_n(1 << multivariate_d);
    const size_t num_public_inputs(1);

    constexpr size_t fr_size = 32;

    using Multivariates = ::Multivariates<FF, num_polys>;

    // clang-format off
    std::array<FF, multivariate_n> w_l =            { 0, 1, 2, 3, 4, 5, 6, 7 };
    std::array<FF, multivariate_n> w_r =            { 0, 0, 0, 0, 0, 0, 0, 0 };
    std::array<FF, multivariate_n> w_o =            { 0, 0, 0, 0, 0, 0, 0, 0 };
    std::array<FF, multivariate_n> z_perm =         { 0, 0, 0, 0, 0, 0, 0, 0 };
    std::array<FF, multivariate_n> z_perm_shift =   { 0, 0, 0, 0, 0, 0, 0, 0 };
    std::array<FF, multivariate_n> q_m =            { 0, 0, 0, 0, 0, 0, 0, 0 };
    std::array<FF, multivariate_n> q_l =            { 0, 0, 0, 0, 0, 0, 0, 0 };
    std::array<FF, multivariate_n> q_r =            { 0, 0, 0, 0, 0, 0, 0, 0 };
    std::array<FF, multivariate_n> q_o =            { 0, 0, 0, 0, 0, 0, 0, 0 };
    std::array<FF, multivariate_n> q_c =            { 0, 0, 0, 0, 0, 0, 0, 0 };
    std::array<FF, multivariate_n> sigma_1 =        { 0, 0, 0, 0, 0, 0, 0, 0 };
    std::array<FF, multivariate_n> sigma_2 =        { 0, 0, 0, 0, 0, 0, 0, 0 };
    std::array<FF, multivariate_n> sigma_3 =        { 0, 0, 0, 0, 0, 0, 0, 0 };
    std::array<FF, multivariate_n> id_1 =           { 0, 0, 0, 0, 0, 0, 0, 0 };
    std::array<FF, multivariate_n> id_2 =           { 0, 0, 0, 0, 0, 0, 0, 0 };
    std::array<FF, multivariate_n> id_3 =           { 0, 0, 0, 0, 0, 0, 0, 0 };
    std::array<FF, multivariate_n> lagrange_first = { 0, 0, 0, 0, 0, 0, 0, 0 };
    std::array<FF, multivariate_n> lagrange_last =  { 0, 0, 0, 0, 0, 0, 0, 0 };
    // clang-format on

    // These will be owned outside the class, probably by the composer.
    std::array<std::span<FF>, Multivariates::num> full_polynomials = {
        w_l,     w_r,  w_o,  z_perm, z_perm_shift,   q_m,          q_l, q_r, q_o, q_c, sigma_1, sigma_2,
        sigma_3, id_1, id_2, id_3,   lagrange_first, lagrange_last
    };

    auto transcript = produce_mocked_transcript(multivariate_d, num_public_inputs);

    auto multivariates = Multivariates(full_polynomials);

    auto sumcheck = Sumcheck<Multivariates,
                             Transcript,
                             ArithmeticRelation,
                             GrandProductComputationRelation,
                             GrandProductInitializationRelation>(multivariates, transcript);

    sumcheck.execute_prover();

    FF u_1 = transcript.get_challenge_field_element("u_1");
    FF u_2 = transcript.get_challenge_field_element("u_2");
    FF u_3 = transcript.get_challenge_field_element("u_3");

    /* sumcheck.execute_prover() terminates with sumcheck.multivariates.folded_polynoimals as an array such that
     * sumcheck.multivariates.folded_polynoimals[i][0] is the evaluatioin of the i'th multivariate at the vector of
     challenges u_i. What does this mean?

     Here we show that if the multivariate is F(X1, X2, X3) defined as above, then what we get is F(u1, u2, u3) and not,
     say F(u3,u2,u1). This is in accordance with Adrian's thesis (cf page 9).
      */

    // Get the values of the Lagrange basis polys L_i defined
    // by: L_i(v) = 1 if i = v, 0 otherwise, for v from 0 to 7.
    // clang-format off
    FF l_0 = (FF(1) - u_1) * (FF(1) - u_2) * (FF(1) - u_3);
    FF l_1 = (FF(1) - u_1) * (FF(1) - u_2) * (        u_3);
    FF l_2 = (FF(1) - u_1) * (        u_2) * (FF(1) - u_3);
    FF l_3 = (FF(1) - u_1) * (        u_2) * (        u_3);
    FF l_4 = (        u_1) * (FF(1) - u_2) * (FF(1) - u_3);
    FF l_5 = (        u_1) * (FF(1) - u_2) * (        u_3);
    FF l_6 = (        u_1) * (        u_2) * (FF(1) - u_3);
    FF l_7 = (        u_1) * (        u_2) * (        u_3);
    // clang-format on

    FF hand_computed_value = l_0 * w_l[0] + l_1 * w_l[1] + l_2 * w_l[2] + l_3 * w_l[3] + l_4 * w_l[4] + l_5 * w_l[5] +
                             l_6 * w_l[6] + l_7 * w_l[7];

    EXPECT_EQ(hand_computed_value, sumcheck.multivariates.folded_polynomials[0][0]);
}

TEST(Sumcheck, Prover)
{
    const size_t num_polys(bonk::StandardArithmetization::NUM_POLYNOMIALS);
    const size_t multivariate_d(2);
    const size_t multivariate_n(1 << multivariate_d);
    const size_t num_public_inputs(1);

    // const size_t max_relation_length = 4;
    constexpr size_t fr_size = 32;

    using Multivariates = ::Multivariates<FF, num_polys>;

    // clang-format off
    std::array<FF, multivariate_n> w_l =            { 1, 2, 0, 0};
    std::array<FF, multivariate_n> w_r =            { 1, 2, 0, 0};
    std::array<FF, multivariate_n> w_o =            { 1, 2, 0, 0};
    std::array<FF, multivariate_n> z_perm =         { 1, 2, 0, 0};
    std::array<FF, multivariate_n> z_perm_shift =   { 0, 2, 0, 0};
    std::array<FF, multivariate_n> q_m =            { 1, 2, 0, 0};
    std::array<FF, multivariate_n> q_l =            { 1, 2, 0, 0};
    std::array<FF, multivariate_n> q_r =            { 1, 2, 0, 0};
    std::array<FF, multivariate_n> q_o =            { 1, 2, 0, 0};
    std::array<FF, multivariate_n> q_c =            { 1, 2, 0, 0};
    std::array<FF, multivariate_n> sigma_1 =        { 1, 2, 0, 0};
    std::array<FF, multivariate_n> sigma_2 =        { 1, 2, 0, 0};
    std::array<FF, multivariate_n> sigma_3 =        { 1, 2, 0, 0};
    std::array<FF, multivariate_n> id_1 =           { 1, 2, 0, 0};
    std::array<FF, multivariate_n> id_2 =           { 1, 2, 0, 0};
    std::array<FF, multivariate_n> id_3 =           { 1, 2, 0, 0};
    std::array<FF, multivariate_n> lagrange_first = { 1, 2, 0, 0};
    std::array<FF, multivariate_n> lagrange_last =  { 1, 2, 0, 0};
    // clang-format on

    // These will be owned outside the class, probably by the composer.
    std::array<std::span<FF>, Multivariates::num> full_polynomials = {
        w_l,     w_r,  w_o,  z_perm, z_perm_shift,   q_m,          q_l, q_r, q_o, q_c, sigma_1, sigma_2,
        sigma_3, id_1, id_2, id_3,   lagrange_first, lagrange_last
    };

    auto transcript = produce_mocked_transcript(multivariate_d, num_public_inputs);

    auto multivariates = Multivariates(full_polynomials);

    auto sumcheck = Sumcheck<Multivariates,
                             Transcript,
                             ArithmeticRelation,
                             GrandProductComputationRelation,
                             GrandProductInitializationRelation>(multivariates, transcript);

    sumcheck.execute_prover();

    FF u_1 = transcript.get_challenge_field_element("u_1");
    FF u_2 = transcript.get_challenge_field_element("u_2");
    std::vector<FF> expected_values;
    for (auto& polynomial : full_polynomials) {
        // using knowledge of inputs here to derive the evaluation
        FF expected = polynomial[0] * (FF(1) - u_2) + polynomial[1] * u_2;
        expected *= (FF(1) - u_1);
        expected_values.emplace_back(expected);
    }

    // pull the sumcheck-produced multivariate evals out of the transcript
    auto sumcheck_evaluations = transcript.get_field_element_vector("multivariate_evaluations");
    for (size_t poly_idx = 0; poly_idx < num_polys; poly_idx++) {
        EXPECT_EQ(sumcheck_evaluations[poly_idx], expected_values[poly_idx]);
    }
}

// TODO(Cody): write standalone test of the verifier.
// TODO(luke): test possibly made obsolete by test ProverAndVerifierLonger
TEST(Sumcheck, ProverAndVerifier)
{
    const size_t num_polys(bonk::StandardArithmetization::NUM_POLYNOMIALS);
    const size_t multivariate_d(1);
    const size_t multivariate_n(1 << multivariate_d);
    const size_t num_public_inputs(1);

    const size_t max_relation_length = 4 /* honk::StandardHonk::MAX_RELATION_LENGTH */;
    constexpr size_t fr_size = 32;

    using Multivariates = ::Multivariates<FF, num_polys>;

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

    // These will be owned outside the class, probably by the composer.
    std::array<std::span<FF>, Multivariates::num> full_polynomials = {
        w_l,     w_r,  w_o,  z_perm, z_perm_shift,   q_m,          q_l, q_r, q_o, q_c, sigma_1, sigma_2,
        sigma_3, id_1, id_2, id_3,   lagrange_first, lagrange_last
    };

    auto transcript = produce_mocked_transcript(multivariate_d, num_public_inputs);

    auto multivariates = Multivariates(full_polynomials);

    auto sumcheck_prover = Sumcheck<Multivariates,
                                    Transcript,
                                    ArithmeticRelation,
                                    GrandProductComputationRelation,
                                    GrandProductInitializationRelation>(multivariates, transcript);

    sumcheck_prover.execute_prover();

    auto sumcheck_verifier = Sumcheck<Multivariates,
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
        const size_t num_polys(bonk::StandardArithmetization::NUM_POLYNOMIALS);
        const size_t multivariate_d(2);
        const size_t multivariate_n(1 << multivariate_d);
        const size_t num_public_inputs(0);

        const size_t max_relation_length = honk::StandardHonk::MAX_RELATION_LENGTH;
        constexpr size_t fr_size = 32;

        using Multivariates = ::Multivariates<FF, num_polys>;

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
    std::array<FF, multivariate_n> sigma_1        = { 0,  0,  0, 0 };
    std::array<FF, multivariate_n> sigma_2        = { 0,  0,  0, 0 };
    std::array<FF, multivariate_n> sigma_3        = { 0,  0,  0, 0 };
    std::array<FF, multivariate_n> id_1           = { 0,  0,  0, 0 };
    std::array<FF, multivariate_n> id_2           = { 0,  0,  0, 0 };
    std::array<FF, multivariate_n> id_3           = { 0,  0,  0, 0 };
    std::array<FF, multivariate_n> lagrange_first = { 1,  0,  0, 0 };
    std::array<FF, multivariate_n> lagrange_last  = { 0,  0,  0, 1 };
        // clang-format on

        // These will be owned outside the class, probably by the composer.
        std::array<std::span<FF>, Multivariates::num> full_polynomials = {
            w_l,     w_r,  w_o,  z_perm, z_perm_shift,   q_m,          q_l, q_r, q_o, q_c, sigma_1, sigma_2,
            sigma_3, id_1, id_2, id_3,   lagrange_first, lagrange_last
        };

        auto transcript = produce_mocked_transcript(multivariate_d, num_public_inputs);

        auto multivariates = Multivariates(full_polynomials);

        auto sumcheck_prover = Sumcheck<Multivariates,
                                        Transcript,
                                        ArithmeticRelation,
                                        GrandProductComputationRelation,
                                        GrandProductInitializationRelation>(multivariates, transcript);

        sumcheck_prover.execute_prover();

        auto sumcheck_verifier = Sumcheck<Multivariates,
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
