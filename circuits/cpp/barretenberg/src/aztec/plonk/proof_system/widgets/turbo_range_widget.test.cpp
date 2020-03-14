#include "create_dummy_transcript.hpp"
#include "turbo_range_widget.hpp"
#include <gtest/gtest.h>
#include <polynomials/polynomial_arithmetic.hpp>

using namespace barretenberg;
using namespace waffle;

namespace {
waffle::ProverTurboRangeWidget create_test_widget_circuit(const size_t num_gates,
                                                          std::shared_ptr<program_witness> witness,
                                                          std::shared_ptr<proving_key> key,
                                                          bool use_coset_fft = false)
{
    polynomial w_1(num_gates);
    polynomial w_2(num_gates);
    polynomial w_3(num_gates);
    polynomial w_4(num_gates);

    polynomial q_range(num_gates);

    fr four = (fr::one() + fr::one() + fr::one() + fr::one());

    std::array<fr, 4> values{ fr::zero(), fr::one(), fr::one() + fr::one(), (fr::one() + fr::one() + fr::one()) };

    w_4[0] = fr::zero();
    for (size_t i = 0; i < num_gates - 1; ++i) {
        w_3[i] = w_4[i] * four + values[i & 3];
        w_2[i] = w_3[i] * four + values[(i + 1) & 3];
        w_1[i] = w_2[i] * four + values[(i + 2) & 3];
        w_4[i + 1] = w_1[i] * four + values[(i + 3) & 3];

        q_range[i] = fr::one();
    }
    w_1[num_gates - 1] = fr::zero();
    w_2[num_gates - 1] = fr::zero();
    w_3[num_gates - 1] = fr::zero();
    q_range[num_gates - 1] = fr::zero();

    polynomial& w_1_fft = key->wire_ffts.at("w_1_fft");
    polynomial& w_2_fft = key->wire_ffts.at("w_2_fft");
    polynomial& w_3_fft = key->wire_ffts.at("w_3_fft");
    polynomial& w_4_fft = key->wire_ffts.at("w_4_fft");

    w_1_fft = polynomial(w_1, 4 * num_gates + 4);
    w_2_fft = polynomial(w_2, 4 * num_gates + 4);
    w_3_fft = polynomial(w_3, 4 * num_gates + 4);
    w_4_fft = polynomial(w_4, 4 * num_gates + 4);

    w_1.ifft(key->small_domain);
    w_2.ifft(key->small_domain);
    w_3.ifft(key->small_domain);
    w_4.ifft(key->small_domain);

    w_1_fft.ifft(key->small_domain);
    w_2_fft.ifft(key->small_domain);
    w_3_fft.ifft(key->small_domain);
    w_4_fft.ifft(key->small_domain);

    if (use_coset_fft) {
        w_1_fft.coset_fft(key->large_domain);
        w_2_fft.coset_fft(key->large_domain);
        w_3_fft.coset_fft(key->large_domain);
        w_4_fft.coset_fft(key->large_domain);
    } else {
        w_1_fft.fft(key->large_domain);
        w_2_fft.fft(key->large_domain);
        w_3_fft.fft(key->large_domain);
        w_4_fft.fft(key->large_domain);
    }

    w_1_fft.add_lagrange_base_coefficient(w_1_fft[0]);
    w_1_fft.add_lagrange_base_coefficient(w_1_fft[1]);
    w_1_fft.add_lagrange_base_coefficient(w_1_fft[2]);
    w_1_fft.add_lagrange_base_coefficient(w_1_fft[3]);
    w_2_fft.add_lagrange_base_coefficient(w_2_fft[0]);
    w_2_fft.add_lagrange_base_coefficient(w_2_fft[1]);
    w_2_fft.add_lagrange_base_coefficient(w_2_fft[2]);
    w_2_fft.add_lagrange_base_coefficient(w_2_fft[3]);
    w_3_fft.add_lagrange_base_coefficient(w_3_fft[0]);
    w_3_fft.add_lagrange_base_coefficient(w_3_fft[1]);
    w_3_fft.add_lagrange_base_coefficient(w_3_fft[2]);
    w_3_fft.add_lagrange_base_coefficient(w_3_fft[3]);
    w_4_fft.add_lagrange_base_coefficient(w_4_fft[0]);
    w_4_fft.add_lagrange_base_coefficient(w_4_fft[1]);
    w_4_fft.add_lagrange_base_coefficient(w_4_fft[2]);
    w_4_fft.add_lagrange_base_coefficient(w_4_fft[3]);

    witness->wires.insert({ "w_1", std::move(w_1) });
    witness->wires.insert({ "w_2", std::move(w_2) });
    witness->wires.insert({ "w_3", std::move(w_3) });
    witness->wires.insert({ "w_4", std::move(w_4) });

    polynomial q_range_fft(q_range, 4 * num_gates);

    q_range_fft.ifft(key->small_domain);
    q_range.ifft(key->small_domain);

    if (use_coset_fft) {
        q_range_fft.coset_fft(key->large_domain);
    } else {
        q_range_fft.fft(key->large_domain);
    }
    key->constraint_selectors.insert({ "q_range", std::move(q_range) });
    key->constraint_selector_ffts.insert({ "q_range_fft", std::move(q_range_fft) });

    key->quotient_large = polynomial(num_gates * 4);
    for (size_t i = 0; i < num_gates * 4; ++i) {
        key->quotient_large[i] = fr::zero();
    }
    waffle::ProverTurboRangeWidget widget(key.get(), witness.get());
    return widget;
}
} // namespace

