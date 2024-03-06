#include "circuit_checker.hpp"
#include "barretenberg/flavor/goblin_ultra.hpp"
#include <barretenberg/plonk/proof_system/constants.hpp>
#include <unordered_set>

namespace bb {

template <> auto CircuitChecker::init_empty_values<UltraCircuitBuilder_<UltraArith<bb::fr>>>()
{
    return UltraFlavor::AllValues{};
}

template <> auto CircuitChecker::init_empty_values<GoblinUltraCircuitBuilder_<bb::fr>>()
{
    return GoblinUltraFlavor::AllValues{};
}

template <typename Builder> bool CircuitChecker::check(const Builder& builder_in)
{
    // Create a copy of the input circuit and finalize it
    Builder builder{ builder_in };
    builder.finalize_circuit();

    // Construct a hash table for lookup table entries to efficiently determine if a lookup gate is valid
    LookupHashTable lookup_hash_table;
    for (const auto& table : builder.lookup_tables) {
        const FF table_index(table.table_index);
        for (size_t i = 0; i < table.size; ++i) {
            lookup_hash_table.insert({ table.column_1[i], table.column_2[i], table.column_3[i], table_index });
        }
    }

    // Instantiate structs used for checking tag and memory record correctness
    TagCheckData tag_data;
    MemoryCheckData memory_data{ builder };

    // Initialize empty AllValues of the correct Flavor based on Builder type; for input to Relation::accumulate
    auto values = init_empty_values<Builder>();
    Params params;
    params.eta = memory_data.eta; // used in Auxiliary relation for RAM/ROM consistency

    // TODO(https://github.com/AztecProtocol/barretenberg/issues/867): Once we sort gates into their respective blocks
    // we'll need to either naively run this on all blocks or run only the relevant checks on each block.
    auto& block = builder.blocks.main;

    // Perform checks on each gate defined in the builder
    bool result = true;
    for (size_t idx = 0; idx < block.size(); ++idx) {
        populate_values(builder, block, values, tag_data, memory_data, idx);

        result = result && check_relation<Arithmetic>(values, params);
        result = result && check_relation<Elliptic>(values, params);
        result = result && check_relation<Auxiliary>(values, params);
        result = result && check_relation<GenPermSort>(values, params);
        result = result && check_lookup(values, lookup_hash_table);
        if constexpr (IsGoblinBuilder<Builder>) {
            result = result && check_relation<PoseidonInternal>(values, params);
            result = result && check_relation<PoseidonExternal>(values, params);
        }
    }

    // Tag check is only expected to pass after all gates have been processed
    result = result && check_tag_data(tag_data);

    return result;
};

template <typename Relation> bool CircuitChecker::check_relation(auto& values, auto& params)
{
    // Define zero initialized array to store the evaluation of each sub-relation
    using SubrelationEvaluations = typename Relation::SumcheckArrayOfValuesOverSubrelations;
    SubrelationEvaluations subrelation_evaluations;
    for (auto& eval : subrelation_evaluations) {
        eval = 0;
    }

    // Evaluate each subrelation in the relation
    Relation::accumulate(subrelation_evaluations, values, params, /*scaling_factor=*/1);

    // Ensure each subrelation evaluates to zero
    for (auto& eval : subrelation_evaluations) {
        if (eval != 0) {
            return false;
        }
    }
    return true;
}

bool CircuitChecker::check_lookup(auto& values, auto& lookup_hash_table)
{
    // If this is a lookup gate, check the inputs are in the hash table containing all table entries
    if (!values.q_lookup.is_zero()) {
        return lookup_hash_table.contains({ values.w_l + values.q_r * values.w_l_shift,
                                            values.w_r + values.q_m * values.w_r_shift,
                                            values.w_o + values.q_c * values.w_o_shift,
                                            values.q_o });
    }
    return true;
};

bool CircuitChecker::check_tag_data(const TagCheckData& tag_data)
{
    return tag_data.left_product == tag_data.right_product;
};

template <typename Builder>
void CircuitChecker::populate_values(
    Builder& builder, auto& block, auto& values, TagCheckData& tag_data, MemoryCheckData& memory_data, size_t idx)
{
    // Function to quickly update tag products and encountered variable set by index and value
    auto update_tag_check_data = [&](const size_t variable_index, const FF& value) {
        size_t real_index = builder.real_variable_index[variable_index];
        // Check to ensure that we are not including a variable twice
        if (tag_data.encountered_variables.contains(real_index)) {
            return;
        }
        uint32_t tag_in = builder.real_variable_tags[real_index];
        if (tag_in != DUMMY_TAG) {
            uint32_t tag_out = builder.tau.at(tag_in);
            tag_data.left_product *= value + tag_data.gamma * FF(tag_in);
            tag_data.right_product *= value + tag_data.gamma * FF(tag_out);
            tag_data.encountered_variables.insert(real_index);
        }
    };

    // A lambda function for computing a memory record term of the form w3 * eta^3 + w2 * eta^2 + w1 * eta
    auto compute_memory_record_term = [](const FF& w_1, const FF& w_2, const FF& w_3, const FF& eta) {
        return ((w_3 * eta + w_2) * eta + w_1) * eta;
    };

    // Set wire values. Wire 4 is treated specially since it may contain memory records
    values.w_l = builder.get_variable(block.w_l()[idx]);
    values.w_r = builder.get_variable(block.w_r()[idx]);
    values.w_o = builder.get_variable(block.w_o()[idx]);
    if (memory_data.read_record_gates.contains(idx)) {
        values.w_4 = compute_memory_record_term(values.w_l, values.w_r, values.w_o, memory_data.eta);
    } else if (memory_data.write_record_gates.contains(idx)) {
        values.w_4 = compute_memory_record_term(values.w_l, values.w_r, values.w_o, memory_data.eta) + FF::one();
    } else {
        values.w_4 = builder.get_variable(block.w_4()[idx]);
    }

    // Set shifted wire values. Again, wire 4 is treated specially. On final row, set shift values to zero
    values.w_l_shift = idx < block.size() - 1 ? builder.get_variable(block.w_l()[idx + 1]) : 0;
    values.w_r_shift = idx < block.size() - 1 ? builder.get_variable(block.w_r()[idx + 1]) : 0;
    values.w_o_shift = idx < block.size() - 1 ? builder.get_variable(block.w_o()[idx + 1]) : 0;
    if (memory_data.read_record_gates.contains(idx + 1)) {
        values.w_4_shift =
            compute_memory_record_term(values.w_l_shift, values.w_r_shift, values.w_o_shift, memory_data.eta);
    } else if (memory_data.write_record_gates.contains(idx + 1)) {
        values.w_4_shift =
            compute_memory_record_term(values.w_l_shift, values.w_r_shift, values.w_o_shift, memory_data.eta) +
            FF::one();
    } else {
        values.w_4_shift = idx < block.size() - 1 ? builder.get_variable(block.w_4()[idx + 1]) : 0;
    }

    // Update tag check data
    update_tag_check_data(block.w_l()[idx], values.w_l);
    update_tag_check_data(block.w_r()[idx], values.w_r);
    update_tag_check_data(block.w_o()[idx], values.w_o);
    update_tag_check_data(block.w_4()[idx], values.w_4);

    // Set selector values
    values.q_m = block.q_m()[idx];
    values.q_c = block.q_c()[idx];
    values.q_l = block.q_1()[idx];
    values.q_r = block.q_2()[idx];
    values.q_o = block.q_3()[idx];
    values.q_4 = block.q_4()[idx];
    values.q_arith = block.q_arith()[idx];
    values.q_sort = block.q_sort()[idx];
    values.q_elliptic = block.q_elliptic()[idx];
    values.q_aux = block.q_aux()[idx];
    values.q_lookup = block.q_lookup_type()[idx];
    if constexpr (IsGoblinBuilder<Builder>) {
        values.q_poseidon2_internal = block.q_poseidon2_internal()[idx];
        values.q_poseidon2_external = block.q_poseidon2_external()[idx];
    }
}

// Template method instantiations for each check method
// template bool CircuitChecker::check<bb::fr>(const StandardCircuitBuilder_<bb::fr>& builder);
// template bool CircuitChecker::check<bb::fq>(const StandardCircuitBuilder_<bb::fq>& builder);
template bool CircuitChecker::check<UltraCircuitBuilder_<UltraArith<bb::fr>>>(
    const UltraCircuitBuilder_<UltraArith<bb::fr>>& builder_in);
template bool CircuitChecker::check<GoblinUltraCircuitBuilder_<bb::fr>>(
    const GoblinUltraCircuitBuilder_<bb::fr>& builder_in);

} // namespace bb
