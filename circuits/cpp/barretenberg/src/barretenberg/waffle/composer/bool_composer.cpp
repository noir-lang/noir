#include "./bool_composer.hpp"

#include "../../curves/bn254/scalar_multiplication/scalar_multiplication.hpp"

#include "../../assert.hpp"
#include "../../curves/bn254/fr.hpp"
#include "../proof_system/proving_key/proving_key.hpp"
#include "../proof_system/verification_key/verification_key.hpp"
#include "../proof_system/widgets/arithmetic_widget.hpp"
#include "../proof_system/widgets/bool_widget.hpp"

#include <math.h>

using namespace barretenberg;

namespace waffle {
void BoolComposer::create_add_gate(const add_triple& in)
{
    StandardComposer::create_add_gate(in);
}

void BoolComposer::create_mul_gate(const mul_triple& in)
{
    StandardComposer::create_mul_gate(in);
}

void BoolComposer::create_bool_gate(const uint32_t variable_index)
{
    if (is_bool[variable_index] == false) {
        is_bool[variable_index] = true;
    }
}

void BoolComposer::create_poly_gate(const poly_triple& in)
{
    StandardComposer::create_poly_gate(in);
}

void BoolComposer::create_dummy_gates()
{
    StandardComposer::create_dummy_gates();
    q_left_bools.emplace_back(fr::zero);
    q_right_bools.emplace_back(fr::zero);
    q_left_bools.emplace_back(fr::zero);
    q_right_bools.emplace_back(fr::zero);

    // add a dummy gate to ensure that left / right bool selectors are nonzero
    q_1.emplace_back(fr({ { 0, 0, 0, 0 } }));
    q_2.emplace_back(fr({ { 0, 0, 0, 0 } }));
    q_3.emplace_back(fr({ { 0, 0, 0, 0 } }));
    q_m.emplace_back(fr({ { 0, 0, 0, 0 } }));
    q_c.emplace_back(fr({ { 0, 0, 0, 0 } }));
    q_left_bools.emplace_back(fr::one);
    q_right_bools.emplace_back(fr::one);
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
}

void BoolComposer::process_bool_gates()
{
    q_left_bools.reserve(n);
    q_right_bools.reserve(n);
    q_output_bools.reserve(n);
    for (size_t i = 0; i < n; ++i) {
        q_left_bools.emplace_back(is_bool[w_l[i]] ? fr::one : fr::zero);
        q_right_bools.emplace_back(is_bool[w_r[i]] ? fr::one : fr::zero);
        q_output_bools.emplace_back(is_bool[w_o[i]] ? fr::one : fr::zero);
    }
}

std::shared_ptr<proving_key> BoolComposer::compute_proving_key()
{
    if (computed_proving_key) {
        return circuit_proving_key;
    }

    process_bool_gates();

    ASSERT(wire_epicycles.size() == variables.size());
    ASSERT(n == q_m.size());
    ASSERT(n == q_1.size());
    ASSERT(n == q_2.size());
    ASSERT(n == q_3.size());
    ASSERT(n == q_left_bools.size());
    ASSERT(n == q_right_bools.size());
    ASSERT(n == q_output_bools.size());

    const size_t total_num_gates = n + public_inputs.size();
    size_t log2_n = static_cast<size_t>(log2(total_num_gates + 1));
    if ((1UL << log2_n) != (total_num_gates + 1)) {
        ++log2_n;
    }
    size_t new_n = 1UL << log2_n;
    for (size_t i = total_num_gates; i < new_n; ++i) {
        q_1.emplace_back(fr::zero);
        q_2.emplace_back(fr::zero);
        q_3.emplace_back(fr::zero);
        q_m.emplace_back(fr::zero);
        q_c.emplace_back(fr::zero);
        q_left_bools.emplace_back(fr::zero);
        q_right_bools.emplace_back(fr::zero);
        q_output_bools.emplace_back(fr::zero);
    }
    for (size_t i = 0; i < public_inputs.size(); ++i) {
        epicycle left{ static_cast<uint32_t>(i - public_inputs.size()), WireType::LEFT };
        wire_epicycles[static_cast<size_t>(public_inputs[i])].emplace_back(left);
    }

    circuit_proving_key = std::make_shared<proving_key>(new_n, public_inputs.size(), crs_path);

    polynomial poly_q_m(new_n);
    polynomial poly_q_c(new_n);
    polynomial poly_q_1(new_n);
    polynomial poly_q_2(new_n);
    polynomial poly_q_3(new_n);
    polynomial poly_q_bl(new_n);
    polynomial poly_q_br(new_n);
    polynomial poly_q_bo(new_n);

    for (size_t i = 0; i < public_inputs.size(); ++i) {
        poly_q_m[i] = fr::zero;
        poly_q_1[i] = fr::zero;
        poly_q_2[i] = fr::zero;
        poly_q_3[i] = fr::zero;
        poly_q_c[i] = fr::zero;
        poly_q_bl[i] = fr::zero;
        poly_q_br[i] = fr::zero;
        poly_q_bo[i] = fr::zero;
    }
    for (size_t i = public_inputs.size(); i < new_n; ++i) {
        poly_q_1[i] = q_1[i - public_inputs.size()];
        poly_q_2[i] = q_2[i - public_inputs.size()];
        poly_q_3[i] = q_3[i - public_inputs.size()];
        poly_q_m[i] = q_m[i - public_inputs.size()];
        poly_q_c[i] = q_c[i - public_inputs.size()];
        poly_q_bl[i] = q_left_bools[i - public_inputs.size()];
        poly_q_br[i] = q_right_bools[i - public_inputs.size()];
        poly_q_bo[i] = q_output_bools[i - public_inputs.size()];
    }

    poly_q_1.ifft(circuit_proving_key->small_domain);
    poly_q_2.ifft(circuit_proving_key->small_domain);
    poly_q_3.ifft(circuit_proving_key->small_domain);
    poly_q_m.ifft(circuit_proving_key->small_domain);
    poly_q_c.ifft(circuit_proving_key->small_domain);
    poly_q_bl.ifft(circuit_proving_key->small_domain);
    poly_q_br.ifft(circuit_proving_key->small_domain);
    poly_q_bo.ifft(circuit_proving_key->small_domain);

    polynomial poly_q_1_fft(poly_q_1, new_n * 2);
    polynomial poly_q_2_fft(poly_q_2, new_n * 2);
    polynomial poly_q_3_fft(poly_q_3, new_n * 2);
    polynomial poly_q_m_fft(poly_q_m, new_n * 2);
    polynomial poly_q_c_fft(poly_q_c, new_n * 2);
    polynomial poly_q_bl_fft(poly_q_bl, new_n * 2);
    polynomial poly_q_br_fft(poly_q_br, new_n * 2);
    polynomial poly_q_bo_fft(poly_q_bo, new_n * 2);

    poly_q_1_fft.coset_fft(circuit_proving_key->mid_domain);
    poly_q_2_fft.coset_fft(circuit_proving_key->mid_domain);
    poly_q_3_fft.coset_fft(circuit_proving_key->mid_domain);
    poly_q_m_fft.coset_fft(circuit_proving_key->mid_domain);
    poly_q_c_fft.coset_fft(circuit_proving_key->mid_domain);
    poly_q_bl_fft.coset_fft(circuit_proving_key->mid_domain);
    poly_q_br_fft.coset_fft(circuit_proving_key->mid_domain);
    poly_q_bo_fft.coset_fft(circuit_proving_key->mid_domain);

    circuit_proving_key->constraint_selectors.insert({ "q_m", std::move(poly_q_m) });
    circuit_proving_key->constraint_selectors.insert({ "q_c", std::move(poly_q_c) });
    circuit_proving_key->constraint_selectors.insert({ "q_1", std::move(poly_q_1) });
    circuit_proving_key->constraint_selectors.insert({ "q_2", std::move(poly_q_2) });
    circuit_proving_key->constraint_selectors.insert({ "q_3", std::move(poly_q_3) });
    circuit_proving_key->constraint_selectors.insert({ "q_bl", std::move(poly_q_bl) });
    circuit_proving_key->constraint_selectors.insert({ "q_br", std::move(poly_q_br) });
    circuit_proving_key->constraint_selectors.insert({ "q_bo", std::move(poly_q_bo) });

    circuit_proving_key->constraint_selector_ffts.insert({ "q_m_fft", std::move(poly_q_m_fft) });
    circuit_proving_key->constraint_selector_ffts.insert({ "q_c_fft", std::move(poly_q_c_fft) });
    circuit_proving_key->constraint_selector_ffts.insert({ "q_1_fft", std::move(poly_q_1_fft) });
    circuit_proving_key->constraint_selector_ffts.insert({ "q_2_fft", std::move(poly_q_2_fft) });
    circuit_proving_key->constraint_selector_ffts.insert({ "q_3_fft", std::move(poly_q_3_fft) });
    circuit_proving_key->constraint_selector_ffts.insert({ "q_bl_fft", std::move(poly_q_bl_fft) });
    circuit_proving_key->constraint_selector_ffts.insert({ "q_br_fft", std::move(poly_q_br_fft) });
    circuit_proving_key->constraint_selector_ffts.insert({ "q_bo_fft", std::move(poly_q_bo_fft) });

    compute_sigma_permutations<3>(circuit_proving_key.get());
    computed_proving_key = true;
    return circuit_proving_key;
}

std::shared_ptr<verification_key> BoolComposer::compute_verification_key()
{
    if (computed_verification_key) {
        return circuit_verification_key;
    }
    if (!computed_proving_key) {
        compute_proving_key();
    }

    std::array<fr*, 11> poly_coefficients;
    poly_coefficients[0] = circuit_proving_key->constraint_selectors.at("q_1").get_coefficients();
    poly_coefficients[1] = circuit_proving_key->constraint_selectors.at("q_2").get_coefficients();
    poly_coefficients[2] = circuit_proving_key->constraint_selectors.at("q_3").get_coefficients();
    poly_coefficients[3] = circuit_proving_key->constraint_selectors.at("q_m").get_coefficients();
    poly_coefficients[4] = circuit_proving_key->constraint_selectors.at("q_c").get_coefficients();
    poly_coefficients[5] = circuit_proving_key->constraint_selectors.at("q_bl").get_coefficients();
    poly_coefficients[6] = circuit_proving_key->constraint_selectors.at("q_br").get_coefficients();
    poly_coefficients[7] = circuit_proving_key->constraint_selectors.at("q_bo").get_coefficients();
    poly_coefficients[8] = circuit_proving_key->permutation_selectors.at("sigma_1").get_coefficients();
    poly_coefficients[9] = circuit_proving_key->permutation_selectors.at("sigma_2").get_coefficients();
    poly_coefficients[10] = circuit_proving_key->permutation_selectors.at("sigma_3").get_coefficients();

    std::vector<barretenberg::g1::affine_element> commitments;
    commitments.resize(11);

    for (size_t i = 0; i < 11; ++i) {
        g1::jacobian_to_affine(scalar_multiplication::pippenger(poly_coefficients[i],
                                                                circuit_proving_key->reference_string.monomials,
                                                                circuit_proving_key->n),
                               commitments[i]);
    }

    circuit_verification_key =
        std::make_shared<verification_key>(circuit_proving_key->n, circuit_proving_key->num_public_inputs, crs_path);

    circuit_verification_key->constraint_selectors.insert({ "Q_1", commitments[0] });
    circuit_verification_key->constraint_selectors.insert({ "Q_2", commitments[1] });
    circuit_verification_key->constraint_selectors.insert({ "Q_3", commitments[2] });
    circuit_verification_key->constraint_selectors.insert({ "Q_M", commitments[3] });
    circuit_verification_key->constraint_selectors.insert({ "Q_C", commitments[4] });
    circuit_verification_key->constraint_selectors.insert({ "Q_BL", commitments[5] });
    circuit_verification_key->constraint_selectors.insert({ "Q_BR", commitments[6] });
    circuit_verification_key->constraint_selectors.insert({ "Q_BO", commitments[7] });

    circuit_verification_key->permutation_selectors.insert({ "SIGMA_1", commitments[8] });
    circuit_verification_key->permutation_selectors.insert({ "SIGMA_2", commitments[9] });
    circuit_verification_key->permutation_selectors.insert({ "SIGMA_3", commitments[10] });

    computed_verification_key = true;
    return circuit_verification_key;
}

std::shared_ptr<program_witness> BoolComposer::compute_witness()
{
    if (computed_witness) {
        return witness;
    }
    witness = std::make_shared<program_witness>();

    const size_t total_num_gates = n + public_inputs.size();
    size_t log2_n = static_cast<size_t>(log2(total_num_gates + 1));
    if ((1UL << log2_n) != (total_num_gates + 1)) {
        ++log2_n;
    }
    size_t new_n = 1UL << log2_n;
    for (size_t i = total_num_gates; i < new_n; ++i) {
        w_l.emplace_back(zero_idx);
        w_r.emplace_back(zero_idx);
        w_o.emplace_back(zero_idx);
    }

    polynomial poly_w_1(new_n);
    polynomial poly_w_2(new_n);
    polynomial poly_w_3(new_n);

    for (size_t i = 0; i < public_inputs.size(); ++i) {
        fr::__copy(variables[public_inputs[i]], poly_w_1[i]);
        fr::__copy(fr::zero, poly_w_2[i]);
        fr::__copy(fr::zero, poly_w_3[i]);
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

Verifier BoolComposer::create_verifier()
{
    compute_verification_key();
    Verifier output_state(circuit_verification_key, create_manifest(public_inputs.size()));

    std::unique_ptr<VerifierArithmeticWidget> arithmetic_widget = std::make_unique<VerifierArithmeticWidget>();
    std::unique_ptr<VerifierBoolWidget> bool_widget = std::make_unique<VerifierBoolWidget>();

    output_state.verifier_widgets.emplace_back(std::move(arithmetic_widget));
    output_state.verifier_widgets.emplace_back(std::move(bool_widget));

    return output_state;
}

Prover BoolComposer::preprocess()
{
    compute_proving_key();
    compute_witness();

    Prover output_state(circuit_proving_key, witness, create_manifest(public_inputs.size()));

    std::unique_ptr<ProverBoolWidget> bool_widget =
        std::make_unique<ProverBoolWidget>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverArithmeticWidget> arithmetic_widget =
        std::make_unique<ProverArithmeticWidget>(circuit_proving_key.get(), witness.get());

    output_state.widgets.emplace_back(std::move(arithmetic_widget));
    output_state.widgets.emplace_back(std::move(bool_widget));

    return output_state;
}
} // namespace waffle