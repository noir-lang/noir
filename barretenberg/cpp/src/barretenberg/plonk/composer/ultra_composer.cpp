#include "ultra_composer.hpp"
#include "barretenberg/plonk/composer/composer_lib.hpp"
#include "barretenberg/plonk/proof_system/commitment_scheme/kate_commitment_scheme.hpp"
#include "barretenberg/plonk/proof_system/types/program_settings.hpp"
#include "barretenberg/plonk/proof_system/types/prover_settings.hpp"
#include "barretenberg/plonk/proof_system/verifier/verifier.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"
#include "barretenberg/proof_system/composer/permutation_lib.hpp"

#include <cstddef>
#include <cstdint>
#include <string>

namespace bb::plonk {

/**
 * @brief Computes `this.witness`, which is basiclly a set of polynomials mapped-to by strings.
 *
 * Note: this doesn't actually compute the _entire_ witness. Things missing: randomness for blinding both the wires and
 * sorted `s` poly, lookup rows of the wire witnesses, the values of `z_lookup`, `z`. These are all calculated
 * elsewhere.
 */

void UltraComposer::compute_witness(CircuitBuilder& circuit_constructor)
{
    if (computed_witness) {
        return;
    }

    size_t tables_size = 0;
    size_t lookups_size = 0;
    for (const auto& table : circuit_constructor.lookup_tables) {
        tables_size += table.size;
        lookups_size += table.lookup_gates.size();
    }

    const size_t filled_gates = circuit_constructor.num_gates + circuit_constructor.public_inputs.size();
    const size_t total_num_gates = std::max(filled_gates, tables_size + lookups_size);

    const size_t subgroup_size = circuit_constructor.get_circuit_subgroup_size(total_num_gates + NUM_RESERVED_GATES);

    // Pad the wires (pointers to `witness_indices` of the `variables` vector).
    // Note: the remaining NUM_RESERVED_GATES indices are padded with zeros within `compute_witness_base` (called
    // next).
    for (size_t i = filled_gates; i < total_num_gates; ++i) {
        circuit_constructor.w_l().emplace_back(circuit_constructor.zero_idx);
        circuit_constructor.w_r().emplace_back(circuit_constructor.zero_idx);
        circuit_constructor.w_o().emplace_back(circuit_constructor.zero_idx);
        circuit_constructor.w_4().emplace_back(circuit_constructor.zero_idx);
    }

    auto wire_polynomial_evaluations = construct_wire_polynomials_base<Flavor>(circuit_constructor, subgroup_size);

    for (size_t j = 0; j < program_width; ++j) {
        std::string index = std::to_string(j + 1);
        circuit_proving_key->polynomial_store.put("w_" + index + "_lagrange",
                                                  std::move(wire_polynomial_evaluations[j]));
    }

    polynomial s_1(subgroup_size);
    polynomial s_2(subgroup_size);
    polynomial s_3(subgroup_size);
    polynomial s_4(subgroup_size);
    polynomial z_lookup(subgroup_size + 1); // Only instantiated in this function; nothing assigned.

    // Save space for adding random scalars in the s polynomial later.
    // The subtracted 1 allows us to insert a `1` at the end, to ensure the evaluations (and hence coefficients) aren't
    // all 0.
    // See ComposerBase::compute_proving_key_base for further explanation, as a similar trick is done there.
    size_t count = subgroup_size - tables_size - lookups_size - s_randomness - 1;
    for (size_t i = 0; i < count; ++i) {
        s_1[i] = 0;
        s_2[i] = 0;
        s_3[i] = 0;
        s_4[i] = 0;
    }

    for (auto& table : circuit_constructor.lookup_tables) {
        const fr table_index(table.table_index);
        auto& lookup_gates = table.lookup_gates;
        for (size_t i = 0; i < table.size; ++i) {
            if (table.use_twin_keys) {
                lookup_gates.push_back({
                    {
                        table.column_1[i].from_montgomery_form().data[0],
                        table.column_2[i].from_montgomery_form().data[0],
                    },
                    {
                        table.column_3[i],
                        0,
                    },
                });
            } else {
                lookup_gates.push_back({
                    {
                        table.column_1[i].from_montgomery_form().data[0],
                        0,
                    },
                    {
                        table.column_2[i],
                        table.column_3[i],
                    },
                });
            }
        }

#ifdef NO_TBB
        std::sort(lookup_gates.begin(), lookup_gates.end());
#else
        std::sort(std::execution::par_unseq, lookup_gates.begin(), lookup_gates.end());
#endif

        for (const auto& entry : lookup_gates) {
            const auto components = entry.to_sorted_list_components(table.use_twin_keys);
            s_1[count] = components[0];
            s_2[count] = components[1];
            s_3[count] = components[2];
            s_4[count] = table_index;
            ++count;
        }
    }

    // Initialize the `s_randomness` positions in the s polynomials with 0.
    // These will be the positions where we will be adding random scalars to add zero knowledge
    // to plookup (search for `Blinding` in plonk/proof_system/widgets/random_widgets/plookup_widget_impl.hpp
    // ProverPlookupWidget::compute_sorted_list_polynomial())
    for (size_t i = 0; i < s_randomness; ++i) {
        s_1[count] = 0;
        s_2[count] = 0;
        s_3[count] = 0;
        s_4[count] = 0;
        ++count;
    }

    circuit_proving_key->polynomial_store.put("s_1_lagrange", std::move(s_1));
    circuit_proving_key->polynomial_store.put("s_2_lagrange", std::move(s_2));
    circuit_proving_key->polynomial_store.put("s_3_lagrange", std::move(s_3));
    circuit_proving_key->polynomial_store.put("s_4_lagrange", std::move(s_4));

    // Copy memory read/write record data into proving key. Prover needs to know which gates contain a read/write
    // 'record' witness on the 4th wire. This wire value can only be fully computed once the first 3 wire polynomials
    // have been committed to. The 4th wire on these gates will be a random linear combination of the first 3 wires,
    // using the plookup challenge `eta`. Because we shift the gates by the number of public inputs, we need to update
    // the records with the public_inputs offset
    const uint32_t public_inputs_count = static_cast<uint32_t>(circuit_constructor.public_inputs.size());
    auto add_public_inputs_offset = [public_inputs_count](uint32_t gate_index) {
        return gate_index + public_inputs_count;
    };
    circuit_proving_key->memory_read_records = std::vector<uint32_t>();
    circuit_proving_key->memory_write_records = std::vector<uint32_t>();

    std::transform(circuit_constructor.memory_read_records.begin(),
                   circuit_constructor.memory_read_records.end(),
                   std::back_inserter(circuit_proving_key->memory_read_records),
                   add_public_inputs_offset);
    std::transform(circuit_constructor.memory_write_records.begin(),
                   circuit_constructor.memory_write_records.end(),
                   std::back_inserter(circuit_proving_key->memory_write_records),
                   add_public_inputs_offset);

    computed_witness = true;
}

UltraProver UltraComposer::create_prover(CircuitBuilder& circuit_constructor)
{
    circuit_constructor.finalize_circuit();

    compute_proving_key(circuit_constructor);
    compute_witness(circuit_constructor);

    UltraProver output_state(circuit_proving_key, create_manifest(circuit_constructor.public_inputs.size()));

    auto permutation_widget = std::make_unique<ProverPermutationWidget<4, true>>(circuit_proving_key.get());
    auto plookup_widget = std::make_unique<ProverPlookupWidget<>>(circuit_proving_key.get());
    auto arithmetic_widget = std::make_unique<ProverPlookupArithmeticWidget<ultra_settings>>(circuit_proving_key.get());
    auto sort_widget = std::make_unique<ProverGenPermSortWidget<ultra_settings>>(circuit_proving_key.get());
    auto elliptic_widget = std::make_unique<ProverEllipticWidget<ultra_settings>>(circuit_proving_key.get());
    auto auxiliary_widget = std::make_unique<ProverPlookupAuxiliaryWidget<ultra_settings>>(circuit_proving_key.get());

    output_state.random_widgets.emplace_back(std::move(permutation_widget));
    output_state.random_widgets.emplace_back(std::move(plookup_widget));

    output_state.transition_widgets.emplace_back(std::move(arithmetic_widget));
    output_state.transition_widgets.emplace_back(std::move(sort_widget));
    output_state.transition_widgets.emplace_back(std::move(elliptic_widget));
    output_state.transition_widgets.emplace_back(std::move(auxiliary_widget));

    std::unique_ptr<KateCommitmentScheme<ultra_settings>> kate_commitment_scheme =
        std::make_unique<KateCommitmentScheme<ultra_settings>>();

    output_state.commitment_scheme = std::move(kate_commitment_scheme);

    return output_state;
}

/**
 * @brief Uses slightly different settings from the UltraProver.
 */
UltraToStandardProver UltraComposer::create_ultra_to_standard_prover(CircuitBuilder& circuit_constructor)
{
    circuit_constructor.finalize_circuit();

    compute_proving_key(circuit_constructor);
    compute_witness(circuit_constructor);

    UltraToStandardProver output_state(circuit_proving_key, create_manifest(circuit_constructor.public_inputs.size()));

    std::unique_ptr<ProverPermutationWidget<4, true>> permutation_widget =
        std::make_unique<ProverPermutationWidget<4, true>>(circuit_proving_key.get());

    std::unique_ptr<ProverPlookupWidget<>> plookup_widget =
        std::make_unique<ProverPlookupWidget<>>(circuit_proving_key.get());

    std::unique_ptr<ProverPlookupArithmeticWidget<ultra_to_standard_settings>> arithmetic_widget =
        std::make_unique<ProverPlookupArithmeticWidget<ultra_to_standard_settings>>(circuit_proving_key.get());

    std::unique_ptr<ProverGenPermSortWidget<ultra_to_standard_settings>> sort_widget =
        std::make_unique<ProverGenPermSortWidget<ultra_to_standard_settings>>(circuit_proving_key.get());

    std::unique_ptr<ProverEllipticWidget<ultra_to_standard_settings>> elliptic_widget =
        std::make_unique<ProverEllipticWidget<ultra_to_standard_settings>>(circuit_proving_key.get());

    std::unique_ptr<ProverPlookupAuxiliaryWidget<ultra_to_standard_settings>> auxiliary_widget =
        std::make_unique<ProverPlookupAuxiliaryWidget<ultra_to_standard_settings>>(circuit_proving_key.get());

    output_state.random_widgets.emplace_back(std::move(permutation_widget));
    output_state.random_widgets.emplace_back(std::move(plookup_widget));

    output_state.transition_widgets.emplace_back(std::move(arithmetic_widget));
    output_state.transition_widgets.emplace_back(std::move(sort_widget));
    output_state.transition_widgets.emplace_back(std::move(elliptic_widget));
    output_state.transition_widgets.emplace_back(std::move(auxiliary_widget));

    std::unique_ptr<KateCommitmentScheme<ultra_to_standard_settings>> kate_commitment_scheme =
        std::make_unique<KateCommitmentScheme<ultra_to_standard_settings>>();

    output_state.commitment_scheme = std::move(kate_commitment_scheme);

    return output_state;
}

/**
 * @brief Uses slightly different settings from the UltraProver.
 */
UltraWithKeccakProver UltraComposer::create_ultra_with_keccak_prover(CircuitBuilder& circuit_constructor)
{
    circuit_constructor.finalize_circuit();
    compute_proving_key(circuit_constructor);
    compute_witness(circuit_constructor);

    UltraWithKeccakProver output_state(circuit_proving_key, create_manifest(circuit_constructor.public_inputs.size()));

    std::unique_ptr<ProverPermutationWidget<4, true>> permutation_widget =
        std::make_unique<ProverPermutationWidget<4, true>>(circuit_proving_key.get());

    std::unique_ptr<ProverPlookupWidget<>> plookup_widget =
        std::make_unique<ProverPlookupWidget<>>(circuit_proving_key.get());

    std::unique_ptr<ProverPlookupArithmeticWidget<ultra_with_keccak_settings>> arithmetic_widget =
        std::make_unique<ProverPlookupArithmeticWidget<ultra_with_keccak_settings>>(circuit_proving_key.get());

    std::unique_ptr<ProverGenPermSortWidget<ultra_with_keccak_settings>> sort_widget =
        std::make_unique<ProverGenPermSortWidget<ultra_with_keccak_settings>>(circuit_proving_key.get());

    std::unique_ptr<ProverEllipticWidget<ultra_with_keccak_settings>> elliptic_widget =
        std::make_unique<ProverEllipticWidget<ultra_with_keccak_settings>>(circuit_proving_key.get());

    std::unique_ptr<ProverPlookupAuxiliaryWidget<ultra_with_keccak_settings>> auxiliary_widget =
        std::make_unique<ProverPlookupAuxiliaryWidget<ultra_with_keccak_settings>>(circuit_proving_key.get());

    output_state.random_widgets.emplace_back(std::move(permutation_widget));
    output_state.random_widgets.emplace_back(std::move(plookup_widget));

    output_state.transition_widgets.emplace_back(std::move(arithmetic_widget));
    output_state.transition_widgets.emplace_back(std::move(sort_widget));
    output_state.transition_widgets.emplace_back(std::move(elliptic_widget));
    output_state.transition_widgets.emplace_back(std::move(auxiliary_widget));

    std::unique_ptr<KateCommitmentScheme<ultra_with_keccak_settings>> kate_commitment_scheme =
        std::make_unique<KateCommitmentScheme<ultra_with_keccak_settings>>();

    output_state.commitment_scheme = std::move(kate_commitment_scheme);

    return output_state;
}

/**
 * Create verifier: compute verification key,
 * initialize verifier with it and an initial manifest and initialize commitment_scheme.
 *
 * @return The verifier.
 * */

plonk::UltraVerifier UltraComposer::create_verifier(CircuitBuilder& circuit_constructor)
{
    auto verification_key = compute_verification_key(circuit_constructor);

    plonk::UltraVerifier output_state(circuit_verification_key,
                                      create_manifest(circuit_constructor.public_inputs.size()));

    auto kate_commitment_scheme = std::make_unique<plonk::KateCommitmentScheme<plonk::ultra_settings>>();

    output_state.commitment_scheme = std::move(kate_commitment_scheme);

    return output_state;
}

/**
 * @brief Create a verifier using pedersen hash for the transcript
 *
 * @param circuit_constructor
 * @return UltraToStandardVerifier
 */
UltraToStandardVerifier UltraComposer::create_ultra_to_standard_verifier(CircuitBuilder& circuit_constructor)
{
    auto verification_key = compute_verification_key(circuit_constructor);

    UltraToStandardVerifier output_state(circuit_verification_key,
                                         create_manifest(circuit_constructor.public_inputs.size()));

    std::unique_ptr<KateCommitmentScheme<ultra_to_standard_settings>> kate_commitment_scheme =
        std::make_unique<KateCommitmentScheme<ultra_to_standard_settings>>();

    output_state.commitment_scheme = std::move(kate_commitment_scheme);

    return output_state;
}

/**
 * @brief Create a verifier using keccak for the transcript
 *
 * @param circuit_constructor
 * @return UltraWithKeccakVerifier
 */
UltraWithKeccakVerifier UltraComposer::create_ultra_with_keccak_verifier(CircuitBuilder& circuit_constructor)
{
    auto verification_key = compute_verification_key(circuit_constructor);

    UltraWithKeccakVerifier output_state(circuit_verification_key,
                                         create_manifest(circuit_constructor.public_inputs.size()));

    std::unique_ptr<KateCommitmentScheme<ultra_with_keccak_settings>> kate_commitment_scheme =
        std::make_unique<KateCommitmentScheme<ultra_with_keccak_settings>>();

    output_state.commitment_scheme = std::move(kate_commitment_scheme);

    return output_state;
}

std::shared_ptr<proving_key> UltraComposer::compute_proving_key(CircuitBuilder& circuit_constructor)
{
    if (circuit_proving_key) {
        return circuit_proving_key;
    }

    circuit_constructor.finalize_circuit();

    size_t tables_size = 0;
    size_t lookups_size = 0;
    for (const auto& table : circuit_constructor.lookup_tables) {
        tables_size += table.size;
        lookups_size += table.lookup_gates.size();
    }

    const size_t minimum_circuit_size = tables_size + lookups_size;
    const size_t num_randomized_gates = NUM_RESERVED_GATES;
    auto crs_factory = srs::get_crs_factory();
    // Initialize circuit_proving_key
    // TODO(#392)(Kesha): replace composer types.
    circuit_proving_key = initialize_proving_key(
        circuit_constructor, crs_factory.get(), minimum_circuit_size, num_randomized_gates, CircuitType::ULTRA);

    construct_selector_polynomials<Flavor>(circuit_constructor, circuit_proving_key.get());

    enforce_nonzero_selector_polynomials(circuit_constructor, circuit_proving_key.get());

    compute_monomial_and_coset_selector_forms(circuit_proving_key.get(), ultra_selector_properties());

    compute_plonk_generalized_sigma_permutations<Flavor>(circuit_constructor, circuit_proving_key.get());

    const size_t subgroup_size = circuit_proving_key->circuit_size;

    polynomial poly_q_table_column_1(subgroup_size);
    polynomial poly_q_table_column_2(subgroup_size);
    polynomial poly_q_table_column_3(subgroup_size);
    polynomial poly_q_table_column_4(subgroup_size);

    size_t offset = subgroup_size - tables_size - s_randomness - 1;

    // Create lookup selector polynomials which interpolate each table column.
    // Our selector polys always need to interpolate the full subgroup size, so here we offset so as to
    // put the table column's values at the end. (The first gates are for non-lookup constraints).
    // [0, ..., 0, ...table, 0, 0, 0, x]
    //  ^^^^^^^^^  ^^^^^^^^  ^^^^^^^  ^nonzero to ensure uniqueness and to avoid infinity commitments
    //  |          table     randomness
    //  ignored, as used for regular constraints and padding to the next power of 2.

    for (size_t i = 0; i < offset; ++i) {
        poly_q_table_column_1[i] = 0;
        poly_q_table_column_2[i] = 0;
        poly_q_table_column_3[i] = 0;
        poly_q_table_column_4[i] = 0;
    }

    for (const auto& table : circuit_constructor.lookup_tables) {
        const fr table_index(table.table_index);

        for (size_t i = 0; i < table.size; ++i) {
            poly_q_table_column_1[offset] = table.column_1[i];
            poly_q_table_column_2[offset] = table.column_2[i];
            poly_q_table_column_3[offset] = table.column_3[i];
            poly_q_table_column_4[offset] = table_index;
            ++offset;
        }
    }

    // Initialize the last `s_randomness` positions in table polynomials with 0. We don't need to actually randomize
    // the table polynomials.
    for (size_t i = 0; i < s_randomness; ++i) {
        poly_q_table_column_1[offset] = 0;
        poly_q_table_column_2[offset] = 0;
        poly_q_table_column_3[offset] = 0;
        poly_q_table_column_4[offset] = 0;
        ++offset;
    }

    // // In the case of using UltraPlonkComposer for a circuit which does _not_ make use of any lookup tables, all four
    // // table columns would be all zeros. This would result in these polys' commitments all being the point at
    // infinity
    // // (which is bad because our point arithmetic assumes we'll never operate on the point at infinity). To avoid
    // this,
    // // we set the last evaluation of each poly to be nonzero. The last `num_roots_cut_out_of_vanishing_poly = 4`
    // // evaluations are ignored by constraint checks; we arbitrarily choose the very-last evaluation to be nonzero.
    // See
    // // ComposerBase::compute_proving_key_base for further explanation, as a similar trick is done there. We could
    // // have chosen `1` for each such evaluation here, but that would have resulted in identical commitments for
    // // all four columns. We don't want to have equal commitments, because biggroup operations assume no points are
    // // equal, so if we tried to verify an ultra proof in a circuit, the biggroup operations would fail. To combat
    // // this, we just choose distinct values:
    ASSERT(offset == subgroup_size - 1);
    auto unique_last_value =
        get_num_selectors() + 1; // Note: in compute_proving_key_base, moments earlier, each selector
                                 // vector was given a unique last value from 1..num_selectors. So we
                                 // avoid those values and continue the count, to ensure uniqueness.
    poly_q_table_column_1[subgroup_size - 1] = unique_last_value;
    poly_q_table_column_2[subgroup_size - 1] = ++unique_last_value;
    poly_q_table_column_3[subgroup_size - 1] = ++unique_last_value;
    poly_q_table_column_4[subgroup_size - 1] = ++unique_last_value;

    add_table_column_selector_poly_to_proving_key(poly_q_table_column_1, "table_value_1");
    add_table_column_selector_poly_to_proving_key(poly_q_table_column_2, "table_value_2");
    add_table_column_selector_poly_to_proving_key(poly_q_table_column_3, "table_value_3");
    add_table_column_selector_poly_to_proving_key(poly_q_table_column_4, "table_value_4");

    // Instantiate z_lookup and s polynomials in the proving key (no values assigned yet).
    // Note: might be better to add these polys to cache only after they've been computed, as is convention
    // TODO(luke): Don't put empty polynomials in the store, just add these where they're computed
    polynomial z_lookup_fft(subgroup_size * 4);
    polynomial s_fft(subgroup_size * 4);
    circuit_proving_key->polynomial_store.put("z_lookup_fft", std::move(z_lookup_fft));
    circuit_proving_key->polynomial_store.put("s_fft", std::move(s_fft));

    circuit_proving_key->recursive_proof_public_input_indices =
        std::vector<uint32_t>(circuit_constructor.recursive_proof_public_input_indices.begin(),
                              circuit_constructor.recursive_proof_public_input_indices.end());

    circuit_proving_key->contains_recursive_proof = circuit_constructor.contains_recursive_proof;

    return circuit_proving_key;
}

/**
 * Compute verification key consisting of selector precommitments.
 *
 * @return Pointer to created circuit verification key.
 * */

std::shared_ptr<plonk::verification_key> UltraComposer::compute_verification_key(CircuitBuilder& circuit_constructor)
{
    if (circuit_verification_key) {
        return circuit_verification_key;
    }

    auto crs_factory = srs::get_crs_factory();

    if (!circuit_proving_key) {
        compute_proving_key(circuit_constructor);
    }
    circuit_verification_key = compute_verification_key_common(circuit_proving_key, crs_factory->get_verifier_crs());

    circuit_verification_key->circuit_type = CircuitType::ULTRA;

    // See `add_recusrive_proof()` for how this recursive data is assigned.
    circuit_verification_key->recursive_proof_public_input_indices =
        std::vector<uint32_t>(circuit_constructor.recursive_proof_public_input_indices.begin(),
                              circuit_constructor.recursive_proof_public_input_indices.end());

    circuit_verification_key->contains_recursive_proof = circuit_constructor.contains_recursive_proof;

    return circuit_verification_key;
}

void UltraComposer::add_table_column_selector_poly_to_proving_key(polynomial& selector_poly_lagrange_form,
                                                                  const std::string& tag)
{
    polynomial selector_poly_lagrange_form_copy(selector_poly_lagrange_form, circuit_proving_key->small_domain.size);

    selector_poly_lagrange_form.ifft(circuit_proving_key->small_domain);
    auto& selector_poly_coeff_form = selector_poly_lagrange_form;

    polynomial selector_poly_coset_form(selector_poly_coeff_form, circuit_proving_key->circuit_size * 4);
    selector_poly_coset_form.coset_fft(circuit_proving_key->large_domain);

    circuit_proving_key->polynomial_store.put(tag, std::move(selector_poly_coeff_form));
    circuit_proving_key->polynomial_store.put(tag + "_lagrange", std::move(selector_poly_lagrange_form_copy));
    circuit_proving_key->polynomial_store.put(tag + "_fft", std::move(selector_poly_coset_form));
}

} // namespace bb::plonk
