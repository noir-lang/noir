#include "transcript_wrappers.hpp"
#include <gtest/gtest.h>
#include "../honk/sumcheck/polynomials/univariate.hpp"

namespace {
transcript::Manifest create_manifest(const size_t num_public_inputs)
{
    // add public inputs....
    constexpr size_t g1_size = 64;
    constexpr size_t fr_size = 32;
    const size_t public_input_size = fr_size * num_public_inputs;
    const transcript::Manifest output = transcript::Manifest(
        { transcript::Manifest::RoundManifest(
              { { "circuit_size", 4, true }, { "public_input_size", 4, true } }, "init", 1),
          transcript::Manifest::RoundManifest({ { "public_inputs", public_input_size, false },
                                                { "W_1", g1_size, false },
                                                { "W_2", g1_size, false },
                                                { "W_3", g1_size, false } },
                                              "beta",
                                              2),
          transcript::Manifest::RoundManifest({ { "Z_PERM", g1_size, false } }, "alpha", 1),
          transcript::Manifest::RoundManifest(
              { { "T_1", g1_size, false }, { "T_2", g1_size, false }, { "T_3", g1_size, false } }, "z", 1),
          transcript::Manifest::RoundManifest({ { "w_1", fr_size, false },
                                                { "w_2", fr_size, false },
                                                { "w_3", fr_size, false },
                                                { "w_3_omega", fr_size, false },
                                                { "z_perm_omega", fr_size, false },
                                                { "sigma_1", fr_size, false },
                                                { "sigma_2", fr_size, false },
                                                { "r", fr_size, false },
                                                { "t", fr_size, true } },
                                              "nu",
                                              10),
          transcript::Manifest::RoundManifest(
              { { "PI_Z", g1_size, false }, { "PI_Z_OMEGA", g1_size, false } }, "separator", 1) });
    return output;
}
} // namespace

TEST(transcript, validate_transcript)
{
    std::vector<uint8_t> g1_vector(64);
    std::vector<uint8_t> g2_vector(128);
    std::vector<uint8_t> fr_vector(32);

    for (size_t i = 0; i < g1_vector.size(); ++i) {
        g1_vector[i] = 1;
    }
    for (size_t i = 0; i < g2_vector.size(); ++i) {
        g2_vector[i] = 1;
    }
    for (size_t i = 0; i < fr_vector.size(); ++i) {
        fr_vector[i] = 1;
    }
    transcript::Transcript transcript = transcript::Transcript(create_manifest(0));
    transcript.add_element("circuit_size", { 1, 2, 3, 4 });
    transcript.add_element("public_input_size", { 1, 2, 3, 4 });
    transcript.apply_fiat_shamir("init");

    transcript.add_element("public_inputs", {});

    transcript.add_element("W_1", g1_vector);
    transcript.add_element("W_2", g1_vector);
    transcript.add_element("W_3", g1_vector);

    transcript.apply_fiat_shamir("beta");

    transcript.add_element("Z_PERM", g1_vector);

    transcript.apply_fiat_shamir("alpha");

    transcript.add_element("T_1", g1_vector);
    transcript.add_element("T_2", g1_vector);
    transcript.add_element("T_3", g1_vector);

    transcript.apply_fiat_shamir("z");

    transcript.add_element("w_1", fr_vector);
    transcript.add_element("w_2", fr_vector);
    transcript.add_element("w_3", fr_vector);
    transcript.add_element("w_3_omega", fr_vector);
    transcript.add_element("z_perm_omega", fr_vector);
    transcript.add_element("sigma_1", fr_vector);
    transcript.add_element("sigma_2", fr_vector);
    transcript.add_element("r", fr_vector);
    transcript.add_element("t", fr_vector);

    transcript.apply_fiat_shamir("nu");

    transcript.add_element("PI_Z", g1_vector);
    transcript.add_element("PI_Z_OMEGA", g1_vector);

    transcript.apply_fiat_shamir("separator");

    std::vector<uint8_t> result = transcript.get_element("PI_Z_OMEGA");
    EXPECT_EQ(result.size(), g1_vector.size());
    for (size_t i = 0; i < result.size(); ++i) {
        EXPECT_EQ(result[i], g1_vector[i]);
    }
}

