#include "ultra_honk_composer_helper.hpp"
#include "barretenberg/honk/proof_system/ultra_prover.hpp"
#include "barretenberg/plonk/proof_system/types/program_settings.hpp"
#include "barretenberg/plonk/proof_system/types/prover_settings.hpp"
// #include "barretenberg/plonk/proof_system/verifier/verifier.hpp"
#include "barretenberg/proof_system/circuit_constructors/ultra_circuit_constructor.hpp"
#include "barretenberg/proof_system/composer/permutation_helper.hpp"
#include "barretenberg/plonk/proof_system/commitment_scheme/kate_commitment_scheme.hpp"

#include <cstddef>
#include <cstdint>
#include <string>
#include <utility>

namespace proof_system::honk {

/**
 * @brief Compute witness polynomials
 *
 * TODO(luke): The wire polynomials are returned directly whereas the sorted list polys are added to the proving
 * key. This should be made consistent once Cody's Flavor work is settled.
 */
template <typename CircuitConstructor>
void UltraHonkComposerHelper<CircuitConstructor>::compute_witness(CircuitConstructor& circuit_constructor)
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
        circuit_constructor.w_l.emplace_back(circuit_constructor.zero_idx);
        circuit_constructor.w_r.emplace_back(circuit_constructor.zero_idx);
        circuit_constructor.w_o.emplace_back(circuit_constructor.zero_idx);
        circuit_constructor.w_4.emplace_back(circuit_constructor.zero_idx);
    }

    // TODO(#340)(luke): within compute_witness_base, the 3rd argument is used in the calculation of the dyadic circuit
    // size (subgroup_size). Here (and in other split composers) we're passing in NUM_RANDOMIZED_GATES, but elsewhere,
    // e.g. directly above, we use NUM_RESERVED_GATES in a similar role. Therefore, these two constants must be equal
    // for everything to be consistent. What we should do is compute the dyadic circuit size once and for all then pass
    // that around rather than computing in multiple places.
    wire_polynomials = compute_witness_base(circuit_constructor, total_num_gates, NUM_RANDOMIZED_GATES);

    polynomial s_1(subgroup_size);
    polynomial s_2(subgroup_size);
    polynomial s_3(subgroup_size);
    polynomial s_4(subgroup_size);
    polynomial z_lookup(subgroup_size + 1); // Only instantiated in this function; nothing assigned.

    // Save space for adding random scalars in the s polynomial later. The subtracted 1 allows us to insert a `1` at the
    // end, to ensure the evaluations (and hence coefficients) aren't all 0. See ComposerBase::compute_proving_key_base
    // for further explanation, as a similar trick is done there.
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

    // Initialise the `s_randomness` positions in the s polynomials with 0.
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

    // TODO(luke): Adding these to the key for now but this is inconsistent since these are 'witness' polys. Need
    // to see what becomes of the proving key before making a decision here.
    circuit_proving_key->polynomial_store.put("s_1_lagrange", std::move(s_1));
    circuit_proving_key->polynomial_store.put("s_2_lagrange", std::move(s_2));
    circuit_proving_key->polynomial_store.put("s_3_lagrange", std::move(s_3));
    circuit_proving_key->polynomial_store.put("s_4_lagrange", std::move(s_4));

    computed_witness = true;
}

template <typename CircuitConstructor>
UltraProver UltraHonkComposerHelper<CircuitConstructor>::create_prover(CircuitConstructor& circuit_constructor)
{
    finalize_circuit(circuit_constructor);

    compute_proving_key(circuit_constructor);
    compute_witness(circuit_constructor);

    UltraProver output_state(std::move(wire_polynomials), circuit_proving_key);

    return output_state;
}

// /**
//  * Create verifier: compute verification key,
//  * initialize verifier with it and an initial manifest and initialize commitment_scheme.
//  *
//  * @return The verifier.
//  * */
// // TODO(Cody): This should go away altogether.
// template <typename CircuitConstructor>
// plonk::UltraVerifier UltraHonkComposerHelper<CircuitConstructor>::create_verifier(
//     const CircuitConstructor& circuit_constructor)
// {
//     auto verification_key = compute_verification_key(circuit_constructor);

//     plonk::UltraVerifier output_state(circuit_verification_key,
//                                       create_manifest(circuit_constructor.public_inputs.size()));

//     std::unique_ptr<plonk::KateCommitmentScheme<plonk::ultra_settings>> kate_commitment_scheme =
//         std::make_unique<plonk::KateCommitmentScheme<plonk::ultra_settings>>();

//     output_state.commitment_scheme = std::move(kate_commitment_scheme);

//     return output_state;
// }

