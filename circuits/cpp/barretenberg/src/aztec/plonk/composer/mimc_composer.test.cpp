#include "mimc_composer.hpp"
#include <gtest/gtest.h>

using namespace barretenberg;

TEST(mimc_composer, test_mimc_gate_proof)
{
    size_t n = 95;
    waffle::MiMCComposer composer = waffle::MiMCComposer(n);
    fr c[n];
    for (size_t i = 0; i < n; ++i) {
        c[i] = fr::random_element();
    }
    fr x = fr::random_element();
    fr k = fr::random_element();

    uint32_t x_in_idx = composer.add_public_variable(x);
    uint32_t k_idx = composer.add_variable(k);
    uint32_t x_out_idx;
    uint32_t x_cubed_idx = 0;
    for (size_t i = 0; i < n; ++i) {
        fr T0 = ((x + k) + c[i]);
        fr x_cubed = T0.sqr();
        x_cubed = x_cubed * T0;
        x_cubed_idx = composer.add_variable(x_cubed);
        fr x_out = x_cubed.sqr();
        x_out = x_out * T0;
        x_out_idx = composer.add_variable(x_out);
        composer.create_mimc_gate({ x_in_idx, x_cubed_idx, k_idx, x_out_idx, c[i] });
        x_in_idx = x_out_idx;
        x = x_out;
    }

    waffle::Prover prover = composer.preprocess();

    waffle::MiMCVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
    EXPECT_EQ(result, true);
}

// TEST(mimc_composer, composer_consistency_check)
// {
//     waffle::StandardComposer standard_composer = waffle::StandardComposer();
//     waffle::MiMCComposer mimc_composer = waffle::MiMCComposer();

//     fr input = fr::random_element();
//     fr k_in = fr::zero();

//     // stdlib::field_t<waffle::StandardComposer> standard_input(
//         // stdlib::public_witness_t<waffle::StandardComposer>(&standard_composer, input));
//     // stdlib::field_t<waffle::StandardComposer> standard_k(
//     //     stdlib::public_witness_t<waffle::StandardComposer>(&standard_composer, k_in));

//     // stdlib::field_t<waffle::MiMCComposer> mimc_input(
//     //     stdlib::public_witness_t<waffle::MiMCComposer>(&mimc_composer, input));
//     // stdlib::field_t<waffle::MiMCComposer> mimc_k(stdlib::public_witness_t<waffle::MiMCComposer>(&mimc_composer, k_in));

//     fr standard_out = mimc_block_cipher(input, k_in);
//     standard_out = standard_out.normalize();

//     // stdlib::field_t<waffle::MiMCComposer> mimc_out = mimc_block_cipher(mimc_input, mimc_k);

//     // EXPECT_EQ((standard_out.get_value() == mimc_out.get_value()), true);

//     // waffle::Prover standard_prover = standard_composer.preprocess();
//     // waffle::Prover mimc_prover = mimc_composer.preprocess();

//     // waffle::Verifier standard_verifier = standard_composer.create_verifier();
//     // waffle::MiMCVerifier mimc_verifier = mimc_composer.create_verifier();

//     // waffle::plonk_proof proofs[2]{ standard_prover.construct_proof(), mimc_prover.construct_proof() };
//     // bool results[2]{ standard_verifier.verify_proof(proofs[0]), mimc_verifier.verify_proof(proofs[1]) };
//     // EXPECT_EQ(results[0], true);
//     // EXPECT_EQ(results[1], true);
// }

