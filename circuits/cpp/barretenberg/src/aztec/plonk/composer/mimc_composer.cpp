#include "mimc_composer.hpp"
#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include <numeric/bitop/get_msb.hpp>
#include <plonk/proof_system/widgets/arithmetic_widget.hpp>
#include <plonk/proof_system/widgets/mimc_widget.hpp>

using namespace barretenberg;

namespace waffle {
void MiMCComposer::create_add_gate(const add_triple& in)
{
    if (current_output_wire != static_cast<uint32_t>(-1)) {
        create_noop_gate();
    }
    StandardComposer::create_add_gate(in);
    q_mimc_coefficient.emplace_back(0);
    q_mimc_selector.emplace_back(0);
    current_output_wire = static_cast<uint32_t>(-1);
}

void MiMCComposer::create_mul_gate(const mul_triple& in)
{
    if (current_output_wire != static_cast<uint32_t>(-1)) {
        create_noop_gate();
    }
    StandardComposer::create_mul_gate(in);
    q_mimc_coefficient.emplace_back(0);
    q_mimc_selector.emplace_back(0);
    current_output_wire = static_cast<uint32_t>(-1);
}

void MiMCComposer::create_bool_gate(const uint32_t variable_index)
{
    if (current_output_wire != static_cast<uint32_t>(-1)) {
        create_noop_gate();
    }
    StandardComposer::create_bool_gate(variable_index);
    q_mimc_coefficient.emplace_back(0);
    q_mimc_selector.emplace_back(0);
    current_output_wire = static_cast<uint32_t>(-1);
}

void MiMCComposer::create_poly_gate(const poly_triple& in)
{
    if (current_output_wire != static_cast<uint32_t>(-1)) {
        create_noop_gate();
    }
    StandardComposer::create_poly_gate(in);
    q_mimc_coefficient.emplace_back(0);
    q_mimc_selector.emplace_back(0);
    current_output_wire = static_cast<uint32_t>(-1);
}

void MiMCComposer::create_mimc_gate(const mimc_quadruplet& in)
{
    if ((current_output_wire != static_cast<uint32_t>(-1)) && (in.x_in_idx != current_output_wire)) {
        create_noop_gate();
    }
    w_o.emplace_back(in.x_in_idx);
    w_l.emplace_back(in.k_idx);
    w_r.emplace_back(in.x_cubed_idx);
    current_output_wire = in.x_out_idx;

    q_m.emplace_back(0);
    q_1.emplace_back(0);
    q_2.emplace_back(0);
    q_3.emplace_back(0);
    q_c.emplace_back(0);
    q_mimc_coefficient.emplace_back(in.mimc_constant);
    q_mimc_selector.emplace_back(fr::one());

    epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
    epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
    epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };
    wire_epicycles[static_cast<size_t>(in.k_idx)].emplace_back(left);
    wire_epicycles[static_cast<size_t>(in.x_cubed_idx)].emplace_back(right);
    wire_epicycles[static_cast<size_t>(in.x_in_idx)].emplace_back(out);
    ++n;
}

void MiMCComposer::create_noop_gate()
{
    q_m.emplace_back(0);
    q_1.emplace_back(0);
    q_2.emplace_back(0);
    q_3.emplace_back(0);
    q_c.emplace_back(0);
    q_mimc_coefficient.emplace_back(0);
    q_mimc_selector.emplace_back(0);
    w_l.emplace_back(zero_idx);
    w_r.emplace_back(zero_idx);

    epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
    epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
    epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };
    if (current_output_wire != static_cast<uint32_t>(-1)) {
        w_o.emplace_back(current_output_wire);
        wire_epicycles[static_cast<size_t>(current_output_wire)].emplace_back(out);
        current_output_wire = static_cast<uint32_t>(-1);
    } else {
        w_o.emplace_back(zero_idx);
        wire_epicycles[static_cast<size_t>(zero_idx)].emplace_back(out);
    }
    wire_epicycles[static_cast<size_t>(zero_idx)].emplace_back(left);
    wire_epicycles[static_cast<size_t>(zero_idx)].emplace_back(right);

    ++n;
}

