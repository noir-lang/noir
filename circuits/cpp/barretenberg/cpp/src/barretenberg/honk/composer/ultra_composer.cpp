#include "ultra_composer.hpp"
#include "barretenberg/honk/proof_system/ultra_prover.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"
#include "barretenberg/proof_system/composer/composer_lib.hpp"
#include "barretenberg/proof_system/composer/permutation_lib.hpp"

namespace proof_system::honk {

/**
 * @brief Helper method to compute quantities like total number of gates and dyadic circuit size
 *
 * @tparam Flavor
 * @param circuit_constructor
 */
template <UltraFlavor Flavor>
void UltraComposer_<Flavor>::compute_circuit_size_parameters(CircuitBuilder& circuit_constructor)
{
    // Compute total length of the tables and the number of lookup gates; their sum is the minimum circuit size
    for (const auto& table : circuit_constructor.lookup_tables) {
        tables_size += table.size;
        lookups_size += table.lookup_gates.size();
    }

    // Get num conventional gates, num public inputs and num Goblin style ECC op gates
    const size_t num_gates = circuit_constructor.num_gates;
    num_public_inputs = circuit_constructor.public_inputs.size();
    num_ecc_op_gates = circuit_constructor.num_ecc_op_gates;

    // minimum circuit size due to the length of lookups plus tables
    const size_t minimum_circuit_size_due_to_lookups = tables_size + lookups_size + num_zero_rows;

    // number of populated rows in the execution trace
    size_t num_rows_populated_in_execution_trace = num_zero_rows + num_ecc_op_gates + num_public_inputs + num_gates;

    // The number of gates is max(lookup gates + tables, rows already populated in trace) + 1, where the +1 is due to
    // addition of a "zero row" at top of the execution trace to ensure wires and other polys are shiftable.
    total_num_gates = std::max(minimum_circuit_size_due_to_lookups, num_rows_populated_in_execution_trace);

    // Next power of 2
    dyadic_circuit_size = circuit_constructor.get_circuit_subgroup_size(total_num_gates);
}

/**
 * @brief Compute witness polynomials
 *
 */
template <UltraFlavor Flavor> void UltraComposer_<Flavor>::compute_witness(CircuitBuilder& circuit_constructor)
{
    if (computed_witness) {
        return;
    }

    // Construct the conventional wire polynomials
    auto wire_polynomials = construct_wire_polynomials_base<Flavor>(circuit_constructor, dyadic_circuit_size);

    proving_key->w_l = wire_polynomials[0];
    proving_key->w_r = wire_polynomials[1];
    proving_key->w_o = wire_polynomials[2];
    proving_key->w_4 = wire_polynomials[3];

    // If Goblin, construct the ECC op queue wire polynomials
    if constexpr (IsGoblinFlavor<Flavor>) {
        construct_ecc_op_wire_polynomials(wire_polynomials);
    }

    // Construct the sorted concatenated list polynomials for the lookup argument
    polynomial s_1(dyadic_circuit_size);
    polynomial s_2(dyadic_circuit_size);
    polynomial s_3(dyadic_circuit_size);
    polynomial s_4(dyadic_circuit_size);

    // The sorted list polynomials have (tables_size + lookups_size) populated entries. We define the index below so
    // that these entries are written into the last indices of the polynomials. The values on the first
    // dyadic_circuit_size - (tables_size + lookups_size) indices are automatically initialized to zero via the
    // polynomial constructor.
    size_t s_index = dyadic_circuit_size - tables_size - lookups_size;
    ASSERT(s_index > 0); // We need at least 1 row of zeroes for the permutation argument

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
            s_1[s_index] = components[0];
            s_2[s_index] = components[1];
            s_3[s_index] = components[2];
            s_4[s_index] = table_index;
            ++s_index;
        }
    }

    // Polynomial memory is zeroed out when constructed with size hint, so we don't have to initialize trailing space
    proving_key->sorted_1 = s_1;
    proving_key->sorted_2 = s_2;
    proving_key->sorted_3 = s_3;
    proving_key->sorted_4 = s_4;

    // Copy memory read/write record data into proving key. Prover needs to know which gates contain a read/write
    // 'record' witness on the 4th wire. This wire value can only be fully computed once the first 3 wire polynomials
    // have been committed to. The 4th wire on these gates will be a random linear combination of the first 3 wires,
    // using the plookup challenge `eta`. We need to update the records with an offset Because we shift the gates to
    // account for everything that comes before them in the execution trace, e.g. public inputs, a zero row, etc.
    size_t offset = num_ecc_op_gates + num_public_inputs + num_zero_rows;
    auto add_public_inputs_offset = [offset](uint32_t gate_index) { return gate_index + offset; };
    proving_key->memory_read_records = std::vector<uint32_t>();
    proving_key->memory_write_records = std::vector<uint32_t>();

    std::transform(circuit_constructor.memory_read_records.begin(),
                   circuit_constructor.memory_read_records.end(),
                   std::back_inserter(proving_key->memory_read_records),
                   add_public_inputs_offset);
    std::transform(circuit_constructor.memory_write_records.begin(),
                   circuit_constructor.memory_write_records.end(),
                   std::back_inserter(proving_key->memory_write_records),
                   add_public_inputs_offset);

    computed_witness = true;
}

