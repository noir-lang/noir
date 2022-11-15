#include <plonk/composer/standard_composer.hpp>
#include <plonk/transcript/transcript.hpp>

namespace waffle {
inline transcript::StandardTranscript create_dummy_standard_transcript()
{
    std::vector<uint8_t> g1_vector(64);
    std::vector<uint8_t> fr_vector(32);

    for (size_t i = 0; i < g1_vector.size(); ++i) {
        g1_vector[i] = 1;
    }

    for (size_t i = 0; i < fr_vector.size(); ++i) {
        fr_vector[i] = 1;
    }
    transcript::StandardTranscript transcript =
        transcript::StandardTranscript(waffle::StandardComposer::create_manifest(0));
    transcript.add_element("circuit_size", { 1, 2, 3, 4 });
    transcript.add_element("public_input_size", { 0, 0, 0, 0 });
    transcript.apply_fiat_shamir("init");
    transcript.add_element("public_inputs", {});
    transcript.add_element("W_1", g1_vector);
    transcript.add_element("W_2", g1_vector);
    transcript.add_element("W_3", g1_vector);
    transcript.apply_fiat_shamir("beta");
    transcript.add_element("Z_PERM", g1_vector);
    transcript.apply_fiat_shamir("alpha");
    return transcript;
}

inline transcript::Manifest create_dummy_ultra_manifest(const size_t num_public_inputs)
{
    // add public inputs....
    constexpr size_t g1_size = 64;
    constexpr size_t fr_size = 32;
    const size_t public_input_size = fr_size * num_public_inputs;
    const transcript::Manifest output =
        transcript::Manifest({ transcript::Manifest::RoundManifest(
                                   { { "circuit_size", 4, true }, { "public_input_size", 4, true } }, "init", 1),
                               transcript::Manifest::RoundManifest({ { "public_inputs", public_input_size, false },
                                                                     { "W_1", g1_size, false },
                                                                     { "W_2", g1_size, false },
                                                                     { "W_3", g1_size, false },
                                                                     { "W_4", g1_size, false } },
                                                                   "eta",
                                                                   1),
                               transcript::Manifest::RoundManifest({ { "S", g1_size, false } }, "beta", 2),
                               transcript::Manifest::RoundManifest({ { "Z_PERM", g1_size, false } }, "alpha", 1),
                               transcript::Manifest::RoundManifest({ { "T_1", g1_size, false },
                                                                     { "T_2", g1_size, false },
                                                                     { "T_3", g1_size, false },
                                                                     { "T_4", g1_size, false } },
                                                                   "z",
                                                                   1),
                               transcript::Manifest::RoundManifest({ { "w_1", fr_size, false },
                                                                     { "w_2", fr_size, false },
                                                                     { "w_3", fr_size, false },
                                                                     { "w_4", fr_size, false },
                                                                     { "z_perm_omega", fr_size, false },
                                                                     { "sigma_1", fr_size, false },
                                                                     { "sigma_2", fr_size, false },
                                                                     { "sigma_3", fr_size, false },
                                                                     { "q_arith", fr_size, false },
                                                                     { "q_ecc_1", fr_size, false },
                                                                     { "q_c", fr_size, false },
                                                                     { "r", fr_size, false },
                                                                     { "w_1_omega", fr_size, false },
                                                                     { "w_2_omega", fr_size, false },
                                                                     { "w_3_omega", fr_size, false },
                                                                     { "w_4_omega", fr_size, false },
                                                                     { "t", fr_size, true } },
                                                                   "nu",
                                                                   12,
                                                                   true),
                               transcript::Manifest::RoundManifest(
                                   { { "PI_Z", g1_size, false }, { "PI_Z_OMEGA", g1_size, false } }, "separator", 1) });
    return output;
}

inline transcript::StandardTranscript create_dummy_ultra_transcript()
{
    std::vector<uint8_t> g1_vector(64);
    std::vector<uint8_t> fr_vector(32);

    for (size_t i = 0; i < g1_vector.size(); ++i) {
        g1_vector[i] = 1;
    }

    for (size_t i = 0; i < fr_vector.size(); ++i) {
        fr_vector[i] = 1;
    }
    transcript::StandardTranscript transcript = transcript::StandardTranscript(create_dummy_ultra_manifest(0));
    transcript.add_element("circuit_size", { 1, 2, 3, 4 });
    transcript.add_element("public_input_size", { 0, 0, 0, 0 });
    transcript.apply_fiat_shamir("init");
    transcript.add_element("public_inputs", {});
    transcript.add_element("W_1", g1_vector);
    transcript.add_element("W_2", g1_vector);
    transcript.add_element("W_3", g1_vector);
    transcript.add_element("W_4", g1_vector);
    transcript.apply_fiat_shamir("eta");
    transcript.add_element("S", g1_vector);
    transcript.apply_fiat_shamir("beta");
    transcript.add_element("Z_PERM", g1_vector);
    transcript.apply_fiat_shamir("alpha");
    return transcript;
}
} // namespace waffle