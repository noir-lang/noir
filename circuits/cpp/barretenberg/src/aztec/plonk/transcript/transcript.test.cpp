#include "transcript.hpp"
#include <gtest/gtest.h>

namespace {
transcript::Manifest create_manifest(const size_t num_public_inputs)
{
    // add public inputs....
    constexpr size_t g1_size = 64;
    constexpr size_t fr_size = 32;
    const size_t public_input_size = fr_size * num_public_inputs;
    const transcript::Manifest output = transcript::Manifest(
        { transcript::Manifest::RoundManifest({ { "circuit_size", 4, true }, { "public_input_size", 4, true } },
                                              "init"),
          transcript::Manifest::RoundManifest({ { "public_inputs", public_input_size, false },
                                                { "W_1", g1_size, false },
                                                { "W_2", g1_size, false },
                                                { "W_3", g1_size, false } },
                                              "beta"),
          transcript::Manifest::RoundManifest({ {} }, "gamma"),
          transcript::Manifest::RoundManifest({ { "Z", g1_size, false } }, "alpha"),
          transcript::Manifest::RoundManifest(
              { { "T_1", g1_size, false }, { "T_2", g1_size, false }, { "T_3", g1_size, false } }, "z"),
          transcript::Manifest::RoundManifest({ { "w_1", fr_size, false },
                                                { "w_2", fr_size, false },
                                                { "w_3", fr_size, false },
                                                { "w_3_omega", fr_size, false },
                                                { "z_omega", fr_size, false },
                                                { "sigma_1", fr_size, false },
                                                { "sigma_2", fr_size, false },
                                                { "r", fr_size, false },
                                                { "t", fr_size, true } },
                                              "nu"),
          transcript::Manifest::RoundManifest({ { "PI_Z", g1_size, false }, { "PI_Z_OMEGA", g1_size, false } },
                                              "separator") });
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
    // transcript.add_element("Q_M", g1_vector);
    // transcript.add_element("Q_L", g1_vector);
    // transcript.add_element("Q_R", g1_vector);
    // transcript.add_element("Q_O", g1_vector);
    // transcript.add_element("Q_C", g1_vector);
    // transcript.add_element("SIGMA_1", g1_vector);
    // transcript.add_element("SIGMA_2", g1_vector);
    // transcript.add_element("SIGMA_3", g1_vector);
    // transcript.add_element("G2", g2_vector);
    // transcript.add_element("T2", g2_vector);
    transcript.apply_fiat_shamir("init");

    transcript.add_element("public_inputs", {});
    transcript.add_element("W_1", g1_vector);
    transcript.add_element("W_2", g1_vector);
    transcript.add_element("W_3", g1_vector);
    transcript.apply_fiat_shamir("beta");
    transcript.apply_fiat_shamir("gamma");

    transcript.add_element("Z", g1_vector);
    transcript.apply_fiat_shamir("alpha");

    transcript.add_element("T_1", g1_vector);
    transcript.add_element("T_2", g1_vector);
    transcript.add_element("T_3", g1_vector);
    transcript.apply_fiat_shamir("z");

    transcript.add_element("w_1", fr_vector);
    transcript.add_element("w_2", fr_vector);
    transcript.add_element("w_3", fr_vector);
    transcript.add_element("z_omega", fr_vector);
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