/**
 * @brief Construct Goblin style ECC op wire polynomials
 * @details The Ecc op wire values are assumed to have already been stored in the corresponding block of the
 * conventional wire polynomials. The values for the ecc op wire polynomials are set based on those values.
 *
 * @tparam Flavor
 * @param wire_polynomials
 */
template <UltraFlavor Flavor> void UltraComposer_<Flavor>::construct_ecc_op_wire_polynomials(auto& wire_polynomials)
{
    std::array<polynomial, Flavor::NUM_WIRES> op_wire_polynomials;
    for (auto& poly : op_wire_polynomials) {
        poly = polynomial(dyadic_circuit_size);
    }

    // The ECC op wires are constructed to contain the op data on the appropriate range and to vanish everywhere else.
    // The op data is assumed to have already been stored at the correct location in the convetional wires so the data
    // can simply be copied over directly.
    const size_t op_wire_offset = Flavor::has_zero_row ? 1 : 0;
    for (size_t poly_idx = 0; poly_idx < Flavor::NUM_WIRES; ++poly_idx) {
        for (size_t i = 0; i < num_ecc_op_gates; ++i) {
            size_t idx = i + op_wire_offset;
            op_wire_polynomials[poly_idx][idx] = wire_polynomials[poly_idx][idx];
        }
    }

    proving_key->ecc_op_wire_1 = op_wire_polynomials[0];
    proving_key->ecc_op_wire_2 = op_wire_polynomials[1];
    proving_key->ecc_op_wire_3 = op_wire_polynomials[2];
    proving_key->ecc_op_wire_4 = op_wire_polynomials[3];
}

template <UltraFlavor Flavor>
UltraProver_<Flavor> UltraComposer_<Flavor>::create_prover(CircuitBuilder& circuit_constructor)
{
    circuit_constructor.add_gates_to_ensure_all_polys_are_non_zero();
    circuit_constructor.finalize_circuit();

    // Compute total number of gates, dyadic circuit size, etc.
    compute_circuit_size_parameters(circuit_constructor);

    compute_proving_key(circuit_constructor);
    compute_witness(circuit_constructor);

    compute_commitment_key(proving_key->circuit_size);

    UltraProver_<Flavor> output_state(proving_key, commitment_key);

    return output_state;
}

/**
 * Create verifier: compute verification key,
 * initialize verifier with it and an initial manifest and initialize commitment_scheme.
 *
 * @return The verifier.
 * */
template <UltraFlavor Flavor>
UltraVerifier_<Flavor> UltraComposer_<Flavor>::create_verifier(const CircuitBuilder& circuit_constructor)
{
    auto verification_key = compute_verification_key(circuit_constructor);

    UltraVerifier_<Flavor> output_state(verification_key);

    auto pcs_verification_key = std::make_unique<PCSVerificationKey>(verification_key->circuit_size, crs_factory_);

    output_state.pcs_verification_key = std::move(pcs_verification_key);

    return output_state;
}

