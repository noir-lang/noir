#include "generalized_permutation_composer.hpp"

#include <algorithm>

#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include <numeric/bitop/get_msb.hpp>
#include <plonk/proof_system/widgets/random_widgets/permutation_widget.hpp>
#include <plonk/proof_system/widgets/transition_widgets/turbo_arithmetic_widget.hpp>
#include <plonk/proof_system/widgets/transition_widgets/turbo_fixed_base_widget.hpp>
#include <plonk/proof_system/widgets/transition_widgets/turbo_logic_widget.hpp>
#include <plonk/proof_system/widgets/transition_widgets/turbo_range_widget.hpp>
#include <plonk/proof_system/widgets/transition_widgets/genperm_sort_widget.hpp>
#include <plonk/reference_string/file_reference_string.hpp>
#include <plonk/proof_system/utils/permutation.hpp>

using namespace barretenberg;

namespace waffle {

#define TURBO_SELECTOR_REFS                                                                                            \
    auto& q_m = selectors[TurboSelectors::QM];                                                                         \
    auto& q_c = selectors[TurboSelectors::QC];                                                                         \
    auto& q_1 = selectors[TurboSelectors::Q1];                                                                         \
    auto& q_2 = selectors[TurboSelectors::Q2];                                                                         \
    auto& q_3 = selectors[TurboSelectors::Q3];                                                                         \
    auto& q_4 = selectors[TurboSelectors::Q4];                                                                         \
    auto& q_5 = selectors[TurboSelectors::Q5];                                                                         \
    auto& q_arith = selectors[TurboSelectors::QARITH];                                                                 \
    auto& q_ecc_1 = selectors[TurboSelectors::QECC_1];                                                                 \
    auto& q_range = selectors[TurboSelectors::QRANGE];                                                                 \
    auto& q_logic = selectors[TurboSelectors::QLOGIC];

GenPermComposer::GenPermComposer()
    : TurboComposer()
{
    tau.insert({ DUMMY_TAG, DUMMY_TAG });
};

GenPermComposer::RangeList GenPermComposer::create_range_list(const uint64_t target_range)
{
    RangeList result;
    const auto range_tag = get_new_tag(); // current_tag + 1;
    const auto tau_tag = get_new_tag();   // current_tag + 2;
    create_tag(range_tag, tau_tag);
    create_tag(tau_tag, range_tag);
    result.target_range = target_range;
    result.range_tag = range_tag;
    result.tau_tag = tau_tag;

    const uint64_t num_multiples_of_three = (target_range / 3);

    result.variable_indices.reserve(num_multiples_of_three);
    for (uint64_t i = 0; i <= num_multiples_of_three; ++i) {
        const uint32_t index = add_variable(i * 3);
        result.variable_indices.emplace_back(index);
        assign_tag(index, result.range_tag);
    }
    {
        const uint32_t index = add_variable(target_range);
        result.variable_indices.emplace_back(index);
        assign_tag(index, result.range_tag);
    }
    // Need this because these variables will not appear in the witness otherwise
    create_dummy_constraint(result.variable_indices);

    return result;
}

void GenPermComposer::create_range_constraint(const uint32_t variable_index, const uint64_t target_range)
{
    if (range_lists.count(target_range) == 0) {
        range_lists.insert({ target_range, create_range_list(target_range) });
    }

    auto& list = range_lists[target_range];
    assign_tag(variable_index, list.range_tag);
    list.variable_indices.emplace_back(variable_index);
}
void GenPermComposer::process_range_list(const RangeList& list)
{
    // go over variables
    // for each variable, create mirror variable with same value - with tau tag
    // need to make sure that, in original list, increments of at most 3
    std::vector<uint64_t> sorted_list;
    sorted_list.reserve(list.variable_indices.size());
    for (const auto variable_index : list.variable_indices) {
        const auto& field_element = get_variable(variable_index);
        const uint64_t shrinked_value = field_element.from_montgomery_form().data[0];
        sorted_list.emplace_back(shrinked_value);
    }
    std::sort(sorted_list.begin(), sorted_list.end());
    std::vector<uint32_t> indices;

    // list must be padded to a multipe of 4
    const size_t padding = 4 - (list.variable_indices.size() % 4) % 4; // TODO: this 4 maybe tied to program_width
    for (size_t i = 0; i < padding; ++i) {
        indices.emplace_back(zero_idx);
    }
    for (const auto sorted_value : sorted_list) {
        const uint32_t index = add_variable(sorted_value);
        assign_tag(index, list.tau_tag);
        indices.emplace_back(index);
    }
    create_sort_constraint_with_edges(indices, 0, list.target_range);
}
void GenPermComposer::process_range_lists()
{
    for (const auto& i : range_lists)
        process_range_list(i.second);
}
/*
 Create range constraint:
  * add variable index to a list of range constrained variables
  * data structures: vector of lists, each list contains:
  *    - the range size
  *    - the list of variables in the range
  *    - a generalised permutation tag
  *
  * create range constraint parameters: variable index && range size
  *
  * std::map<uint64_t, RangeList> range_lists;
*/
// Check for a sequence of variables that neighboring differences are at most 3 (used for batched range checkj)
void GenPermComposer::create_sort_constraint(const std::vector<uint32_t> variable_index)
{
    TURBO_SELECTOR_REFS
    ASSERT(variable_index.size() % 4 == 0);
    for (size_t i = 0; i < variable_index.size(); i++) {
        ASSERT(static_cast<uint32_t>(variables.size()) > variable_index[i]);
    }

    for (size_t i = 0; i < variable_index.size(); i += 4) {
        w_l.emplace_back(variable_index[i]);
        w_r.emplace_back(variable_index[i + 1]);
        w_o.emplace_back(variable_index[i + 2]);
        w_4.emplace_back(variable_index[i + 3]);
        ++n;
        q_m.emplace_back(fr::zero());
        q_1.emplace_back(fr::zero());
        q_2.emplace_back(fr::zero());
        q_3.emplace_back(fr::zero());
        q_c.emplace_back(fr::zero());
        q_arith.emplace_back(fr::zero());
        q_4.emplace_back(fr::zero());
        q_5.emplace_back(fr::zero());
        q_ecc_1.emplace_back(fr::zero());
        q_logic.emplace_back(fr::zero());
        q_range.emplace_back(fr::one());
    }
    // dummy gate needed because of sort widget's check of next row
    w_l.emplace_back(variable_index[variable_index.size() - 1]);
    w_r.emplace_back(zero_idx);
    w_o.emplace_back(zero_idx);
    w_4.emplace_back(zero_idx);
    ++n;
    q_m.emplace_back(fr::zero());
    q_1.emplace_back(fr::zero());
    q_2.emplace_back(fr::zero());
    q_3.emplace_back(fr::zero());
    q_c.emplace_back(fr::zero());
    q_arith.emplace_back(fr::zero());
    q_4.emplace_back(fr::zero());
    q_5.emplace_back(fr::zero());
    q_ecc_1.emplace_back(fr::zero());
    q_logic.emplace_back(fr::zero());
    q_range.emplace_back(fr::zero());
}
// useful to put variables in the witness that aren't already used - e.g. the dummy variables of the range constraint in
// multiples of three
void GenPermComposer::create_dummy_constraint(const std::vector<uint32_t> variable_index)
{
    TURBO_SELECTOR_REFS
    ASSERT(variable_index.size() % 4 == 0);
    for (size_t i = 0; i < variable_index.size(); i++) {
        ASSERT(static_cast<uint32_t>(variables.size()) > variable_index[i]);
    }

    for (size_t i = 0; i < variable_index.size(); i += 4) {
        w_l.emplace_back(variable_index[i]);
        w_r.emplace_back(variable_index[i + 1]);
        w_o.emplace_back(variable_index[i + 2]);
        w_4.emplace_back(variable_index[i + 3]);
        ++n;
        q_m.emplace_back(fr::zero());
        q_1.emplace_back(fr::zero());
        q_2.emplace_back(fr::zero());
        q_3.emplace_back(fr::zero());
        q_c.emplace_back(fr::zero());
        q_arith.emplace_back(fr::zero());
        q_4.emplace_back(fr::zero());
        q_5.emplace_back(fr::zero());
        q_ecc_1.emplace_back(fr::zero());
        q_logic.emplace_back(fr::zero());
        q_range.emplace_back(fr::zero());
    }
}
// Check for a sequence of variables that neighboring differences are at most 3 (used for batched range checkj)
void GenPermComposer::create_sort_constraint_with_edges(const std::vector<uint32_t> variable_index,
                                                        const fr start,
                                                        const fr end)
{
    TURBO_SELECTOR_REFS
    // Convenient to assume size is at least 8 for separate gates for start and end conditions
    ASSERT(variable_index.size() % 4 == 0 && variable_index.size() > 4);
    for (size_t i = 0; i < variable_index.size(); i++) {
        ASSERT(static_cast<uint32_t>(variables.size()) > variable_index[i]);
    }
    // enforce range checks of first row and starting at start
    w_l.emplace_back(variable_index[0]);
    w_r.emplace_back(variable_index[1]);
    w_o.emplace_back(variable_index[2]);
    w_4.emplace_back(variable_index[3]);
    ++n;
    q_m.emplace_back(fr::zero());
    q_1.emplace_back(fr::one());
    q_2.emplace_back(fr::zero());
    q_3.emplace_back(fr::zero());
    q_c.emplace_back(-start);
    q_arith.emplace_back(fr::one());
    q_4.emplace_back(fr::zero());
    q_5.emplace_back(fr::zero());
    q_ecc_1.emplace_back(fr::zero());
    q_logic.emplace_back(fr::zero());
    q_range.emplace_back(fr::one());
    // enforce range check for middle rows
    for (size_t i = 4; i < variable_index.size() - 4; i += 4) {
        w_l.emplace_back(variable_index[i]);
        w_r.emplace_back(variable_index[i + 1]);
        w_o.emplace_back(variable_index[i + 2]);
        w_4.emplace_back(variable_index[i + 3]);
        ++n;
        q_m.emplace_back(fr::zero());
        q_1.emplace_back(fr::zero());
        q_2.emplace_back(fr::zero());
        q_3.emplace_back(fr::zero());
        q_c.emplace_back(fr::zero());
        q_arith.emplace_back(fr::zero());
        q_4.emplace_back(fr::zero());
        q_5.emplace_back(fr::zero());
        q_ecc_1.emplace_back(fr::zero());
        q_logic.emplace_back(fr::zero());
        q_range.emplace_back(fr::one());
    }
    // enforce range checks of last row and ending at end
    w_l.emplace_back(variable_index[variable_index.size() - 4]);
    w_r.emplace_back(variable_index[variable_index.size() - 3]);
    w_o.emplace_back(variable_index[variable_index.size() - 2]);
    w_4.emplace_back(variable_index[variable_index.size() - 1]);
    ++n;
    q_m.emplace_back(fr::zero());
    q_1.emplace_back(fr::zero());
    q_2.emplace_back(fr::zero());
    q_3.emplace_back(fr::zero());
    q_c.emplace_back(-end);
    q_arith.emplace_back(fr::one());
    q_4.emplace_back(fr::one());
    q_5.emplace_back(fr::zero());
    q_ecc_1.emplace_back(fr::zero());
    q_logic.emplace_back(fr::zero());
    q_range.emplace_back(fr::one());
    // dummy gate needed because of sort widget's check of next row
    w_l.emplace_back(variable_index[variable_index.size() - 1]);
    w_r.emplace_back(zero_idx);
    w_o.emplace_back(zero_idx);
    w_4.emplace_back(zero_idx);
    ++n;
    q_m.emplace_back(fr::zero());
    q_1.emplace_back(fr::zero());
    q_2.emplace_back(fr::zero());
    q_3.emplace_back(fr::zero());
    q_c.emplace_back(fr::zero());
    q_arith.emplace_back(fr::zero());
    q_4.emplace_back(fr::zero());
    q_5.emplace_back(fr::zero());
    q_ecc_1.emplace_back(fr::zero());
    q_logic.emplace_back(fr::zero());
    q_range.emplace_back(fr::zero());
}

std::shared_ptr<proving_key> GenPermComposer::compute_proving_key()
{

    if (circuit_proving_key) {
        return circuit_proving_key;
    }
    ComposerBase::compute_proving_key_base();
    compute_sigma_permutations<4, true>(circuit_proving_key.get());

    std::copy(genperm_polynomial_manifest,
              genperm_polynomial_manifest + 24,
              std::back_inserter(circuit_proving_key->polynomial_manifest));

    return circuit_proving_key;
}

std::shared_ptr<verification_key> GenPermComposer::compute_verification_key()
{
    if (circuit_verification_key) {
        return circuit_verification_key;
    }
    if (!circuit_proving_key) {
        compute_proving_key();
    }
    std::array<fr*, 19> poly_coefficients;
    poly_coefficients[0] = circuit_proving_key->constraint_selectors.at("q_1").get_coefficients();
    poly_coefficients[1] = circuit_proving_key->constraint_selectors.at("q_2").get_coefficients();
    poly_coefficients[2] = circuit_proving_key->constraint_selectors.at("q_3").get_coefficients();
    poly_coefficients[3] = circuit_proving_key->constraint_selectors.at("q_4").get_coefficients();
    poly_coefficients[4] = circuit_proving_key->constraint_selectors.at("q_5").get_coefficients();
    poly_coefficients[5] = circuit_proving_key->constraint_selectors.at("q_m").get_coefficients();
    poly_coefficients[6] = circuit_proving_key->constraint_selectors.at("q_c").get_coefficients();
    poly_coefficients[7] = circuit_proving_key->constraint_selectors.at("q_arith").get_coefficients();
    poly_coefficients[8] = circuit_proving_key->constraint_selectors.at("q_ecc_1").get_coefficients();
    poly_coefficients[9] = circuit_proving_key->constraint_selectors.at("q_range").get_coefficients();
    poly_coefficients[10] = circuit_proving_key->constraint_selectors.at("q_logic").get_coefficients();

    poly_coefficients[11] = circuit_proving_key->permutation_selectors.at("sigma_1").get_coefficients();
    poly_coefficients[12] = circuit_proving_key->permutation_selectors.at("sigma_2").get_coefficients();
    poly_coefficients[13] = circuit_proving_key->permutation_selectors.at("sigma_3").get_coefficients();
    poly_coefficients[14] = circuit_proving_key->permutation_selectors.at("sigma_4").get_coefficients();

    poly_coefficients[15] = circuit_proving_key->permutation_selectors.at("id_1").get_coefficients();
    poly_coefficients[16] = circuit_proving_key->permutation_selectors.at("id_2").get_coefficients();
    poly_coefficients[17] = circuit_proving_key->permutation_selectors.at("id_3").get_coefficients();
    poly_coefficients[18] = circuit_proving_key->permutation_selectors.at("id_4").get_coefficients();

    scalar_multiplication::pippenger_runtime_state state(circuit_proving_key->n);
    std::vector<barretenberg::g1::affine_element> commitments;
    commitments.resize(19);

    for (size_t i = 0; i < 19; ++i) {
        commitments[i] =
            g1::affine_element(scalar_multiplication::pippenger(poly_coefficients[i],
                                                                circuit_proving_key->reference_string->get_monomials(),
                                                                circuit_proving_key->n,
                                                                state));
    }

    auto crs = crs_factory_->get_verifier_crs();
    circuit_verification_key =
        std::make_shared<verification_key>(circuit_proving_key->n, circuit_proving_key->num_public_inputs, crs);

    circuit_verification_key->constraint_selectors.insert({ "Q_1", commitments[0] });
    circuit_verification_key->constraint_selectors.insert({ "Q_2", commitments[1] });
    circuit_verification_key->constraint_selectors.insert({ "Q_3", commitments[2] });
    circuit_verification_key->constraint_selectors.insert({ "Q_4", commitments[3] });
    circuit_verification_key->constraint_selectors.insert({ "Q_5", commitments[4] });
    circuit_verification_key->constraint_selectors.insert({ "Q_M", commitments[5] });
    circuit_verification_key->constraint_selectors.insert({ "Q_C", commitments[6] });
    circuit_verification_key->constraint_selectors.insert({ "Q_ARITHMETIC_SELECTOR", commitments[7] });
    circuit_verification_key->constraint_selectors.insert({ "Q_FIXED_BASE_SELECTOR", commitments[8] });
    circuit_verification_key->constraint_selectors.insert({ "Q_RANGE_SELECTOR", commitments[9] });
    circuit_verification_key->constraint_selectors.insert({ "Q_LOGIC_SELECTOR", commitments[10] });

    circuit_verification_key->permutation_selectors.insert({ "SIGMA_1", commitments[11] });
    circuit_verification_key->permutation_selectors.insert({ "SIGMA_2", commitments[12] });
    circuit_verification_key->permutation_selectors.insert({ "SIGMA_3", commitments[13] });
    circuit_verification_key->permutation_selectors.insert({ "SIGMA_4", commitments[14] });
    circuit_verification_key->permutation_selectors.insert({ "ID_1", commitments[15] });
    circuit_verification_key->permutation_selectors.insert({ "ID_2", commitments[16] });
    circuit_verification_key->permutation_selectors.insert({ "ID_3", commitments[17] });
    circuit_verification_key->permutation_selectors.insert({ "ID_4", commitments[18] });

    std::copy(genperm_polynomial_manifest,
              genperm_polynomial_manifest + 24,
              std::back_inserter(circuit_verification_key->polynomial_manifest));

    return circuit_verification_key;
}

// std::shared_ptr<program_witness> GenPermComposer::compute_witness()
// {
//     if (computed_witness) {
//         return witness;
//     }
//     const size_t total_num_gates = n + public_inputs.size();
//     size_t log2_n = static_cast<size_t>(numeric::get_msb(total_num_gates + 1));
//     if ((1UL << log2_n) != (total_num_gates + 1)) {
//         ++log2_n;
//     }
//     size_t new_n = 1UL << log2_n;

//     for (size_t i = total_num_gates; i < new_n; ++i) {
//         w_l.emplace_back(zero_idx);
//         w_r.emplace_back(zero_idx);
//         w_o.emplace_back(zero_idx);
//         w_4.emplace_back(zero_idx);
//     }

//     polynomial poly_w_1(new_n);
//     polynomial poly_w_2(new_n);
//     polynomial poly_w_3(new_n);
//     polynomial poly_w_4(new_n);

//     for (size_t i = 0; i < public_inputs.size(); ++i) {
//         fr::__copy(fr::zero(), poly_w_1[i]);
//         fr::__copy(variables[public_inputs[i]], poly_w_2[i]);
//         fr::__copy(fr::zero(), poly_w_3[i]);
//         fr::__copy(fr::zero(), poly_w_4[i]);
//     }
//     for (size_t i = public_inputs.size(); i < new_n; ++i) {
//         fr::__copy(variables[w_l[i - public_inputs.size()]], poly_w_1.at(i));
//         fr::__copy(variables[w_r[i - public_inputs.size()]], poly_w_2.at(i));
//         fr::__copy(variables[w_o[i - public_inputs.size()]], poly_w_3.at(i));
//         fr::__copy(variables[w_4[i - public_inputs.size()]], poly_w_4.at(i));
//     }

//     witness = std::make_shared<program_witness>();
//     witness->wires.insert({ "w_1", std::move(poly_w_1) });
//     witness->wires.insert({ "w_2", std::move(poly_w_2) });
//     witness->wires.insert({ "w_3", std::move(poly_w_3) });
//     witness->wires.insert({ "w_4", std::move(poly_w_4) });

//     computed_witness = true;
//     return witness;
// }

GenPermProver GenPermComposer::create_prover()
{
    compute_proving_key();
    compute_witness();

    GenPermProver output_state(circuit_proving_key, witness, create_manifest(public_inputs.size()));

    std::unique_ptr<ProverPermutationWidget<4, true>> permutation_widget =
        std::make_unique<ProverPermutationWidget<4, true>>(circuit_proving_key.get(), witness.get());

    std::unique_ptr<ProverTurboArithmeticWidget<turbo_settings>> arithmetic_widget =
        std::make_unique<ProverTurboArithmeticWidget<turbo_settings>>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverTurboFixedBaseWidget<turbo_settings>> fixed_base_widget =
        std::make_unique<ProverTurboFixedBaseWidget<turbo_settings>>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverTurboLogicWidget<turbo_settings>> logic_widget =
        std::make_unique<ProverTurboLogicWidget<turbo_settings>>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverGenPermSortWidget<turbo_settings>> sort_widget =
        std::make_unique<ProverGenPermSortWidget<turbo_settings>>(circuit_proving_key.get(), witness.get());

    output_state.random_widgets.emplace_back(std::move(permutation_widget));
    output_state.transition_widgets.emplace_back(std::move(arithmetic_widget));
    output_state.transition_widgets.emplace_back(std::move(fixed_base_widget));
    output_state.transition_widgets.emplace_back(std::move(logic_widget));
    output_state.transition_widgets.emplace_back(std::move(sort_widget));

    return output_state;
}
// void GenPermComposer::create_dummy_gate()
// {
//     gate_flags.push_back(0);
//     uint32_t idx = add_variable(fr{ 1, 1, 1, 1 }.to_montgomery_form());
//     w_l.emplace_back(idx);
//     w_r.emplace_back(idx);
//     w_o.emplace_back(idx);
//     w_4.emplace_back(idx);
//     q_arith.emplace_back(fr::zero());
//     q_4.emplace_back(fr::zero());
//     q_5.emplace_back(fr::zero());
//     q_ecc_1.emplace_back(fr::zero());
//     q_m.emplace_back(fr::zero());
//     q_1.emplace_back(fr::zero());
//     q_2.emplace_back(fr::zero());
//     q_3.emplace_back(fr::zero());
//     q_c.emplace_back(fr::zero());
//     q_range.emplace_back(fr::zero());
//     q_logic.emplace_back(fr::zero());

//     epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
//     epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
//     epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };
//     epicycle fourth{ static_cast<uint32_t>(n), WireType::FOURTH };

//     wire_epicycles[static_cast<size_t>(idx)].emplace_back(left);
//     wire_epicycles[static_cast<size_t>(idx)].emplace_back(right);
//     wire_epicycles[static_cast<size_t>(idx)].emplace_back(out);
//     wire_epicycles[static_cast<size_t>(idx)].emplace_back(fourth);

//     ++n;
// }

UnrolledTurboProver GenPermComposer::create_unrolled_prover()
{
    compute_proving_key();
    compute_witness();

    UnrolledTurboProver output_state(circuit_proving_key, witness, create_unrolled_manifest(public_inputs.size()));

    std::unique_ptr<ProverPermutationWidget<4, true>> permutation_widget =
        std::make_unique<ProverPermutationWidget<4, true>>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverTurboArithmeticWidget<turbo_settings>> arithmetic_widget =
        std::make_unique<ProverTurboArithmeticWidget<turbo_settings>>(circuit_proving_key.get(), witness.get());

    std::unique_ptr<ProverTurboFixedBaseWidget<turbo_settings>> fixed_base_widget =
        std::make_unique<ProverTurboFixedBaseWidget<turbo_settings>>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverTurboLogicWidget<turbo_settings>> logic_widget =
        std::make_unique<ProverTurboLogicWidget<turbo_settings>>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverGenPermSortWidget<turbo_settings>> sort_widget =
        std::make_unique<ProverGenPermSortWidget<turbo_settings>>(circuit_proving_key.get(), witness.get());

    output_state.random_widgets.emplace_back(std::move(permutation_widget));
    output_state.transition_widgets.emplace_back(std::move(arithmetic_widget));
    output_state.transition_widgets.emplace_back(std::move(fixed_base_widget));
    output_state.transition_widgets.emplace_back(std::move(logic_widget));
    output_state.transition_widgets.emplace_back(std::move(sort_widget));

    return output_state;
}

GenPermVerifier GenPermComposer::create_verifier()
{
    compute_verification_key();

    GenPermVerifier output_state(circuit_verification_key, create_manifest(public_inputs.size()));

    return output_state;
}

UnrolledTurboVerifier GenPermComposer::create_unrolled_verifier()
{
    compute_verification_key();

    UnrolledTurboVerifier output_state(circuit_verification_key, create_unrolled_manifest(public_inputs.size()));

    return output_state;
}

// uint32_t GenPermComposer::put_constant_variable(const barretenberg::fr& variable)
// {
//     if (constant_variables.count(variable) == 1) {
//         return constant_variables.at(variable);
//     } else {
//         uint32_t variable_index = add_variable(variable);
//         fix_witness(variable_index, variable);
//         constant_variables.insert({ variable, variable_index });
//         return variable_index;
//     }
// }

} // namespace waffle