void MiMCComposer::create_dummy_gates()
{
    StandardComposer::create_dummy_gates();
    q_mimc_coefficient.emplace_back(fr::zero());
    q_mimc_selector.emplace_back(fr::zero());
    q_mimc_coefficient.emplace_back(fr::zero());
    q_mimc_selector.emplace_back(fr::zero());

    // add in dummy gates to ensure that all of our polynomials are not zero and not identical
    // TODO: sanitise this :/
    q_m.emplace_back(0);
    q_1.emplace_back(0);
    q_2.emplace_back(0);
    q_3.emplace_back(0);
    q_c.emplace_back(0);
    q_mimc_coefficient.emplace_back(0);
    q_mimc_selector.emplace_back(fr::one());
    w_l.emplace_back(zero_idx);
    w_r.emplace_back(zero_idx);
    w_o.emplace_back(zero_idx);
    epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
    epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
    epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };
    wire_epicycles[static_cast<size_t>(zero_idx)].emplace_back(left);
    wire_epicycles[static_cast<size_t>(zero_idx)].emplace_back(right);
    wire_epicycles[static_cast<size_t>(zero_idx)].emplace_back(out);

    ++n;

    q_m.emplace_back(0);
    q_1.emplace_back(0);
    q_2.emplace_back(0);
    q_3.emplace_back(0);
    q_c.emplace_back(0);
    q_mimc_coefficient.emplace_back(fr::one());
    q_mimc_selector.emplace_back(0);
    w_l.emplace_back(zero_idx);
    w_r.emplace_back(zero_idx);
    w_o.emplace_back(zero_idx);

    // add a permutation on zero_idx to ensure SIGMA_1, SIGMA_2, SIGMA_3 are well formed
    left = { static_cast<uint32_t>(n), WireType::LEFT };
    right = { static_cast<uint32_t>(n), WireType::RIGHT };
    out = { static_cast<uint32_t>(n), WireType::OUTPUT };
    wire_epicycles[static_cast<size_t>(zero_idx)].emplace_back(left);
    wire_epicycles[static_cast<size_t>(zero_idx)].emplace_back(right);
    wire_epicycles[static_cast<size_t>(zero_idx)].emplace_back(out);
    ++n;
}

