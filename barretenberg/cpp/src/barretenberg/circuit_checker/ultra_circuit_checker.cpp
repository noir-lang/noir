#include "ultra_circuit_checker.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_flavor.hpp"
#include <barretenberg/plonk/proof_system/constants.hpp>
#include <unordered_set>

namespace bb {

template <> auto UltraCircuitChecker::init_empty_values<UltraCircuitBuilder_<UltraArith<bb::fr>>>()
{
    return UltraFlavor::AllValues{};
}

template <> auto UltraCircuitChecker::init_empty_values<MegaCircuitBuilder_<bb::fr>>()
{
    return MegaFlavor::AllValues{};
}

template <typename Builder> bool UltraCircuitChecker::check(const Builder& builder_in)
{
    // Create a copy of the input circuit and finalize it
    Builder builder{ builder_in };
    builder.finalize_circuit();

    // Construct a hash table for lookup table entries to efficiently determine if a lookup gate is valid
    LookupHashTable lookup_hash_table;
    for (const auto& table : builder.lookup_tables) {
        const FF table_index(table.table_index);
        for (size_t i = 0; i < table.size(); ++i) {
            lookup_hash_table.insert({ table.column_1[i], table.column_2[i], table.column_3[i], table_index });
        }
    }

    // Instantiate structs used for checking tag and memory record correctness
    TagCheckData tag_data;
    MemoryCheckData memory_data{ builder };

    bool result = true;
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/870): Currently we check all relations for each block.
    // Once sorting is complete, is will be sufficient to check only the relevant relation(s) per block.
    size_t block_idx = 0;
    for (auto& block : builder.blocks.get()) {
        result = result && check_block(builder, block, tag_data, memory_data, lookup_hash_table);
        if (!result) {
            info("Failed at block idx = ", block_idx);
            return false;
        }
        block_idx++;
    }

    // Tag check is only expected to pass after entire execution trace (all blocks) have been processed
    result = result && check_tag_data(tag_data);
    if (!result) {
        info("Failed tag check.");
        return false;
    }

    return result;
};

template <typename Builder>
bool UltraCircuitChecker::check_block(Builder& builder,
                                      auto& block,
                                      TagCheckData& tag_data,
                                      MemoryCheckData& memory_data,
                                      LookupHashTable& lookup_hash_table)
{
    // Initialize empty AllValues of the correct Flavor based on Builder type; for input to Relation::accumulate
    auto values = init_empty_values<Builder>();
    Params params;
    params.eta = memory_data.eta; // used in Auxiliary relation for RAM/ROM consistency
    params.eta_two = memory_data.eta_two;
    params.eta_three = memory_data.eta_three;

    auto report_fail = [&](const char* message, size_t row_idx) {
        info(message, row_idx);
#ifdef CHECK_CIRCUIT_STACKTRACES
        block.stack_traces.print(row_idx);
#endif
        return false;
    };

    // Perform checks on each gate defined in the builder
    bool result = true;
    for (size_t idx = 0; idx < block.size(); ++idx) {

        populate_values(builder, block, values, tag_data, memory_data, idx);

        result = result && check_relation<Arithmetic>(values, params);
        if (!result) {
            return report_fail("Failed Arithmetic relation at row idx = ", idx);
        }
        result = result && check_relation<Elliptic>(values, params);
        if (!result) {
            return report_fail("Failed Elliptic relation at row idx = ", idx);
        }
        result = result && check_relation<Auxiliary>(values, params);
        if (!result) {
            return report_fail("Failed Auxiliary relation at row idx = ", idx);
        }
        result = result && check_relation<DeltaRangeConstraint>(values, params);
        if (!result) {
            return report_fail("Failed DeltaRangeConstraint relation at row idx = ", idx);
        }
        result = result && check_lookup(values, lookup_hash_table);
        if (!result) {
            return report_fail("Failed Lookup check relation at row idx = ", idx);
        }
        if constexpr (IsMegaBuilder<Builder>) {
            result = result && check_relation<PoseidonInternal>(values, params);
            if (!result) {
                return report_fail("Failed PoseidonInternal relation at row idx = ", idx);
            }
            result = result && check_relation<PoseidonExternal>(values, params);
            if (!result) {
                return report_fail("Failed PoseidonExternal relation at row idx = ", idx);
            }
            result = result && check_databus_read(values, builder);
            if (!result) {
                return report_fail("Failed databus read at row idx = ", idx);
            }
        }
        if (!result) {
            return report_fail("Failed at row idx = ", idx);
        }
    }

    return result;
};

template <typename Relation> bool UltraCircuitChecker::check_relation(auto& values, auto& params)
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

bool UltraCircuitChecker::check_lookup(auto& values, auto& lookup_hash_table)
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

