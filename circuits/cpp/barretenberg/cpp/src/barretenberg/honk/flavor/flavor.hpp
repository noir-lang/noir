#pragma once
#include <array>
#include <string>
#include "barretenberg/common/log.hpp"
#include "barretenberg/proof_system/arithmetization/arithmetization.hpp"
#include "barretenberg/transcript/manifest.hpp"

namespace proof_system::honk {
// TODO(Cody) This _should_ be shared with Plonk, but it isn't.
struct StandardArithmetization {
    /**
     * @brief All of the multivariate polynomials used by the Standard Honk Prover.
     * @details The polynomials are broken into three categories: precomputed, witness, and shifted.
     * This separation must be maintained to allow for programmatic access, but the ordering of the
     * polynomials can be permuted within each category if necessary. Polynomials can also be added
     * or removed (assuming consistency with the prover algorithm) but the constants describing the
     * number of polynomials in each category must be manually updated.
     *
     */
    enum POLYNOMIAL {
        /* --- PRECOMPUTED POLYNOMIALS --- */
        Q_C,
        Q_L,
        Q_R,
        Q_O,
        Q_M,
        SIGMA_1,
        SIGMA_2,
        SIGMA_3,
        ID_1,
        ID_2,
        ID_3,
        LAGRANGE_FIRST,
        LAGRANGE_LAST, // = LAGRANGE_N-1 whithout ZK, but can be less
        /* --- WITNESS POLYNOMIALS --- */
        W_L,
        W_R,
        W_O,
        Z_PERM,
        /* --- SHIFTED POLYNOMIALS --- */
        Z_PERM_SHIFT,
        /* --- --- */
        COUNT // for programmatic determination of NUM_POLYNOMIALS
    };

    static constexpr size_t NUM_POLYNOMIALS = POLYNOMIAL::COUNT;
    static constexpr size_t NUM_SHIFTED_POLYNOMIALS = 1;
    static constexpr size_t NUM_PRECOMPUTED_POLYNOMIALS = 13;
    static constexpr size_t NUM_UNSHIFTED_POLYNOMIALS = NUM_POLYNOMIALS - NUM_SHIFTED_POLYNOMIALS;

    // *** WARNING: The order of this array must be manually updated to match POLYNOMIAL enum ***
    // TODO(luke): This is a temporary measure to associate the above enum with sting tags. Its only needed because
    // the
    // polynomials/commitments in the prover/verifier are stored in maps. This storage could be converted to simple
    // arrays at which point these string tags can be removed.
    inline static const std::array<std::string, 18> ENUM_TO_COMM = {
        "Q_C",           "Q_1",     "Q_2",  "Q_3",  "Q_M",    "SIGMA_1",
        "SIGMA_2",       "SIGMA_3", "ID_1", "ID_2", "ID_3",   "LAGRANGE_FIRST",
        "LAGRANGE_LAST", "W_1",     "W_2",  "W_3",  "Z_PERM", "Z_PERM_SHIFT"
    };
};
} // namespace proof_system::honk

namespace proof_system::honk {
struct StandardHonk {
  public:
    // This whole file is broken; changes here are in anticipation of a follow-up rework of the flavor specificaiton.
    using Arithmetization = arithmetization::Standard;
    using MULTIVARIATE = proof_system::honk::StandardArithmetization::POLYNOMIAL;
    // // TODO(Cody): Where to specify? is this polynomial manifest size?
    // static constexpr size_t STANDARD_HONK_MANIFEST_SIZE = 16;
    // TODO(Cody): Maybe relation should be supplied and this should be computed as is done in sumcheck?
    // Then honk::StandardHonk (or whatever we rename it) would become an alias for a Honk flavor with a
    // certain set of parameters, including the relations?
    static constexpr size_t MAX_RELATION_LENGTH = 5;

    // TODO(Cody): should extract this from the parameter pack. Maybe that should be done here?