std::shared_ptr<proving_key> MiMCComposer::compute_proving_key()
{
    if (computed_proving_key) {
        return circuit_proving_key;
    }
    ASSERT(wire_epicycles.size() == variables.size());
    ASSERT(n == q_m.size());
    ASSERT(n == q_1.size());
    ASSERT(n == q_2.size());
    ASSERT(n == q_3.size());
    ASSERT(n == q_mimc_coefficient.size());
    ASSERT(n == q_mimc_selector.size());

    size_t offset = 0;
    if (current_output_wire != static_cast<uint32_t>(-1)) {
        q_m.emplace_back(fr::zero());
        q_1.emplace_back(fr::zero());
        q_2.emplace_back(fr::zero());
        q_3.emplace_back(fr::zero());
        q_c.emplace_back(fr::zero());
        q_mimc_coefficient.emplace_back(fr::zero());
        q_mimc_selector.emplace_back(fr::zero());
        ++offset;
    }

    const size_t total_num_gates = n + offset + public_inputs.size();
    size_t log2_n = static_cast<size_t>(numeric::get_msb(total_num_gates + 1));
    if ((1UL << log2_n) != (total_num_gates + 1)) {
        ++log2_n;
    }
    size_t new_n = 1UL << log2_n;
    for (size_t i = total_num_gates; i < new_n; ++i) {
        q_m.emplace_back(fr::zero());
        q_1.emplace_back(fr::zero());
        q_2.emplace_back(fr::zero());
        q_3.emplace_back(fr::zero());
        q_c.emplace_back(fr::zero());
        q_mimc_coefficient.emplace_back(fr::zero());
        q_mimc_selector.emplace_back(fr::zero());
    }

    circuit_proving_key = std::make_shared<proving_key>(new_n, public_inputs.size(), crs_path);

    polynomial poly_q_m(new_n);
    polynomial poly_q_c(new_n);
    polynomial poly_q_1(new_n);
    polynomial poly_q_2(new_n);
    polynomial poly_q_3(new_n);
    polynomial poly_q_mimc_coefficient(new_n);
    polynomial poly_q_mimc_selector(new_n);

    for (size_t i = 0; i < public_inputs.size(); ++i) {
        epicycle left{ static_cast<uint32_t>(i - public_inputs.size()), WireType::LEFT };
        epicycle right{ static_cast<uint32_t>(i - public_inputs.size()), WireType::RIGHT };

        std::vector<epicycle>& old_epicycles = wire_epicycles[static_cast<size_t>(public_inputs[i])];

        std::vector<epicycle> new_epicycles;

        new_epicycles.emplace_back(left);
        new_epicycles.emplace_back(right);
        for (size_t i = 0; i < old_epicycles.size(); ++i) {
            new_epicycles.emplace_back(old_epicycles[i]);
        }
        old_epicycles = new_epicycles;
    }
    for (size_t i = 0; i < public_inputs.size(); ++i) {
        poly_q_m[i] = fr::zero();
        poly_q_1[i] = fr::one();
        poly_q_2[i] = fr::zero();
        poly_q_3[i] = fr::zero();
        poly_q_c[i] = fr::zero();
        poly_q_mimc_coefficient[i] = fr::zero();
        poly_q_mimc_selector[i] = fr::zero();
    }
    for (size_t i = public_inputs.size(); i < new_n; ++i) {
        poly_q_m[i] = q_m[i - public_inputs.size()];
        poly_q_1[i] = q_1[i - public_inputs.size()];
        poly_q_2[i] = q_2[i - public_inputs.size()];
        poly_q_3[i] = q_3[i - public_inputs.size()];
        poly_q_c[i] = q_c[i - public_inputs.size()];
        poly_q_mimc_coefficient[i] = q_mimc_coefficient[i - public_inputs.size()];
        poly_q_mimc_selector[i] = q_mimc_selector[i - public_inputs.size()];
    }

    poly_q_1.ifft(circuit_proving_key->small_domain);
    poly_q_2.ifft(circuit_proving_key->small_domain);
    poly_q_3.ifft(circuit_proving_key->small_domain);
    poly_q_m.ifft(circuit_proving_key->small_domain);
    poly_q_c.ifft(circuit_proving_key->small_domain);
    poly_q_mimc_coefficient.ifft(circuit_proving_key->small_domain);
    poly_q_mimc_selector.ifft(circuit_proving_key->small_domain);

    polynomial poly_q_1_fft(poly_q_1, new_n * 2);
    polynomial poly_q_2_fft(poly_q_2, new_n * 2);
    polynomial poly_q_3_fft(poly_q_3, new_n * 2);
    polynomial poly_q_mimc_coefficient_fft(poly_q_mimc_coefficient, new_n * 4);
    polynomial poly_q_mimc_selector_fft(poly_q_mimc_selector, new_n * 4);
    polynomial poly_q_m_fft(poly_q_m, new_n * 2);
    polynomial poly_q_c_fft(poly_q_c, new_n * 2);

    poly_q_1_fft.coset_fft(circuit_proving_key->mid_domain);
    poly_q_2_fft.coset_fft(circuit_proving_key->mid_domain);
    poly_q_3_fft.coset_fft(circuit_proving_key->mid_domain);
    poly_q_m_fft.coset_fft(circuit_proving_key->mid_domain);
    poly_q_c_fft.coset_fft(circuit_proving_key->mid_domain);
    poly_q_mimc_coefficient_fft.coset_fft(circuit_proving_key->large_domain);
    poly_q_mimc_selector_fft.coset_fft(circuit_proving_key->large_domain);

    circuit_proving_key->constraint_selectors.insert({ "q_m", std::move(poly_q_m) });
    circuit_proving_key->constraint_selectors.insert({ "q_c", std::move(poly_q_c) });
    circuit_proving_key->constraint_selectors.insert({ "q_mimc_coefficient", std::move(poly_q_mimc_coefficient) });
    circuit_proving_key->constraint_selectors.insert({ "q_mimc_selector", std::move(poly_q_mimc_selector) });
    circuit_proving_key->constraint_selectors.insert({ "q_1", std::move(poly_q_1) });
    circuit_proving_key->constraint_selectors.insert({ "q_2", std::move(poly_q_2) });
    circuit_proving_key->constraint_selectors.insert({ "q_3", std::move(poly_q_3) });

    circuit_proving_key->constraint_selector_ffts.insert({ "q_m_fft", std::move(poly_q_m_fft) });
    circuit_proving_key->constraint_selector_ffts.insert({ "q_c_fft", std::move(poly_q_c_fft) });
    circuit_proving_key->constraint_selector_ffts.insert(
        { "q_mimc_selector_fft", std::move(poly_q_mimc_selector_fft) });
    circuit_proving_key->constraint_selector_ffts.insert(
        { "q_mimc_coefficient_fft", std::move(poly_q_mimc_coefficient_fft) });
    circuit_proving_key->constraint_selector_ffts.insert({ "q_1_fft", std::move(poly_q_1_fft) });
    circuit_proving_key->constraint_selector_ffts.insert({ "q_2_fft", std::move(poly_q_2_fft) });
    circuit_proving_key->constraint_selector_ffts.insert({ "q_3_fft", std::move(poly_q_3_fft) });

    compute_sigma_permutations<3>(circuit_proving_key.get());
    computed_proving_key = true;
    return circuit_proving_key;
}