template <UltraFlavor Flavor>
std::shared_ptr<typename Flavor::ProvingKey> UltraComposer_<Flavor>::compute_proving_key(
    const CircuitBuilder& circuit_constructor)
{
    if (proving_key) {
        return proving_key;
    }

    proving_key = std::make_shared<ProvingKey>(dyadic_circuit_size, num_public_inputs);

    construct_selector_polynomials<Flavor>(circuit_constructor, proving_key.get());

    compute_honk_generalized_sigma_permutations<Flavor>(circuit_constructor, proving_key.get());

    compute_first_and_last_lagrange_polynomials<Flavor>(proving_key.get());

    polynomial poly_q_table_column_1(dyadic_circuit_size);
    polynomial poly_q_table_column_2(dyadic_circuit_size);
    polynomial poly_q_table_column_3(dyadic_circuit_size);
    polynomial poly_q_table_column_4(dyadic_circuit_size);

    size_t offset = dyadic_circuit_size - tables_size;

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

    // Polynomial memory is zeroed out when constructed with size hint, so we don't have to initialize trailing space

    proving_key->table_1 = poly_q_table_column_1;
    proving_key->table_2 = poly_q_table_column_2;
    proving_key->table_3 = poly_q_table_column_3;
    proving_key->table_4 = poly_q_table_column_4;

    proving_key->recursive_proof_public_input_indices =
        std::vector<uint32_t>(recursive_proof_public_input_indices.begin(), recursive_proof_public_input_indices.end());

    proving_key->contains_recursive_proof = contains_recursive_proof;

    if constexpr (IsGoblinFlavor<Flavor>) {
        proving_key->num_ecc_op_gates = num_ecc_op_gates;
    }

    return proving_key;
}

/**
 * Compute verification key consisting of selector precommitments.
 *
 * @return Pointer to created circuit verification key.
 * */
template <UltraFlavor Flavor>
std::shared_ptr<typename Flavor::VerificationKey> UltraComposer_<Flavor>::compute_verification_key(
    const CircuitBuilder& circuit_constructor)
{
    if (verification_key) {
        return verification_key;
    }

    if (!proving_key) {
        compute_proving_key(circuit_constructor);
    }

    verification_key =
        std::make_shared<typename Flavor::VerificationKey>(proving_key->circuit_size, proving_key->num_public_inputs);

    // Compute and store commitments to all precomputed polynomials
    verification_key->q_m = commitment_key->commit(proving_key->q_m);
    verification_key->q_l = commitment_key->commit(proving_key->q_l);
    verification_key->q_r = commitment_key->commit(proving_key->q_r);
    verification_key->q_o = commitment_key->commit(proving_key->q_o);
    verification_key->q_4 = commitment_key->commit(proving_key->q_4);
    verification_key->q_c = commitment_key->commit(proving_key->q_c);
    verification_key->q_arith = commitment_key->commit(proving_key->q_arith);
    verification_key->q_sort = commitment_key->commit(proving_key->q_sort);
    verification_key->q_elliptic = commitment_key->commit(proving_key->q_elliptic);
    verification_key->q_aux = commitment_key->commit(proving_key->q_aux);
    verification_key->q_lookup = commitment_key->commit(proving_key->q_lookup);
    verification_key->sigma_1 = commitment_key->commit(proving_key->sigma_1);
    verification_key->sigma_2 = commitment_key->commit(proving_key->sigma_2);
    verification_key->sigma_3 = commitment_key->commit(proving_key->sigma_3);
    verification_key->sigma_4 = commitment_key->commit(proving_key->sigma_4);
    verification_key->id_1 = commitment_key->commit(proving_key->id_1);
    verification_key->id_2 = commitment_key->commit(proving_key->id_2);
    verification_key->id_3 = commitment_key->commit(proving_key->id_3);
    verification_key->id_4 = commitment_key->commit(proving_key->id_4);
    verification_key->table_1 = commitment_key->commit(proving_key->table_1);
    verification_key->table_2 = commitment_key->commit(proving_key->table_2);
    verification_key->table_3 = commitment_key->commit(proving_key->table_3);
    verification_key->table_4 = commitment_key->commit(proving_key->table_4);
    verification_key->lagrange_first = commitment_key->commit(proving_key->lagrange_first);
    verification_key->lagrange_last = commitment_key->commit(proving_key->lagrange_last);

    // TODO(luke): Similar to the lagrange_first/last polynomials, we dont really need to commit to this polynomial due
    // to its simple structure. Handling it in the same way as the lagrange polys for now for simplicity.
    if constexpr (IsGoblinFlavor<Flavor>) {
        verification_key->lagrange_ecc_op = commitment_key->commit(proving_key->lagrange_ecc_op);
    }

    // // See `add_recusrive_proof()` for how this recursive data is assigned.
    // verification_key->recursive_proof_public_input_indices =
    //     std::vector<uint32_t>(recursive_proof_public_input_indices.begin(),
    //     recursive_proof_public_input_indices.end());

    // verification_key->contains_recursive_proof = contains_recursive_proof;

    return verification_key;
}
template class UltraComposer_<honk::flavor::Ultra>;
template class UltraComposer_<honk::flavor::UltraGrumpkin>;
template class UltraComposer_<honk::flavor::GoblinUltra>;

} // namespace proof_system::honk
