#include "barretenberg/vm/avm/trace/gas_trace.hpp"

#include <cstddef>
#include <cstdint>

#include "barretenberg/vm/avm/trace/fixed_gas.hpp"
#include "barretenberg/vm/avm/trace/opcode.hpp"

namespace bb::avm_trace {

void AvmGasTraceBuilder::reset()
{
    gas_trace.clear();
}

void AvmGasTraceBuilder::set_initial_gas(uint32_t l2_gas, uint32_t da_gas)
{
    initial_l2_gas = l2_gas;
    initial_da_gas = da_gas;

    // Remaining gas will be mutated on each opcode
    remaining_l2_gas = l2_gas;
    remaining_da_gas = da_gas;
}

uint32_t AvmGasTraceBuilder::get_l2_gas_left() const
{
    return gas_trace.back().remaining_l2_gas;
}

uint32_t AvmGasTraceBuilder::get_da_gas_left() const
{
    return gas_trace.back().remaining_da_gas;
}

void AvmGasTraceBuilder::constrain_gas(uint32_t clk, OpCode opcode, uint32_t dyn_gas_multiplier)
{
    gas_opcode_lookup_counter[opcode]++;

    // Get the gas prices for this opcode
    const auto& GAS_COST_TABLE = FixedGasTable::get();
    const auto& gas_info = GAS_COST_TABLE.at(opcode);
    auto base_l2_gas_cost = static_cast<uint32_t>(gas_info.base_l2_gas_fixed_table);
    auto base_da_gas_cost = static_cast<uint32_t>(gas_info.base_da_gas_fixed_table);
    auto dyn_l2_gas_cost = static_cast<uint32_t>(gas_info.dyn_l2_gas_fixed_table);
    auto dyn_da_gas_cost = static_cast<uint32_t>(gas_info.dyn_da_gas_fixed_table);

    // Decrease the gas left
    remaining_l2_gas -= base_l2_gas_cost + dyn_l2_gas_cost * dyn_gas_multiplier;
    remaining_da_gas -= base_da_gas_cost + dyn_da_gas_cost * dyn_gas_multiplier;

    // Create a gas trace entry
    gas_trace.push_back({
        .clk = clk,
        .opcode = opcode,
        .base_l2_gas_cost = base_l2_gas_cost,
        .base_da_gas_cost = base_da_gas_cost,
        .dyn_l2_gas_cost = dyn_l2_gas_cost,
        .dyn_da_gas_cost = dyn_da_gas_cost,
        .dyn_gas_multiplier = dyn_gas_multiplier,
        .remaining_l2_gas = remaining_l2_gas,
        .remaining_da_gas = remaining_da_gas,
    });
}

void AvmGasTraceBuilder::constrain_gas_for_external_call(uint32_t clk,
                                                         uint32_t dyn_gas_multiplier,
                                                         uint32_t nested_l2_gas_cost,
                                                         uint32_t nested_da_gas_cost)
{
    const OpCode opcode = OpCode::CALL;
    gas_opcode_lookup_counter[opcode]++;

    // Get the gas prices for this opcode
    const auto& GAS_COST_TABLE = FixedGasTable::get();
    const auto& gas_info = GAS_COST_TABLE.at(opcode);
    auto base_l2_gas_cost = static_cast<uint32_t>(gas_info.base_l2_gas_fixed_table);
    auto base_da_gas_cost = static_cast<uint32_t>(gas_info.base_da_gas_fixed_table);
    auto dyn_l2_gas_cost = static_cast<uint32_t>(gas_info.dyn_l2_gas_fixed_table);
    auto dyn_da_gas_cost = static_cast<uint32_t>(gas_info.dyn_da_gas_fixed_table);

    // TODO: this is the only difference, unify.
    // Decrease the gas left
    remaining_l2_gas -= (base_l2_gas_cost + dyn_gas_multiplier * dyn_l2_gas_cost) + nested_l2_gas_cost;
    remaining_da_gas -= (base_da_gas_cost + dyn_gas_multiplier * dyn_da_gas_cost) + nested_da_gas_cost;

    // Create a gas trace entry
    gas_trace.push_back({
        .clk = clk,
        .opcode = opcode,
        .base_l2_gas_cost = base_l2_gas_cost,
        .base_da_gas_cost = base_da_gas_cost,
        .dyn_l2_gas_cost = dyn_l2_gas_cost,
        .dyn_da_gas_cost = dyn_da_gas_cost,
        .dyn_gas_multiplier = dyn_gas_multiplier,
        .remaining_l2_gas = remaining_l2_gas,
        .remaining_da_gas = remaining_da_gas,
    });
}

void AvmGasTraceBuilder::finalize(std::vector<AvmFullRow<FF>>& main_trace)
{
    // Add the gas cost table to the main trace
    // TODO: do i need a way to produce an interupt that will stop the execution of the trace when the gas left
    // becomes zero in the gas_trace_builder Does all of the gas trace information need to be added to this main
    // machine?????

    // Add the gas accounting for each row
    // We can assume that the gas trace will never be larger than the main trace
    // We infer that a row is active for gas (.main_gas_cost_active = 1) based on the presence
    // of a gas entry row.
    // Set the initial gas
    auto& first_opcode_row = main_trace.at(0);
    first_opcode_row.main_l2_gas_remaining = initial_l2_gas;
    first_opcode_row.main_da_gas_remaining = initial_da_gas;

    uint32_t current_clk = 1;
    uint32_t current_l2_gas_remaining = initial_l2_gas;
    uint32_t current_da_gas_remaining = initial_da_gas;

    // Assume that gas_trace entries are ordered by a strictly increasing clk sequence.
    for (auto const& gas_entry : gas_trace) {
        // There should be no gaps in the gas_trace.
        ASSERT(gas_entry.clk == current_clk);

        auto& dest = main_trace.at(gas_entry.clk - 1);
        auto& next = main_trace.at(gas_entry.clk);

        // Write each of the relevant gas accounting values
        dest.main_opcode_val = static_cast<uint8_t>(gas_entry.opcode);
        dest.main_base_l2_gas_op_cost = gas_entry.base_l2_gas_cost;
        dest.main_base_da_gas_op_cost = gas_entry.base_da_gas_cost;
        dest.main_dyn_l2_gas_op_cost = gas_entry.dyn_l2_gas_cost;
        dest.main_dyn_da_gas_op_cost = gas_entry.dyn_da_gas_cost;
        dest.main_dyn_gas_multiplier = gas_entry.dyn_gas_multiplier;

        // If gas remaining is increasing, it means we underflowed in uint32_t
        bool l2_out_of_gas = current_l2_gas_remaining < gas_entry.remaining_l2_gas;
        bool da_out_of_gas = current_da_gas_remaining < gas_entry.remaining_da_gas;

        uint32_t abs_l2_gas_remaining = l2_out_of_gas ? -gas_entry.remaining_l2_gas : gas_entry.remaining_l2_gas;
        uint32_t abs_da_gas_remaining = da_out_of_gas ? -gas_entry.remaining_da_gas : gas_entry.remaining_da_gas;

        dest.main_abs_l2_rem_gas = abs_l2_gas_remaining;
        dest.main_abs_da_rem_gas = abs_da_gas_remaining;

        dest.main_l2_out_of_gas = static_cast<uint32_t>(l2_out_of_gas);
        dest.main_da_out_of_gas = static_cast<uint32_t>(da_out_of_gas);

        current_l2_gas_remaining = gas_entry.remaining_l2_gas;
        current_da_gas_remaining = gas_entry.remaining_da_gas;
        next.main_l2_gas_remaining =
            l2_out_of_gas ? FF::modulus - uint256_t(abs_l2_gas_remaining) : current_l2_gas_remaining;
        next.main_da_gas_remaining =
            da_out_of_gas ? FF::modulus - uint256_t(abs_da_gas_remaining) : current_da_gas_remaining;

        current_clk++;
    }

    reset();
}

void AvmGasTraceBuilder::finalize_lookups(std::vector<AvmFullRow<FF>>& main_trace)
{
    // Finalise gas left lookup counts
    // TODO: find the right place for this. This is not really over the main trace, but over the opcode trace.
    for (auto const& [opcode, count] : gas_opcode_lookup_counter) {
        main_trace.at(static_cast<uint8_t>(opcode)).lookup_opcode_gas_counts = count;
    }
}

} // namespace bb::avm_trace