std::shared_ptr<verification_key> MiMCComposer::compute_verification_key()
{
    if (computed_verification_key) {
        return circuit_verification_key;
    }
    if (!computed_proving_key) {
        compute_proving_key();
    }

    std::array<fr*, 10> poly_coefficients;
    poly_coefficients[0] = circuit_proving_key->constraint_selectors.at("q_1").get_coefficients();
    poly_coefficients[1] = circuit_proving_key->constraint_selectors.at("q_2").get_coefficients();
    poly_coefficients[2] = circuit_proving_key->constraint_selectors.at("q_3").get_coefficients();
    poly_coefficients[3] = circuit_proving_key->constraint_selectors.at("q_m").get_coefficients();
    poly_coefficients[4] = circuit_proving_key->constraint_selectors.at("q_c").get_coefficients();

    poly_coefficients[5] = circuit_proving_key->constraint_selectors.at("q_mimc_coefficient").get_coefficients();
    poly_coefficients[6] = circuit_proving_key->constraint_selectors.at("q_mimc_selector").get_coefficients();
    poly_coefficients[7] = circuit_proving_key->permutation_selectors.at("sigma_1").get_coefficients();
    poly_coefficients[8] = circuit_proving_key->permutation_selectors.at("sigma_2").get_coefficients();
    poly_coefficients[9] = circuit_proving_key->permutation_selectors.at("sigma_3").get_coefficients();

    std::vector<barretenberg::g1::affine_element> commitments;
    scalar_multiplication::pippenger_runtime_state state(circuit_proving_key->n);

    commitments.resize(10);

    for (size_t i = 0; i < 10; ++i) {
        commitments[i] = g1::affine_element(scalar_multiplication::pippenger(
            poly_coefficients[i], circuit_proving_key->reference_string.monomials, circuit_proving_key->n, state));
    }

    circuit_verification_key =
        std::make_shared<verification_key>(circuit_proving_key->n, circuit_proving_key->num_public_inputs, crs_path);

    circuit_verification_key->constraint_selectors.insert({ "Q_1", commitments[0] });
    circuit_verification_key->constraint_selectors.insert({ "Q_2", commitments[1] });
    circuit_verification_key->constraint_selectors.insert({ "Q_3", commitments[2] });
    circuit_verification_key->constraint_selectors.insert({ "Q_M", commitments[3] });
    circuit_verification_key->constraint_selectors.insert({ "Q_C", commitments[4] });
    circuit_verification_key->constraint_selectors.insert({ "Q_MIMC_COEFFICIENT", commitments[5] });
    circuit_verification_key->constraint_selectors.insert({ "Q_MIMC_SELECTOR", commitments[6] });

    circuit_verification_key->permutation_selectors.insert({ "SIGMA_1", commitments[7] });
    circuit_verification_key->permutation_selectors.insert({ "SIGMA_2", commitments[8] });
    circuit_verification_key->permutation_selectors.insert({ "SIGMA_3", commitments[9] });

    computed_verification_key = true;
    return circuit_verification_key;
}