template <typename Builder> bool UltraCircuitChecker::check_databus_read(auto& values, Builder& builder)
{
    if (!values.q_busread.is_zero()) {
        // Extract the {index, value} pair from the read gate inputs
        auto raw_read_idx = static_cast<size_t>(uint256_t(values.w_r));
        auto value = values.w_l;

        // Determine the type of read based on selector values
        bool is_calldata_read = (values.q_l == 1);
        bool is_secondary_calldata_read = (values.q_r == 1);
        bool is_return_data_read = (values.q_o == 1);
        ASSERT(is_calldata_read || is_secondary_calldata_read || is_return_data_read);

        // Check that the claimed value is present in the calldata/return data at the corresponding index
        FF bus_value;
        if (is_calldata_read) {
            auto calldata = builder.get_calldata();
            bus_value = builder.get_variable(calldata[raw_read_idx]);
        }
        if (is_secondary_calldata_read) {
            auto secondary_calldata = builder.get_secondary_calldata();
            bus_value = builder.get_variable(secondary_calldata[raw_read_idx]);
        }
        if (is_return_data_read) {
            auto return_data = builder.get_return_data();
            bus_value = builder.get_variable(return_data[raw_read_idx]);
        }
        return (value == bus_value);
    }
    return true;
};

bool UltraCircuitChecker::check_tag_data(const TagCheckData& tag_data)
{
    return tag_data.left_product == tag_data.right_product;
};

template <typename Builder>
void UltraCircuitChecker::populate_values(
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

    // A lambda function for computing a memory record term of the form w3 * eta_three + w2 * eta_two + w1 * eta
    auto compute_memory_record_term =
        [](const FF& w_1, const FF& w_2, const FF& w_3, const FF& eta, const FF& eta_two, FF& eta_three) {
            return (w_3 * eta_three + w_2 * eta_two + w_1 * eta);
        };

    // Set wire values. Wire 4 is treated specially since it may contain memory records
    values.w_l = builder.get_variable(block.w_l()[idx]);
    values.w_r = builder.get_variable(block.w_r()[idx]);
    values.w_o = builder.get_variable(block.w_o()[idx]);
    // Note: memory_data contains indices into the block to which RAM/ROM gates were added so we need to check that
    // we are indexing into the correct block before updating the w_4 value.
    if (block.has_ram_rom && memory_data.read_record_gates.contains(idx)) {
        values.w_4 = compute_memory_record_term(
            values.w_l, values.w_r, values.w_o, memory_data.eta, memory_data.eta_two, memory_data.eta_three);
    } else if (block.has_ram_rom && memory_data.write_record_gates.contains(idx)) {
        values.w_4 =
            compute_memory_record_term(
                values.w_l, values.w_r, values.w_o, memory_data.eta, memory_data.eta_two, memory_data.eta_three) +
            FF::one();
    } else {
        values.w_4 = builder.get_variable(block.w_4()[idx]);
    }

    // Set shifted wire values. Again, wire 4 is treated specially. On final row, set shift values to zero
    if (idx < block.size() - 1) {
        values.w_l_shift = builder.get_variable(block.w_l()[idx + 1]);
        values.w_r_shift = builder.get_variable(block.w_r()[idx + 1]);
        values.w_o_shift = builder.get_variable(block.w_o()[idx + 1]);
        if (block.has_ram_rom && memory_data.read_record_gates.contains(idx + 1)) {
            values.w_4_shift = compute_memory_record_term(values.w_l_shift,
                                                          values.w_r_shift,
                                                          values.w_o_shift,
                                                          memory_data.eta,
                                                          memory_data.eta_two,
                                                          memory_data.eta_three);
        } else if (block.has_ram_rom && memory_data.write_record_gates.contains(idx + 1)) {
            values.w_4_shift = compute_memory_record_term(values.w_l_shift,
                                                          values.w_r_shift,
                                                          values.w_o_shift,
                                                          memory_data.eta,
                                                          memory_data.eta_two,
                                                          memory_data.eta_three) +
                               FF::one();
        } else {
            values.w_4_shift = builder.get_variable(block.w_4()[idx + 1]);
        }
    } else {
        values.w_l_shift = 0;
        values.w_r_shift = 0;
        values.w_o_shift = 0;
        values.w_4_shift = 0;
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
    values.q_delta_range = block.q_delta_range()[idx];
    values.q_elliptic = block.q_elliptic()[idx];
    values.q_aux = block.q_aux()[idx];
    values.q_lookup = block.q_lookup_type()[idx];
    if constexpr (IsMegaBuilder<Builder>) {
        values.q_busread = block.q_busread()[idx];
        values.q_poseidon2_internal = block.q_poseidon2_internal()[idx];
        values.q_poseidon2_external = block.q_poseidon2_external()[idx];
    }
}

// Template method instantiations for each check method
template bool UltraCircuitChecker::check<UltraCircuitBuilder_<UltraArith<bb::fr>>>(
    const UltraCircuitBuilder_<UltraArith<bb::fr>>& builder_in);
template bool UltraCircuitChecker::check<MegaCircuitBuilder_<bb::fr>>(const MegaCircuitBuilder_<bb::fr>& builder_in);
} // namespace bb
