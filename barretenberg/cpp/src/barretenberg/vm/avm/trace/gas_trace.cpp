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

std::vector<AvmGasTraceBuilder::GasTraceEntry> AvmGasTraceBuilder::finalize()
{
    return std::move(gas_trace);
}

void AvmGasTraceBuilder::set_initial_gas(uint32_t l2_gas, uint32_t da_gas)
{
    initial_l2_gas = l2_gas;
    initial_da_gas = da_gas;

    // Remaining gas will be mutated on each opcode
    remaining_l2_gas = l2_gas;
    remaining_da_gas = da_gas;
}

uint32_t AvmGasTraceBuilder::get_l2_gas_left()
{
    return gas_trace.back().remaining_l2_gas;
}

uint32_t AvmGasTraceBuilder::get_da_gas_left()
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

} // namespace bb::avm_trace