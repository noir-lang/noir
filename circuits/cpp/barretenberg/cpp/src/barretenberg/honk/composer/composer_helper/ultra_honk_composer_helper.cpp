#include "ultra_honk_composer_helper.hpp"
#include "barretenberg/honk/proof_system/ultra_prover.hpp"
#include "barretenberg/proof_system/circuit_constructors/ultra_circuit_constructor.hpp"
#include "barretenberg/proof_system/composer/composer_helper_lib.hpp"
#include "barretenberg/proof_system/composer/permutation_helper.hpp"

namespace proof_system::honk {

/**
 * @brief Compute witness polynomials
 *
 */
void UltraHonkComposerHelper::compute_witness(CircuitConstructor& circuit_constructor)
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

    const size_t subgroup_size = circuit_constructor.get_circuit_subgroup_size(total_num_gates + NUM_RANDOMIZED_GATES);

    // Pad the wires (pointers to `witness_indices` of the `variables` vector).
    // Note: the remaining NUM_RESERVED_GATES indices are padded with zeros within `construct_wire_polynomials_base`
    // (called next).
    for (size_t i = filled_gates; i < total_num_gates; ++i) {
        circuit_constructor.w_l.emplace_back(circuit_constructor.zero_idx);
        circuit_constructor.w_r.emplace_back(circuit_constructor.zero_idx);
        circuit_constructor.w_o.emplace_back(circuit_constructor.zero_idx);
        circuit_constructor.w_4.emplace_back(circuit_constructor.zero_idx);
    }

    // TODO(#340)(luke): within construct_wire_polynomials_base, the 3rd argument is used in the calculation of the
    // dyadic circuit size (subgroup_size). Here (and in other split composers) we're passing in NUM_RANDOMIZED_GATES,
    // but elsewhere, e.g. directly above, we use NUM_RESERVED_GATES in a similar role. Therefore, these two constants
    // must be equal for everything to be consistent. What we should do is compute the dyadic circuit size once and for
    // all then pass that around rather than computing in multiple places.
    auto wire_polynomials =
        construct_wire_polynomials_base<Flavor>(circuit_constructor, total_num_gates, NUM_RANDOMIZED_GATES);

    proving_key->w_l = wire_polynomials[0];
    proving_key->w_r = wire_polynomials[1];
    proving_key->w_o = wire_polynomials[2];
    proving_key->w_4 = wire_polynomials[3];

    polynomial s_1(subgroup_size);
    polynomial s_2(subgroup_size);
    polynomial s_3(subgroup_size);
    polynomial s_4(subgroup_size);
    // TODO(luke): The +1 size for z_lookup is not necessary and can lead to confusion. Resolve.
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

    proving_key->sorted_1 = s_1;
    proving_key->sorted_2 = s_2;
    proving_key->sorted_3 = s_3;
    proving_key->sorted_4 = s_4;

    computed_witness = true;
}

UltraProver UltraHonkComposerHelper::create_prover(CircuitConstructor& circuit_constructor)
{
    finalize_circuit(circuit_constructor);

    compute_proving_key(circuit_constructor);
    compute_witness(circuit_constructor);

    UltraProver output_state(proving_key);

    return output_state;
}

/**
 * Create verifier: compute verification key,
 * initialize verifier with it and an initial manifest and initialize commitment_scheme.
 *
 * @return The verifier.
 * */
UltraVerifier UltraHonkComposerHelper::create_verifier(const CircuitConstructor& circuit_constructor)
{
    auto verification_key = compute_verification_key(circuit_constructor);

    UltraVerifier output_state(verification_key);

    // TODO(Cody): This should be more generic
    auto kate_verification_key = std::make_unique<pcs::kzg::VerificationKey>("../srs_db/ignition");

    output_state.kate_verification_key = std::move(kate_verification_key);

    return output_state;
}