std::shared_ptr<program_witness> MiMCComposer::compute_witness()
{
    if (computed_witness) {
        return witness;
    }
    size_t offset = 0;
    if (current_output_wire != static_cast<uint32_t>(-1)) {
        w_o.emplace_back(current_output_wire);
        w_l.emplace_back(zero_idx);
        w_r.emplace_back(zero_idx);
        epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };
        wire_epicycles[static_cast<size_t>(current_output_wire)].emplace_back(out);
        ++offset;
    }

    witness = std::make_shared<program_witness>();

    const size_t total_num_gates = n + public_inputs.size() + offset;
    size_t log2_n = static_cast<size_t>(numeric::get_msb(total_num_gates + 1));
    if ((1UL << log2_n) != (total_num_gates + 1)) {
        ++log2_n;
    }
    size_t new_n = 1UL << log2_n;
    for (size_t i = (n + public_inputs.size()); i < new_n; ++i) {
        w_l.emplace_back(zero_idx);
        w_r.emplace_back(zero_idx);
        w_o.emplace_back(zero_idx);
    }
    polynomial poly_w_1(new_n);
    polynomial poly_w_2(new_n);
    polynomial poly_w_3(new_n);
    for (size_t i = 0; i < public_inputs.size(); ++i) {
        fr::__copy(fr::zero(), poly_w_1[i]);
        fr::__copy(variables[public_inputs[i]], poly_w_2[i]);
        fr::__copy(fr::zero(), poly_w_3[i]);
    }
    for (size_t i = public_inputs.size(); i < new_n; ++i) {
        fr::__copy(variables[w_l[i - public_inputs.size()]], poly_w_1.at(i));
        fr::__copy(variables[w_r[i - public_inputs.size()]], poly_w_2.at(i));
        fr::__copy(variables[w_o[i - public_inputs.size()]], poly_w_3.at(i));
    }
    witness->wires.insert({ "w_1", std::move(poly_w_1) });
    witness->wires.insert({ "w_2", std::move(poly_w_2) });
    witness->wires.insert({ "w_3", std::move(poly_w_3) });
    computed_witness = true;
    return witness;
}

Prover MiMCComposer::preprocess()
{
    compute_proving_key();
    compute_witness();
    Prover output_state(circuit_proving_key, witness, create_manifest(public_inputs.size()));

    std::unique_ptr<ProverMiMCWidget> mimc_widget =
        std::make_unique<ProverMiMCWidget>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverArithmeticWidget> arithmetic_widget =
        std::make_unique<ProverArithmeticWidget>(circuit_proving_key.get(), witness.get());

    output_state.widgets.emplace_back(std::move(arithmetic_widget));
    output_state.widgets.emplace_back(std::move(mimc_widget));
    return output_state;
}

MiMCVerifier MiMCComposer::create_verifier()
{
    compute_verification_key();

    MiMCVerifier output_state(circuit_verification_key, create_manifest(public_inputs.size()));

    return output_state;
}
} // namespace waffle