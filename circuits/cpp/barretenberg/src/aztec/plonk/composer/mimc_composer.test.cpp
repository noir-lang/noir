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

    uint32_t x_in_idx = composer.add_variable(x);
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

    waffle::Verifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
    EXPECT_EQ(result, true);
}