std::shared_ptr<UltraHonkComposerHelper::Flavor::ProvingKey> UltraHonkComposerHelper::compute_proving_key(
    const CircuitConstructor& circuit_constructor)
{
    if (proving_key) {
        return proving_key;
    }

    size_t tables_size = 0;
    size_t lookups_size = 0;
    for (const auto& table : circuit_constructor.lookup_tables) {
        tables_size += table.size;
        lookups_size += table.lookup_gates.size();
    }

    const size_t minimum_circuit_size = tables_size + lookups_size;
    const size_t num_randomized_gates = NUM_RANDOMIZED_GATES;
    // Initialize proving_key
    // TODO(#392)(Kesha): replace composer types.
    proving_key = initialize_proving_key<Flavor>(
        circuit_constructor, crs_factory_.get(), minimum_circuit_size, num_randomized_gates, ComposerType::PLOOKUP);

    construct_selector_polynomials<Flavor>(circuit_constructor, proving_key.get());

    // TODO(#217)(luke): Naively enforcing non-zero selectors for Honk will result in some relations not being
    // satisfied.
    // enforce_nonzero_polynomial_selectors(circuit_constructor, proving_key.get());

    compute_honk_generalized_sigma_permutations<Flavor>(circuit_constructor, proving_key.get());

    compute_first_and_last_lagrange_polynomials<Flavor>(proving_key.get());

    const size_t subgroup_size = proving_key->circuit_size;

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

    proving_key->table_1 = poly_q_table_column_1;
    proving_key->table_2 = poly_q_table_column_2;
    proving_key->table_3 = poly_q_table_column_3;
    proving_key->table_4 = poly_q_table_column_4;

    // Copy memory read/write record data into proving key. Prover needs to know which gates contain a read/write
    // 'record' witness on the 4th wire. This wire value can only be fully computed once the first 3 wire polynomials
    // have been committed to. The 4th wire on these gates will be a random linear combination of the first 3 wires,
    // using the plookup challenge `eta`
    std::copy(circuit_constructor.memory_read_records.begin(),
              circuit_constructor.memory_read_records.end(),
              std::back_inserter(proving_key->memory_read_records));
    std::copy(circuit_constructor.memory_write_records.begin(),
              circuit_constructor.memory_write_records.end(),
              std::back_inserter(proving_key->memory_write_records));

    proving_key->recursive_proof_public_input_indices =
        std::vector<uint32_t>(recursive_proof_public_input_indices.begin(), recursive_proof_public_input_indices.end());

    proving_key->contains_recursive_proof = contains_recursive_proof;

    return proving_key;
}

/**
 * Compute verification key consisting of selector precommitments.
 *
 * @return Pointer to created circuit verification key.
 * */
std::shared_ptr<UltraHonkComposerHelper::VerificationKey> UltraHonkComposerHelper::compute_verification_key(
    const CircuitConstructor& circuit_constructor)
{
    if (verification_key) {
        return verification_key;
    }

    if (!proving_key) {
        compute_proving_key(circuit_constructor);
    }

    verification_key = std::make_shared<UltraHonkComposerHelper::VerificationKey>(proving_key->circuit_size,
                                                                                  proving_key->num_public_inputs,
                                                                                  crs_factory_->get_verifier_crs(),
                                                                                  proving_key->composer_type);

    // TODO(kesha): Dirty hack for now. Need to actually make commitment-agnositc
    auto commitment_key = pcs::kzg::CommitmentKey(proving_key->circuit_size, "../srs_db/ignition");

    // Compute and store commitments to all precomputed polynomials
    verification_key->q_m = commitment_key.commit(proving_key->q_m);
    verification_key->q_l = commitment_key.commit(proving_key->q_l);
    verification_key->q_r = commitment_key.commit(proving_key->q_r);
    verification_key->q_o = commitment_key.commit(proving_key->q_o);
    verification_key->q_4 = commitment_key.commit(proving_key->q_4);
    verification_key->q_c = commitment_key.commit(proving_key->q_c);
    verification_key->q_arith = commitment_key.commit(proving_key->q_arith);
    verification_key->q_sort = commitment_key.commit(proving_key->q_sort);
    verification_key->q_elliptic = commitment_key.commit(proving_key->q_elliptic);
    verification_key->q_aux = commitment_key.commit(proving_key->q_aux);
    verification_key->q_lookup = commitment_key.commit(proving_key->q_lookup);
    verification_key->sigma_1 = commitment_key.commit(proving_key->sigma_1);
    verification_key->sigma_2 = commitment_key.commit(proving_key->sigma_2);
    verification_key->sigma_3 = commitment_key.commit(proving_key->sigma_3);
    verification_key->sigma_4 = commitment_key.commit(proving_key->sigma_4);
    verification_key->id_1 = commitment_key.commit(proving_key->id_1);
    verification_key->id_2 = commitment_key.commit(proving_key->id_2);
    verification_key->id_3 = commitment_key.commit(proving_key->id_3);
    verification_key->id_4 = commitment_key.commit(proving_key->id_4);
    verification_key->table_1 = commitment_key.commit(proving_key->table_1);
    verification_key->table_2 = commitment_key.commit(proving_key->table_2);
    verification_key->table_3 = commitment_key.commit(proving_key->table_3);
    verification_key->table_4 = commitment_key.commit(proving_key->table_4);
    verification_key->lagrange_first = commitment_key.commit(proving_key->lagrange_first);
    verification_key->lagrange_last = commitment_key.commit(proving_key->lagrange_last);

    // // See `add_recusrive_proof()` for how this recursive data is assigned.
    // verification_key->recursive_proof_public_input_indices =
    //     std::vector<uint32_t>(recursive_proof_public_input_indices.begin(),
    //     recursive_proof_public_input_indices.end());

    // verification_key->contains_recursive_proof = contains_recursive_proof;

    return verification_key;
}

} // namespace proof_system::honk