    // num_sumcheck_rounds = 1 if using quotient polynomials, otherwise = number of sumcheck rounds
    static transcript::Manifest create_manifest(const size_t num_public_inputs, const size_t num_sumcheck_rounds = 1)
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
            /* num_challenges_in = */ 2)
        );

        // Rounds 4, ... 4 + num_sumcheck_rounds-1
        for (size_t i = 0; i < num_sumcheck_rounds; i++) {
            auto label = std::to_string(i);
            manifest_rounds.emplace_back(
                transcript::Manifest::RoundManifest(
            {
              { .name = "univariate_" + label, .num_bytes = fr_size * honk::StandardHonk::MAX_RELATION_LENGTH, .derived_by_verifier = false }
            },
            /* challenge_name = */ "u_" + label,
            /* num_challenges_in = */ 1));
        }

        // Round 5 + num_sumcheck_rounds
        manifest_rounds.emplace_back(transcript::Manifest::RoundManifest(
            {
              { .name = "multivariate_evaluations",     .num_bytes = fr_size * honk::StandardArithmetization::NUM_POLYNOMIALS, .derived_by_verifier = false, .challenge_map_index = 0 },
            },
            /* challenge_name = */ "rho",
            /* num_challenges_in = */ 1)); /* TODO(Cody): magic number! Where should this be specified? */

        // Rounds 6 + num_sumcheck_rounds, ... , 6 + 2 * num_sumcheck_rounds - 1
        std::vector<transcript::Manifest::ManifestEntry> fold_commitment_entries;
        for (size_t i = 1; i < num_sumcheck_rounds; i++) {
            fold_commitment_entries.emplace_back(transcript::Manifest::ManifestEntry(
              { .name = "FOLD_" + std::to_string(i), .num_bytes = g1_size, .derived_by_verifier = false }));
        };
        manifest_rounds.emplace_back(transcript::Manifest::RoundManifest(
            fold_commitment_entries,
            /* challenge_name = */ "r",
            /* num_challenges_in */ 1));

        // Rounds 6 + 2 * num_sumcheck_rounds, ..., 6 + 3 * num_sumcheck_rounds
        std::vector<transcript::Manifest::ManifestEntry> gemini_evaluation_entries;
        for (size_t i = 0; i < num_sumcheck_rounds; i++) {
            gemini_evaluation_entries.emplace_back(transcript::Manifest::ManifestEntry(
            { .name = "a_" + std::to_string(i), .num_bytes = fr_size, .derived_by_verifier = false }));
        };
        manifest_rounds.emplace_back(transcript::Manifest::RoundManifest(
            gemini_evaluation_entries,
            /* challenge_name = */ "nu",
            /* num_challenges_in */ 1));

        // Round 7 + 3 * num_sumcheck_rounds
        manifest_rounds.emplace_back(
            transcript::Manifest::RoundManifest(
            {
              { .name = "Q", .num_bytes = g1_size, .derived_by_verifier = false }
            },
            /* challenge_name = */ "z",
            /* num_challenges_in */ 1));

        // Round 8 + 3 * num_sumcheck_rounds
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

struct UltraArithmetization {
    /**
     * @brief All of the multivariate polynomials used by the Ultra Honk Prover.
     * @details The polynomials are broken into three categories: precomputed, witness, and shifted.
     * This separation must be maintained to allow for programmatic access, but the ordering of the
     * polynomials can be permuted within each category if necessary. Polynomials can also be added
     * or removed (assuming consistency with the prover algorithm) but the constants describing the
     * number of polynomials in each category must be manually updated.
     *
     */
    enum POLYNOMIAL {
        /* --- PRECOMPUTED POLYNOMIALS --- */
        Q_C,
        Q_L,
        Q_R,
        Q_O,
        Q_4,
        Q_M,
        QARITH,
        QSORT,
        QELLIPTIC,
        QAUX,
        QLOOKUPTYPE,
        SIGMA_1,
        SIGMA_2,
        SIGMA_3,
        SIGMA_4,
        ID_1,
        ID_2,
        ID_3,
        ID_4,
        TABLE_1,
        TABLE_2,
        TABLE_3,
        TABLE_4,
        LAGRANGE_FIRST,
        LAGRANGE_LAST, // = LAGRANGE_N-1 whithout ZK, but can be less
        /* --- WITNESS POLYNOMIALS --- */
        W_L,
        W_R,
        W_O,
        W_4,
        S_1,
        S_2,
        S_3,
        S_4,
        S_ACCUM,
        Z_PERM,
        Z_LOOKUP,
        /* --- SHIFTED POLYNOMIALS --- */
        W_1_SHIFT,
        W_2_SHIFT,
        W_3_SHIFT,
        W_4_SHIFT,
        TABLE_1_SHIFT,
        TABLE_2_SHIFT,
        TABLE_3_SHIFT,
        TABLE_4_SHIFT,
        S_ACCUM_SHIFT,
        Z_PERM_SHIFT,
        Z_LOOKUP_SHIFT,
        /* --- --- */
        COUNT // for programmatic determination of NUM_POLYNOMIALS
    };
};
} // namespace proof_system::honk
