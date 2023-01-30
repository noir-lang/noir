#pragma once
#include "proof_system/types/polynomial_manifest.hpp"
#include <common/log.hpp>
#include <transcript/manifest.hpp>

#define STANDARD_HONK_WIDTH 3
// TODO(Cody): "bonk" is short for "both plonk and honk". Just need a short and non-vague temporary name.
namespace bonk {
struct StandardArithmetization {
    enum POLYNOMIAL {
        W_L,
        W_R,
        W_O,
        Z_PERM,
        Z_PERM_SHIFT,
        Q_M,
        Q_L,
        Q_R,
        Q_O,
        Q_C,
        SIGMA_1,
        SIGMA_2,
        SIGMA_3,
        ID_1,
        ID_2,
        ID_3,
        LAGRANGE_FIRST,
        LAGRANGE_LAST, // = LAGRANGE_N-1 whithout ZK, but can be less
        COUNT
    };

    static constexpr size_t NUM_POLYNOMIALS = POLYNOMIAL::COUNT;
};
} // namespace bonk

namespace honk {
struct StandardHonk {
  public:
    using Arithmetization = bonk::StandardArithmetization;
    using MULTIVARIATE = Arithmetization::POLYNOMIAL;
    // // TODO(Cody): Where to specify? is this polynomial manifest size?
    // static constexpr size_t STANDARD_HONK_MANIFEST_SIZE = 16;
    static constexpr size_t MAX_RELATION_LENGTH = 5; // TODO(Cody): increment after fixing add_edge_contribution; kill
                                                     // after moving barycentric class out of relations

    // TODO(Cody): should extract this from the parameter pack. Maybe that should be done here?

    // num_sumcheck_rounds = 1 if using quotient polynomials, otherwise = number of sumcheck rounds
    static transcript::Manifest create_unrolled_manifest(const size_t num_public_inputs,
                                                         const size_t num_sumcheck_rounds = 1)
    {
        constexpr size_t g1_size = 64;
        constexpr size_t fr_size = 32;
        const size_t public_input_size = fr_size * num_public_inputs;
        // clang-format off
        /*  A RoundManifest describes data that will be put in or extracted from a transcript.
            Here we have (1 + 7 + num_sumcheck_rounds)-many RoundManifests. */
        std::vector<transcript::Manifest::RoundManifest> manifest_rounds;

        // Round 0
        manifest_rounds.emplace_back(transcript::Manifest::RoundManifest(
            { 
              { .name = "circuit_size",      .num_bytes = 4, .derived_by_verifier = true },
              { .name = "public_input_size", .num_bytes = 4, .derived_by_verifier = true } 
            },
            /* challenge_name = */ "init",
            /* num_challenges_in = */ 1));
        
        // Round 1
        manifest_rounds.emplace_back(transcript::Manifest::RoundManifest(
            { /* this is a noop */ },
            /* challenge_name = */ "eta",
            /* num_challenges_in = */ 0));

        // Round 2
        manifest_rounds.emplace_back(transcript::Manifest::RoundManifest(
            {
              { .name = "public_inputs", .num_bytes = public_input_size, .derived_by_verifier = false },
              { .name = "W_1",           .num_bytes = g1_size,           .derived_by_verifier = false },
              { .name = "W_2",           .num_bytes = g1_size,           .derived_by_verifier = false },
              { .name = "W_3",           .num_bytes = g1_size,           .derived_by_verifier = false },
            },
            /* challenge_name = */ "beta",
            /* num_challenges_in = */ 2) // also produce "gamma"
        );

        // Round 3
        manifest_rounds.emplace_back(transcript::Manifest::RoundManifest(
            { { .name = "Z_PERM", .num_bytes = g1_size, .derived_by_verifier = false } },
            /* challenge_name = */ "alpha",
            /* num_challenges_in = */ 1));

        // Rounds 3 + 1, ... 3 + num_sumcheck_rounds
        for (size_t i = 0; i < num_sumcheck_rounds; i++) {
            auto label = std::to_string(num_sumcheck_rounds - i);
            manifest_rounds.emplace_back(
                transcript::Manifest::RoundManifest(
            { 
              { .name = "univariate_" + label, .num_bytes = fr_size * honk::StandardHonk::MAX_RELATION_LENGTH, .derived_by_verifier = false } 
            },
            /* challenge_name = */ "u_" + label,
            /* num_challenges_in = */ 1));
        }

        // Rounds 4 + num_sumcheck_rounds
        manifest_rounds.emplace_back(transcript::Manifest::RoundManifest(       
            {
              { .name = "multivariate_evaluations",     .num_bytes = fr_size * waffle::STANDARD_HONK_TOTAL_NUM_POLYS, .derived_by_verifier = false, .challenge_map_index = 0 },
            },
            /* challenge_name = */ "rho",
            /* num_challenges_in = */ 1)); /* TODO(Cody): magic number! Where should this be specified? */

        // Rounds 5 + num_sumcheck_rounds
        std::vector<transcript::Manifest::ManifestEntry> fold_commitment_entries;
        for (size_t i = 1; i < num_sumcheck_rounds; i++) {
            fold_commitment_entries.emplace_back(transcript::Manifest::ManifestEntry(
              { .name = "FOLD_" + std::to_string(i), .num_bytes = g1_size, .derived_by_verifier = false }));
        };
        manifest_rounds.emplace_back(transcript::Manifest::RoundManifest(
            fold_commitment_entries,
            /* challenge_name = */ "r",
            /* num_challenges_in */ 1));

        // Rounds 6 + num_sumcheck_rounds
        std::vector<transcript::Manifest::ManifestEntry> gemini_evaluation_entries;
        for (size_t i = 0; i < num_sumcheck_rounds; i++) {
            gemini_evaluation_entries.emplace_back(transcript::Manifest::ManifestEntry(
            { .name = "a_" + std::to_string(i), .num_bytes = fr_size, .derived_by_verifier = false }));
        };
        manifest_rounds.emplace_back(transcript::Manifest::RoundManifest(
            gemini_evaluation_entries,
            /* challenge_name = */ "nu",
            /* num_challenges_in */ 1));

        // Rounds 7 + num_sumcheck_rounds
        manifest_rounds.emplace_back(
            transcript::Manifest::RoundManifest(
            { 
              { .name = "Q", .num_bytes = g1_size, .derived_by_verifier = false } 
            },
            /* challenge_name = */ "z",
            /* num_challenges_in */ 1));

        // Rounds 8 + num_sumcheck_rounds
        manifest_rounds.emplace_back(
            transcript::Manifest::RoundManifest(
            { 
              { .name = "W", .num_bytes = g1_size, .derived_by_verifier = false }
            },
            /* challenge_name = */ "separator",
            /* num_challenges_in */ 1));

        // clang-format on

        auto output = transcript::Manifest(manifest_rounds);
        return output;
    }
};
} // namespace honk