TEST(turbo_range_widget, quotient_polynomial_satisfiability)
{
    const size_t num_gates = 64;
    std::shared_ptr<program_witness> witness = std::make_shared<program_witness>();
    std::shared_ptr<proving_key> key = std::make_shared<proving_key>(num_gates, 0, BARRETENBERG_SRS_PATH);

    waffle::ProverTurboRangeWidget widget = create_test_widget_circuit(num_gates, witness, key);

    transcript::Transcript transcript = create_dummy_standard_transcript();

    widget.compute_quotient_contribution(fr::one(), transcript);

    for (size_t i = 0; i < num_gates * 4; i += 4) {
        EXPECT_EQ((key->quotient_large[i] == fr::zero()), true);
    }
}

TEST(turbo_range_widget, compute_linear_contribution)
{
    const size_t num_gates = 64;
    std::shared_ptr<program_witness> witness = std::make_shared<program_witness>();
    std::shared_ptr<proving_key> key = std::make_shared<proving_key>(num_gates, 0, BARRETENBERG_SRS_PATH);

    waffle::ProverTurboRangeWidget widget = create_test_widget_circuit(num_gates, witness, key, true);

    transcript::Transcript transcript = create_dummy_standard_transcript();

    widget.compute_quotient_contribution(fr::one(), transcript);

    barretenberg::polynomial_arithmetic::divide_by_pseudo_vanishing_polynomial(
        key->quotient_large.get_coefficients(), key->small_domain, key->large_domain);

    key->quotient_large.coset_ifft(key->large_domain);

    fr z_challenge = fr::random_element();
    fr shifted_z = key->small_domain.root * z_challenge;

    for (size_t i = 0; i < 4; ++i) {
        std::string wire_key = "w_" + std::to_string(i + 1);
        const polynomial& wire = witness->wires.at(wire_key);
        fr wire_eval = wire.evaluate(z_challenge, num_gates);
        transcript.add_element(wire_key, wire_eval.to_buffer());
        fr shifted_wire_eval = wire.evaluate(shifted_z, num_gates);
        transcript.add_element(wire_key + "_omega", shifted_wire_eval.to_buffer());
    }

    for (size_t i = 0; i < num_gates; ++i) {
        key->linear_poly[i] = fr::zero();
    }
    widget.compute_linear_contribution(fr::one(), transcript, key->linear_poly);

    fr quotient_eval = key->quotient_large.evaluate(z_challenge, 4 * num_gates);
    fr result = key->linear_poly.evaluate(z_challenge, num_gates);

    barretenberg::polynomial_arithmetic::lagrange_evaluations lagrange_evals =
        barretenberg::polynomial_arithmetic::get_lagrange_evaluations(z_challenge, key->small_domain);

    fr expected = quotient_eval * lagrange_evals.vanishing_poly;

    EXPECT_EQ((result == expected), true);
}