#include "composer_base.hpp"
#include <proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/utils/permutation.hpp>

namespace waffle {

/**
 * Join variable class b to variable class a.
 *
 * @param a_variable_idx Index of a variable in class a.
 * @param b_variable_idx Index of a variable in class b.
 * @param msg Class tag.
 * */
void ComposerBase::assert_equal(const uint32_t a_variable_idx, const uint32_t b_variable_idx, std::string const& msg)
{
    assert_valid_variables({ a_variable_idx, b_variable_idx });
    bool values_equal = (get_variable(a_variable_idx) == get_variable(b_variable_idx));
    if (!values_equal && !failed()) {
        failure(msg);
    }
    uint32_t a_real_idx = real_variable_index[a_variable_idx];
    uint32_t b_real_idx = real_variable_index[b_variable_idx];
    // If a==b is already enforced, exit method
    if (a_real_idx == b_real_idx)
        return;
    // Otherwise update the real_idx of b-chain members to that of a
    auto b_start_idx = get_first_variable_in_class(b_variable_idx);
    update_real_variable_indices(b_start_idx, a_real_idx);
    // Now merge equivalence classes of a and b by tying last (= real) element of b-chain to first element of a-chain
    auto a_start_idx = get_first_variable_in_class(a_variable_idx);
    next_var_index[b_real_idx] = a_start_idx;
    prev_var_index[a_start_idx] = b_real_idx;
    bool no_tag_clash = (real_variable_tags[a_real_idx] == DUMMY_TAG || real_variable_tags[b_real_idx] == DUMMY_TAG ||
                         real_variable_tags[a_real_idx] == real_variable_tags[b_real_idx]);
    if (!no_tag_clash && !failed()) {
        failure(msg);
    }
    if (real_variable_tags[a_real_idx] == DUMMY_TAG)
        real_variable_tags[a_real_idx] = real_variable_tags[b_real_idx];
}

/**
 * Compute wire copy cycles
 *
 * First set all wire_copy_cycles corresponding to public_inputs to point to themselves.
 * Then go through all witnesses in w_l, w_r, w_o and w_4 (if program width is > 3) and
 * add them to cycles of their real indexes.
 *
 * @tparam program_width Program width
 * */
template <size_t program_width> void ComposerBase::compute_wire_copy_cycles()
{
    // Initialize wire_copy_cycles of public input variables to point to themselves
    for (size_t i = 0; i < public_inputs.size(); ++i) {
        cycle_node left{ static_cast<uint32_t>(i), WireType::LEFT };
        cycle_node right{ static_cast<uint32_t>(i), WireType::RIGHT };

        const auto public_input_index = real_variable_index[public_inputs[i]];
        std::vector<cycle_node>& cycle = wire_copy_cycles[static_cast<size_t>(public_input_index)];
        // These two nodes must be in adjacent locations in the cycle for correct handling of public inputs
        cycle.emplace_back(left);
        cycle.emplace_back(right);
    }

    const uint32_t num_public_inputs = static_cast<uint32_t>(public_inputs.size());

    // Go through all witnesses and add them to the wire_copy_cycles
    for (size_t i = 0; i < num_gates; ++i) {
        const auto w_1_index = real_variable_index[w_l[i]];
        const auto w_2_index = real_variable_index[w_r[i]];
        const auto w_3_index = real_variable_index[w_o[i]];

        wire_copy_cycles[static_cast<size_t>(w_1_index)].emplace_back(static_cast<uint32_t>(i + num_public_inputs),
                                                                      WireType::LEFT);
        wire_copy_cycles[static_cast<size_t>(w_2_index)].emplace_back(static_cast<uint32_t>(i + num_public_inputs),
                                                                      WireType::RIGHT);
        wire_copy_cycles[static_cast<size_t>(w_3_index)].emplace_back(static_cast<uint32_t>(i + num_public_inputs),
                                                                      WireType::OUTPUT);

        if constexpr (program_width > 3) {
            const auto w_4_index = real_variable_index[w_4[i]];
            wire_copy_cycles[static_cast<size_t>(w_4_index)].emplace_back(static_cast<uint32_t>(i + num_public_inputs),
                                                                          WireType::FOURTH);
        }
    }
}

/**
 * Compute sigma and id permutation polynomials in lagrange base.
 *
 * @param key Proving key.
 *
 * @tparam program_width Program width.
 * @tparam with_tags means that we are modifying the Plonk permutation to include "generalised subset permutations". You
 * can assign tags to wires, and then add an equivalence check between two tags. e.g. I could assign wires 'w1, w5, w3'
 * tag1, and 'w2, w4, w6' to tag2. Then assert that tag1 === tag2. The permutation polynomials are modified to assert
 * these two wire sets are equivalent (unlike normal copy constraints, the ordering of the wires within the sets is not
 * enforced). `with_tags` is closely linked with `id_poly`: id_poly is a flag that describes whether we're using
 * Vitalik's trick of using trivial identity permutation polynomials (id_poly = false). OR whether the identity
 * permutation polynomials are circuit-specific and stored in the proving/verification key (id_poly = true).
 */
template <size_t program_width, bool with_tags> void ComposerBase::compute_sigma_permutations(proving_key* key)
{
    // Compute wire copy cycles for public and private variables
    compute_wire_copy_cycles<program_width>();
    std::array<std::vector<permutation_subgroup_element>, program_width> sigma_mappings;
    std::array<std::vector<permutation_subgroup_element>, program_width> id_mappings;

    // Instantiate the sigma and id mappings by reserving enough space and pushing 'default' permutation subgroup
    // elements that point to themselves.
    for (size_t i = 0; i < program_width; ++i) {
        sigma_mappings[i].reserve(key->circuit_size);
        if (with_tags)
            id_mappings[i].reserve(key->circuit_size);
    }
    for (size_t i = 0; i < program_width; ++i) {
        for (size_t j = 0; j < key->circuit_size; ++j) {
            sigma_mappings[i].emplace_back(permutation_subgroup_element{
                .subgroup_index = (uint32_t)j, .column_index = (uint8_t)i, .is_public_input = false, .is_tag = false });
            if (with_tags)
                id_mappings[i].emplace_back(permutation_subgroup_element{ .subgroup_index = (uint32_t)j,
                                                                          .column_index = (uint8_t)i,
                                                                          .is_public_input = false,
                                                                          .is_tag = false });
        }
    }

    // Go through all wire cycles and update sigma and id mappings to point to the next element
    // within each cycle as well as set the appropriate tags
    for (size_t i = 0; i < wire_copy_cycles.size(); ++i) {
        for (size_t j = 0; j < wire_copy_cycles[i].size(); ++j) {
            cycle_node current_cycle_node = wire_copy_cycles[i][j];
            size_t next_cycle_node_index = j == wire_copy_cycles[i].size() - 1 ? 0 : j + 1;
            cycle_node next_cycle_node = wire_copy_cycles[i][next_cycle_node_index];
            const auto current_row = current_cycle_node.gate_index;
            const auto next_row = next_cycle_node.gate_index;

            const uint32_t current_column = static_cast<uint32_t>(current_cycle_node.wire_type) >> 30U;
            const uint32_t next_column = static_cast<uint32_t>(next_cycle_node.wire_type) >> 30U;

            sigma_mappings[current_column][current_row] = { .subgroup_index = next_row,
                                                            .column_index = (uint8_t)next_column,
                                                            .is_public_input = false,
                                                            .is_tag = false };

            bool first_node, last_node;
            if (with_tags) {

                first_node = j == 0;
                last_node = next_cycle_node_index == 0;
                if (first_node) {
                    id_mappings[current_column][current_row].is_tag = true;
                    id_mappings[current_column][current_row].subgroup_index = (real_variable_tags[i]);
                }
                if (last_node) {
                    sigma_mappings[current_column][current_row].is_tag = true;
                    sigma_mappings[current_column][current_row].subgroup_index = tau.at(real_variable_tags[i]);
                }
            }
        }
    }

    const uint32_t num_public_inputs = static_cast<uint32_t>(public_inputs.size());

    // This corresponds in the paper to modifying sigma to sigma' with the zeta_i values; this enforces public input
    // consistency
    for (size_t i = 0; i < num_public_inputs; ++i) {
        sigma_mappings[0][i].subgroup_index = static_cast<uint32_t>(i);
        sigma_mappings[0][i].column_index = 0;
        sigma_mappings[0][i].is_public_input = true;
        if (sigma_mappings[0][i].is_tag) {
            std::cerr << "MAPPING IS BOTH A TAG AND A PUBLIC INPUT" << std::endl;
        }
    }

    for (size_t i = 0; i < program_width; ++i) {

        // Construct permutation polynomials in lagrange base
        std::string index = std::to_string(i + 1);
        barretenberg::polynomial sigma_polynomial_lagrange(key->circuit_size);
        compute_permutation_lagrange_base_single<standard_settings>(
            sigma_polynomial_lagrange, sigma_mappings[i], key->small_domain);

        // Compute permutation polynomial monomial form
        barretenberg::polynomial sigma_polynomial(key->circuit_size);
        barretenberg::polynomial_arithmetic::ifft(
            &sigma_polynomial_lagrange[0], &sigma_polynomial[0], key->small_domain);

        // Compute permutation polynomial coset FFT form
        barretenberg::polynomial sigma_fft(sigma_polynomial, key->large_domain.size);
        sigma_fft.coset_fft(key->large_domain);

        key->polynomial_cache.put("sigma_" + index + "_lagrange", std::move(sigma_polynomial_lagrange));
        key->polynomial_cache.put("sigma_" + index, std::move(sigma_polynomial));
        key->polynomial_cache.put("sigma_" + index + "_fft", std::move(sigma_fft));

        if (with_tags) {
            // Construct id polynomials in lagrange base
            barretenberg::polynomial id_polynomial_lagrange(key->circuit_size);
            compute_permutation_lagrange_base_single<standard_settings>(
                id_polynomial_lagrange, id_mappings[i], key->small_domain);

            // Compute id polynomial monomial form
            barretenberg::polynomial id_polynomial(key->circuit_size);
            barretenberg::polynomial_arithmetic::ifft(&id_polynomial_lagrange[0], &id_polynomial[0], key->small_domain);

            // Compute id polynomial coset FFT form
            barretenberg::polynomial id_fft(id_polynomial, key->large_domain.size);
            id_fft.coset_fft(key->large_domain);

            key->polynomial_cache.put("id_" + index + "_lagrange", std::move(id_polynomial_lagrange));
            key->polynomial_cache.put("id_" + index, std::move(id_polynomial));
            key->polynomial_cache.put("id_" + index + "_fft", std::move(id_fft));
        }
    }
}

/**
 * Compute proving key base.
 *
 * 1. Load crs.
 * 2. Initialize this.circuit_proving_key.
 * 3. Create constraint selector polynomials from each of this composer's `selectors` vectors and add them to the
 * proving key.
 *
 * N.B. Need to add the fix for coefficients
 *
 * @param minimum_circuit_size Used as the total number of gates when larger than n + count of public inputs.
 * @param num_reserved_gates The number of reserved gates.
 * @return Pointer to the initialized proving key updated with selector polynomials.
 * */
std::shared_ptr<proving_key> ComposerBase::compute_proving_key_base(const waffle::ComposerType composer_type,
                                                                    const size_t minimum_circuit_size,
                                                                    const size_t num_reserved_gates)
{
    const size_t num_filled_gates = num_gates + public_inputs.size();
    const size_t total_num_gates = std::max(minimum_circuit_size, num_filled_gates);
    const size_t subgroup_size = get_circuit_subgroup_size(total_num_gates + num_reserved_gates); // next power of 2

    // In the case of standard plonk, if 4 roots are cut out of the vanishing polynomial,
    // then the degree of the quotient polynomial t(X) is 3n. This implies that the degree
    // of the constituent t_{high} of t(X) must be n (as against (n - 1) for other composer types).
    // Thus, to commit to t_{high}, we need the crs size to be (n + 1) for standard plonk.
    //
    // For more explanation about the degree of t(X), see
    // ./src/aztec/plonk/proof_system/prover/prover.cpp/ProverBase::compute_quotient_commitments
    //

    auto crs = crs_factory_->get_prover_crs(subgroup_size + 1);

    // Initialize circuit_proving_key
    circuit_proving_key = std::make_shared<proving_key>(subgroup_size, public_inputs.size(), crs, composer_type);

    for (size_t i = 0; i < num_selectors; ++i) {
        std::vector<barretenberg::fr>& selector_values = selectors[i];
        const auto& properties = selector_properties[i];
        ASSERT(num_gates == selector_values.size());

        // Fill unfilled gates' selector values with zeroes (stopping 1 short; the last value will be nonzero).
        for (size_t j = num_filled_gates; j < subgroup_size - 1; ++j) {
            selector_values.emplace_back(fr::zero());
        }

        // Add a nonzero value at the end of each selector vector. This ensures that, if the selector would otherwise
        // have been 'empty':
        //    1) that its commitment won't be the point at infinity. We avoid the point at
        //    infinity in the native verifier because this is an edge case in the recursive verifier circuit, and we
        //    want the logic to be consistent between both verifiers.
        //    2) that its commitment won't be equal to any other selectors' commitments (which would break biggroup
        //    operations when verifying snarks within a circuit, since doubling is not directly supported). This in turn
        //    ensures that when we commit to a selector, we will never get the point at infinity.
        //
        // Note: Setting the selector to nonzero would ordinarily make the proof fail if we did not have a satisfying
        // constraint. This is not the case for the last selector position, as it is never checked in the proving
        // system; observe that we cut out 4 roots and only use 3 for zero knowledge. The last root, corresponds to this
        // position.
        selector_values.emplace_back(i + 1);

        // Compute lagrange form of selector polynomial
        polynomial selector_poly_lagrange(subgroup_size);
        for (size_t k = 0; k < public_inputs.size(); ++k) {
            selector_poly_lagrange[k] = fr::zero();
        }
        for (size_t k = public_inputs.size(); k < subgroup_size; ++k) {
            selector_poly_lagrange[k] = selector_values[k - public_inputs.size()];
        }

        // Compute monomial form of selector polynomial
        polynomial selector_poly(subgroup_size);
        polynomial_arithmetic::ifft(&selector_poly_lagrange[0], &selector_poly[0], circuit_proving_key->small_domain);

        // Compute coset FFT of selector polynomial
        polynomial selector_poly_fft(selector_poly, subgroup_size * 4 + 4);
        selector_poly_fft.coset_fft(circuit_proving_key->large_domain);

        if (properties.requires_lagrange_base_polynomial) {
            circuit_proving_key->polynomial_cache.put(properties.name + "_lagrange", std::move(selector_poly_lagrange));
        }
        circuit_proving_key->polynomial_cache.put(properties.name, std::move(selector_poly));
        circuit_proving_key->polynomial_cache.put(properties.name + "_fft", std::move(selector_poly_fft));
    }

    return circuit_proving_key;
}

/**
 * @brief Computes the verification key by computing the:
 * (1) commitments to the selector and permutation polynomials,
 * (2) sets the polynomial manifest using the data from proving key.
 */
std::shared_ptr<verification_key> ComposerBase::compute_verification_key_base(
    std::shared_ptr<proving_key> const& proving_key, std::shared_ptr<VerifierReferenceString> const& vrs)
{
    auto circuit_verification_key = std::make_shared<verification_key>(
        proving_key->circuit_size, proving_key->num_public_inputs, vrs, proving_key->composer_type);

    for (size_t i = 0; i < proving_key->polynomial_manifest.size(); ++i) {
        const auto& selector_poly_info = proving_key->polynomial_manifest[i];

        const std::string selector_poly_label(selector_poly_info.polynomial_label);
        const std::string selector_commitment_label(selector_poly_info.commitment_label);

        if (selector_poly_info.source == PolynomialSource::SELECTOR ||
            selector_poly_info.source == PolynomialSource::PERMUTATION) {
            // Fetch the constraint selector polynomial in its coefficient form.
            fr* selector_poly_coefficients;
            selector_poly_coefficients = proving_key->polynomial_cache.get(selector_poly_label).get_coefficients();

            // Commit to the constraint selector polynomial and insert the commitment in the verification key.
            auto selector_poly_commitment =
                g1::affine_element(scalar_multiplication::pippenger(selector_poly_coefficients,
                                                                    proving_key->reference_string->get_monomials(),
                                                                    proving_key->circuit_size,
                                                                    proving_key->pippenger_runtime_state));

            circuit_verification_key->commitments.insert({ selector_commitment_label, selector_poly_commitment });
        }
    }

    // Set the polynomial manifest in verification key.
    circuit_verification_key->polynomial_manifest = PolynomialManifest(proving_key->composer_type);

    return circuit_verification_key;
}

/**
 * Compute witness polynomials (w_1, w_2, w_3, w_4).
 *
 * @details Fills 3 or 4 witness polynomials w_1, w_2, w_3, w_4 with the values of in-circuit variables. The beginning
 * of w_1, w_2 polynomials is filled with public_input values.
 * @return Witness with computed witness polynomials.
 *
 * @tparam Program settings needed to establish if w_4 is being used.
 * */
template <class program_settings> void ComposerBase::compute_witness_base(const size_t minimum_circuit_size)
{
    if (computed_witness) {
        return;
    }

    const size_t total_num_gates = std::max(minimum_circuit_size, num_gates + public_inputs.size());
    const size_t subgroup_size = get_circuit_subgroup_size(total_num_gates + NUM_RESERVED_GATES);

    // Note: randomness is added to 3 of the last 4 positions in plonk/proof_system/prover/prover.cpp
    // ProverBase::execute_preamble_round().
    for (size_t i = total_num_gates; i < subgroup_size; ++i) {
        w_l.emplace_back(zero_idx);
        w_r.emplace_back(zero_idx);
        w_o.emplace_back(zero_idx);
    }
    if (program_settings::program_width > 3) {
        for (size_t i = total_num_gates; i < subgroup_size; ++i) {
            w_4.emplace_back(zero_idx);
        }
    }
    polynomial w_1_lagrange = polynomial(subgroup_size);
    polynomial w_2_lagrange = polynomial(subgroup_size);
    polynomial w_3_lagrange = polynomial(subgroup_size);
    polynomial w_4_lagrange;

    if (program_settings::program_width > 3)
        w_4_lagrange = polynomial(subgroup_size);

    // Push the public inputs' values to the beginning of the wire witness polynomials.
    // Note: each public input variable is assigned to both w_1 and w_2. See
    // plonk/proof_system/public_inputs/public_inputs_impl.hpp for a giant comment explaining why.
    for (size_t i = 0; i < public_inputs.size(); ++i) {
        fr::__copy(get_variable(public_inputs[i]), w_1_lagrange[i]);
        fr::__copy(get_variable(public_inputs[i]), w_2_lagrange[i]);
        fr::__copy(fr::zero(), w_3_lagrange[i]);
        if (program_settings::program_width > 3)
            fr::__copy(fr::zero(), w_4_lagrange[i]);
    }

    // Assign the variable values (which are pointed-to by the `w_` wires) to the wire witness polynomials `poly_w_`,
    // shifted to make room for the public inputs at the beginning.
    for (size_t i = public_inputs.size(); i < subgroup_size; ++i) {
        fr::__copy(get_variable(w_l[i - public_inputs.size()]), w_1_lagrange.at(i));
        fr::__copy(get_variable(w_r[i - public_inputs.size()]), w_2_lagrange.at(i));
        fr::__copy(get_variable(w_o[i - public_inputs.size()]), w_3_lagrange.at(i));
        if (program_settings::program_width > 3)
            fr::__copy(get_variable(w_4[i - public_inputs.size()]), w_4_lagrange.at(i));
    }

    circuit_proving_key->polynomial_cache.put("w_1_lagrange", std::move(w_1_lagrange));
    circuit_proving_key->polynomial_cache.put("w_2_lagrange", std::move(w_2_lagrange));
    circuit_proving_key->polynomial_cache.put("w_3_lagrange", std::move(w_3_lagrange));
    if (program_settings::program_width > 3) {
        circuit_proving_key->polynomial_cache.put("w_4_lagrange", std::move(w_4_lagrange));
    }

    computed_witness = true;
}

template void ComposerBase::compute_sigma_permutations<3, false>(proving_key* key);
template void ComposerBase::compute_sigma_permutations<4, false>(proving_key* key);
template void ComposerBase::compute_sigma_permutations<4, true>(proving_key* key);
template void ComposerBase::compute_witness_base<standard_settings>(const size_t);
template void ComposerBase::compute_witness_base<turbo_settings>(const size_t);
template void ComposerBase::compute_witness_base<ultra_settings>(const size_t);
template void ComposerBase::compute_wire_copy_cycles<3>();
template void ComposerBase::compute_wire_copy_cycles<4>();

} // namespace waffle