namespace {
transcript::Manifest create_toy_honk_manifest(const size_t num_public_inputs, const size_t SUMCHECK_RELATION_LENGTH)
{
    // Create a toy honk manifest that includes a univariate like the ones constructed by the prover in each round of
    // sumcheck
    constexpr size_t g1_size = 64;
    constexpr size_t fr_size = 32;
    const size_t public_input_size = fr_size * num_public_inputs;
    const transcript::Manifest output = transcript::Manifest(
        { transcript::Manifest::RoundManifest(
              { { "circuit_size", 4, true }, { "public_input_size", 4, true } }, "init", 1),
          transcript::Manifest::RoundManifest({ { "public_inputs", public_input_size, false },
                                                { "W_1", g1_size, false },
                                                { "W_2", g1_size, false },
                                                { "W_3", g1_size, false } },
                                              "beta",
                                              2),
          transcript::Manifest::RoundManifest(
              { { "round_univariate_i", fr_size * SUMCHECK_RELATION_LENGTH, false } }, "omicron", 1) });
    return output;
}
} // namespace

/**
 * @brief Test transcript serialization/deserialization of a Univariate object, which is needed for Sumcheck
 *
 */
TEST(transcript, univariate_serialization)
{
    constexpr size_t num_public_inputs = 0;
    constexpr size_t LENGTH = 8;

    using Fr = barretenberg::fr;
    using Univariate = honk::sumcheck::Univariate<Fr, LENGTH>;
    using Transcript = transcript::StandardTranscript;

    std::vector<uint8_t> g1_vector(64);
    std::vector<uint8_t> fr_vector(32);
    std::array<Fr, LENGTH> evaluations;

    for (size_t i = 0; i < g1_vector.size(); ++i) {
        g1_vector[i] = 1;
    }
    for (size_t i = 0; i < fr_vector.size(); ++i) {
        fr_vector[i] = 1;
    }
    for (size_t i = 0; i < LENGTH; ++i) {
        evaluations[i] = Fr::random_element();
    }

    // Instantiate a StandardTranscript
    auto transcript = Transcript(create_toy_honk_manifest(num_public_inputs, LENGTH));

    // Add some junk to the transcript and compute challenges
    transcript.add_element("circuit_size", { 1, 2, 3, 4 });
    transcript.add_element("public_input_size", { 1, 2, 3, 4 });

    transcript.apply_fiat_shamir("init");

    transcript.add_element("public_inputs", {});
    transcript.add_element("W_1", g1_vector);
    transcript.add_element("W_2", g1_vector);
    transcript.add_element("W_3", g1_vector);

    transcript.apply_fiat_shamir("beta");

    // Instantiate a Univariate from the evaluations
    auto univariate = Univariate(evaluations);

    // Add the univariate to the transcript using the to_buffer() member function
    // NOTE: the element being added here does not actually have to exist in the manifest for the
    // serialization/deserialization to work.
    transcript.add_element("round_univariate_i", univariate.to_buffer());

    // Example of challenge generation; not used in test
    transcript.apply_fiat_shamir("omicron");
    Fr omicron = Fr::serialize_from_buffer(transcript.get_challenge("omicron").begin());
    static_cast<void>(omicron);

    // Deserialize into a univariate from the transcript
    auto deserialized_univariate = Univariate::serialize_from_buffer(transcript.get_element("round_univariate_i"));

    for (size_t i = 0; i < LENGTH; ++i) {
        EXPECT_EQ(univariate.value_at(i), deserialized_univariate.value_at(i));
    }
}
