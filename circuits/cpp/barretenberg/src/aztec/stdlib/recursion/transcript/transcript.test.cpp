#include "transcript.hpp"
#include <gtest/gtest.h>

#include <ecc/curves/bn254/fr.hpp>
#include <ecc/curves/bn254/g1.hpp>

#include <plonk/transcript/transcript.hpp>
#include <stdlib/types/turbo.hpp>

using namespace plonk;

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
          transcript::Manifest::RoundManifest({ { "Z", g1_size, false } }, "alpha", 1),
          transcript::Manifest::RoundManifest(
              { { "T_1", g1_size, false }, { "T_2", g1_size, false }, { "T_3", g1_size, false } }, "z", 1),
          transcript::Manifest::RoundManifest({ { "w_1", fr_size, false },
                                                { "w_2", fr_size, false },
                                                { "w_3", fr_size, false },
                                                { "w_3_omega", fr_size, false },
                                                { "z_omega", fr_size, false },
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

struct TestData {
    std::vector<barretenberg::g1::affine_element> g1_elements;
    std::vector<barretenberg::fr> fr_elements;
};

TestData get_test_data()
{
    TestData data;
    for (size_t i = 0; i < 32; ++i) {
        data.g1_elements.push_back(barretenberg::g1::affine_element(barretenberg::g1::element::random_element()));
        data.fr_elements.push_back(barretenberg::fr::random_element());
    }
    return data;
}

transcript::Transcript get_test_base_transcript(const TestData& data)
{
    transcript::Transcript transcript = transcript::Transcript(create_manifest(0));
    transcript.add_element("circuit_size", { 1, 2, 3, 4 });
    transcript.add_element("public_input_size", { 0, 0, 0, 0 });
    transcript.apply_fiat_shamir("init");

    transcript.add_element("public_inputs", {});

    transcript.add_element("W_1", data.g1_elements[0].to_buffer());
    transcript.add_element("W_2", data.g1_elements[1].to_buffer());
    transcript.add_element("W_3", data.g1_elements[2].to_buffer());

    transcript.apply_fiat_shamir("beta");

    transcript.add_element("Z", data.g1_elements[3].to_buffer());

    transcript.apply_fiat_shamir("alpha");

    transcript.add_element("T_1", data.g1_elements[4].to_buffer());
    transcript.add_element("T_2", data.g1_elements[5].to_buffer());
    transcript.add_element("T_3", data.g1_elements[6].to_buffer());

    transcript.apply_fiat_shamir("z");

    transcript.add_element("w_1", data.fr_elements[0].to_buffer());
    transcript.add_element("w_2", data.fr_elements[1].to_buffer());
    transcript.add_element("w_3", data.fr_elements[2].to_buffer());
    transcript.add_element("w_3_omega", data.fr_elements[3].to_buffer());
    transcript.add_element("z_omega", data.fr_elements[4].to_buffer());
    transcript.add_element("sigma_1", data.fr_elements[5].to_buffer());
    transcript.add_element("sigma_2", data.fr_elements[6].to_buffer());
    transcript.add_element("r", data.fr_elements[7].to_buffer());
    transcript.add_element("t", data.fr_elements[8].to_buffer());

    transcript.apply_fiat_shamir("nu");

    transcript.add_element("PI_Z", data.g1_elements[7].to_buffer());
    transcript.add_element("PI_Z_OMEGA", data.g1_elements[8].to_buffer());

    transcript.apply_fiat_shamir("separator");
    return transcript;
}

plonk::stdlib::recursion::Transcript<waffle::TurboComposer> get_circuit_transcript(waffle::TurboComposer* context,
                                                                                   const TestData& data)
{
    plonk::stdlib::recursion::Transcript<waffle::TurboComposer> transcript(context, create_manifest(0));
    uint256_t circuit_size_value = uint256_t(1) + (uint256_t(2) << 8) + (uint256_t(3) << 8) + (uint256_t(4) << 8);
    stdlib::field_t<waffle::TurboComposer> circuit_size(
        stdlib::witness_t(context, barretenberg::fr(circuit_size_value)));
    stdlib::field_t<waffle::TurboComposer> public_input_size(stdlib::witness_t(context, barretenberg::fr(0)));

    transcript.add_field_element("circuit_size", circuit_size);
    transcript.add_field_element("public_input_size", public_input_size);
    transcript.apply_fiat_shamir("init");

    std::vector<stdlib::field_t<waffle::TurboComposer>> public_inputs;
    transcript.add_field_element_vector("public_inputs", public_inputs);

    transcript.add_group_element(
        "W_1", plonk::stdlib::recursion::Transcript<waffle::TurboComposer>::convert_g1(context, data.g1_elements[0]));
    transcript.add_group_element(
        "W_2", plonk::stdlib::recursion::Transcript<waffle::TurboComposer>::convert_g1(context, data.g1_elements[1]));
    transcript.add_group_element(
        "W_3", plonk::stdlib::recursion::Transcript<waffle::TurboComposer>::convert_g1(context, data.g1_elements[2]));

    transcript.apply_fiat_shamir("beta");

    transcript.add_group_element(
        "Z", plonk::stdlib::recursion::Transcript<waffle::TurboComposer>::convert_g1(context, data.g1_elements[3]));

    transcript.apply_fiat_shamir("alpha");

    transcript.add_group_element(
        "T_1", plonk::stdlib::recursion::Transcript<waffle::TurboComposer>::convert_g1(context, data.g1_elements[4]));
    transcript.add_group_element(
        "T_2", plonk::stdlib::recursion::Transcript<waffle::TurboComposer>::convert_g1(context, data.g1_elements[5]));
    transcript.add_group_element(
        "T_3", plonk::stdlib::recursion::Transcript<waffle::TurboComposer>::convert_g1(context, data.g1_elements[6]));

    transcript.apply_fiat_shamir("z");

    transcript.add_field_element(
        "w_1",
        stdlib::field_t<waffle::TurboComposer>(stdlib::witness_t<waffle::TurboComposer>(context, data.fr_elements[0])));
    transcript.add_field_element(
        "w_2",
        stdlib::field_t<waffle::TurboComposer>(stdlib::witness_t<waffle::TurboComposer>(context, data.fr_elements[1])));
    transcript.add_field_element(
        "w_3",
        stdlib::field_t<waffle::TurboComposer>(stdlib::witness_t<waffle::TurboComposer>(context, data.fr_elements[2])));
    transcript.add_field_element(
        "w_3_omega",
        stdlib::field_t<waffle::TurboComposer>(stdlib::witness_t<waffle::TurboComposer>(context, data.fr_elements[3])));
    transcript.add_field_element(
        "z_omega",
        stdlib::field_t<waffle::TurboComposer>(stdlib::witness_t<waffle::TurboComposer>(context, data.fr_elements[4])));
    transcript.add_field_element(
        "sigma_1",
        stdlib::field_t<waffle::TurboComposer>(stdlib::witness_t<waffle::TurboComposer>(context, data.fr_elements[5])));
    transcript.add_field_element(
        "sigma_2",
        stdlib::field_t<waffle::TurboComposer>(stdlib::witness_t<waffle::TurboComposer>(context, data.fr_elements[6])));
    transcript.add_field_element(
        "r",
        stdlib::field_t<waffle::TurboComposer>(stdlib::witness_t<waffle::TurboComposer>(context, data.fr_elements[7])));
    transcript.add_field_element(
        "t",
        stdlib::field_t<waffle::TurboComposer>(stdlib::witness_t<waffle::TurboComposer>(context, data.fr_elements[8])));

    transcript.apply_fiat_shamir("nu");

    transcript.add_group_element(
        "PI_Z", plonk::stdlib::recursion::Transcript<waffle::TurboComposer>::convert_g1(context, data.g1_elements[7]));
    transcript.add_group_element(
        "PI_Z_OMEGA",
        plonk::stdlib::recursion::Transcript<waffle::TurboComposer>::convert_g1(context, data.g1_elements[8]));

    transcript.apply_fiat_shamir("separator");
    return transcript;
}

TEST(stdlib_transcript, validate_transcript)
{
    TestData data = get_test_data();
    transcript::Transcript normal_transcript = get_test_base_transcript(data);

    waffle::TurboComposer composer = waffle::TurboComposer();

    get_circuit_transcript(&composer, data);

    printf("composer gates = %zu\n", composer.get_num_gates());
    waffle::TurboProver prover = composer.create_prover();

    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}