template <typename CircuitConstructor>
std::shared_ptr<plonk::proving_key> UltraHonkComposerHelper<CircuitConstructor>::compute_proving_key(
    const CircuitConstructor& circuit_constructor)
{
    if (circuit_proving_key) {
        return circuit_proving_key;
    }

    size_t tables_size = 0;
    size_t lookups_size = 0;
    for (const auto& table : circuit_constructor.lookup_tables) {
        tables_size += table.size;
        lookups_size += table.lookup_gates.size();
    }

    const size_t minimum_circuit_size = tables_size + lookups_size;
    const size_t num_randomized_gates = NUM_RANDOMIZED_GATES;
    // Initialize circuit_proving_key
    // TODO(#229)(Kesha): replace composer types.
    circuit_proving_key = initialize_proving_key(
        circuit_constructor, crs_factory_.get(), minimum_circuit_size, num_randomized_gates, ComposerType::PLOOKUP);

    construct_lagrange_selector_forms(circuit_constructor, circuit_proving_key.get());

    // TODO(#217)(luke): Naively enforcing non-zero selectors for Honk will result in some relations not being
    // satisfied.
    // enforce_nonzero_polynomial_selectors(circuit_constructor, circuit_proving_key.get());

    compute_honk_generalized_sigma_permutations<CircuitConstructor::program_width>(circuit_constructor,
                                                                                   circuit_proving_key.get());

    compute_first_and_last_lagrange_polynomials(circuit_proving_key.get());

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

    // Initialise the last `s_randomness` positions in table polynomials with 0. We don't need to actually randomise
    // the table polynomials.
    for (size_t i = 0; i < s_randomness; ++i) {
        poly_q_table_column_1[offset] = 0;
        poly_q_table_column_2[offset] = 0;
        poly_q_table_column_3[offset] = 0;
        poly_q_table_column_4[offset] = 0;
        ++offset;
    }

    // // In the case of using UltraPlonkComposer for a circuit which does _not_ make use of any lookup tables, all
    // four
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

    // TODO(#217)(luke): Similar to the selectors, enforcing non-zero values by inserting an arbitrary final element
    // in the table polys will result in lookup relations not being satisfied. Address this with issue #217.
    // size_t num_selectors = circuit_constructor.num_selectors;
    // ASSERT(offset == subgroup_size - 1);
    // auto unique_last_value = num_selectors + 1; // Note: in compute_proving_key_base, moments earlier, each selector
    //                                             // vector was given a unique last value from 1..num_selectors. So we
    //                                             // avoid those values and continue the count, to ensure uniqueness.
    // poly_q_table_column_1[subgroup_size - 1] = unique_last_value;
    // poly_q_table_column_2[subgroup_size - 1] = ++unique_last_value;
    // poly_q_table_column_3[subgroup_size - 1] = ++unique_last_value;
    // poly_q_table_column_4[subgroup_size - 1] = ++unique_last_value;

    circuit_proving_key->polynomial_store.put("table_value_1_lagrange", std::move(poly_q_table_column_1));
    circuit_proving_key->polynomial_store.put("table_value_2_lagrange", std::move(poly_q_table_column_2));
    circuit_proving_key->polynomial_store.put("table_value_3_lagrange", std::move(poly_q_table_column_3));
    circuit_proving_key->polynomial_store.put("table_value_4_lagrange", std::move(poly_q_table_column_4));

    // Copy memory read/write record data into proving key. Prover needs to know which gates contain a read/write
    // 'record' witness on the 4th wire. This wire value can only be fully computed once the first 3 wire polynomials
    // have been committed to. The 4th wire on these gates will be a random linear combination of the first 3 wires,
    // using the plookup challenge `eta`
    std::copy(circuit_constructor.memory_read_records.begin(),
              circuit_constructor.memory_read_records.end(),
              std::back_inserter(circuit_proving_key->memory_read_records));
    std::copy(circuit_constructor.memory_write_records.begin(),
              circuit_constructor.memory_write_records.end(),
              std::back_inserter(circuit_proving_key->memory_write_records));

    circuit_proving_key->recursive_proof_public_input_indices =
        std::vector<uint32_t>(recursive_proof_public_input_indices.begin(), recursive_proof_public_input_indices.end());

    circuit_proving_key->contains_recursive_proof = contains_recursive_proof;

    return circuit_proving_key;
}

// /**
//  * Compute verification key consisting of selector precommitments.
//  *
//  * @return Pointer to created circuit verification key.
//  * */
// template <typename CircuitConstructor>
// std::shared_ptr<plonk::verification_key> UltraHonkComposerHelper<CircuitConstructor>::compute_verification_key(
//     const CircuitConstructor& circuit_constructor)
// {
//     if (circuit_verification_key) {
//         return circuit_verification_key;
//     }

//     if (!circuit_proving_key) {
//         compute_proving_key(circuit_constructor);
//     }
//     circuit_verification_key = compute_verification_key_common(circuit_proving_key,
//     crs_factory_->get_verifier_crs());

//     circuit_verification_key->composer_type = type; // Invariably plookup for this class.

//     // See `add_recusrive_proof()` for how this recursive data is assigned.
//     circuit_verification_key->recursive_proof_public_input_indices =
//         std::vector<uint32_t>(recursive_proof_public_input_indices.begin(),
//         recursive_proof_public_input_indices.end());

//     circuit_verification_key->contains_recursive_proof = contains_recursive_proof;

//     return circuit_verification_key;
// }

// template <typename CircuitConstructor>
// void UltraHonkComposerHelper<CircuitConstructor>::add_table_column_selector_poly_to_proving_key(
//     polynomial& selector_poly_lagrange_form, const std::string& tag)
// {
//     polynomial selector_poly_lagrange_form_copy(selector_poly_lagrange_form, circuit_proving_key->small_domain.size);

//     selector_poly_lagrange_form.ifft(circuit_proving_key->small_domain);
//     auto& selector_poly_coeff_form = selector_poly_lagrange_form;

//     polynomial selector_poly_coset_form(selector_poly_coeff_form, circuit_proving_key->circuit_size * 4);
//     selector_poly_coset_form.coset_fft(circuit_proving_key->large_domain);

//     circuit_proving_key->polynomial_store.put(tag, std::move(selector_poly_coeff_form));
//     circuit_proving_key->polynomial_store.put(tag + "_lagrange", std::move(selector_poly_lagrange_form_copy));
//     circuit_proving_key->polynomial_store.put(tag + "_fft", std::move(selector_poly_coset_form));
// }

template class UltraHonkComposerHelper<UltraCircuitConstructor>;
} // namespace proof_system::honk
