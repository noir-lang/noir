#include "barretenberg/vm/avm_trace/avm_gas_trace.hpp"

#include <cstddef>
#include <cstdint>

#include "barretenberg/vm/avm_trace/avm_opcode.hpp"
#include "barretenberg/vm/avm_trace/fixed_gas.hpp"

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

void AvmGasTraceBuilder::constrain_gas_lookup(uint32_t clk, OpCode opcode)
{
    // TODO: increase lookup counter for the opcode we are looking up into
    gas_opcode_lookup_counter[opcode]++;

    // Get the gas prices for this opcode
    const auto& GAS_COST_TABLE = FixedGasTable::get();
    auto l2_gas_cost = static_cast<uint32_t>(GAS_COST_TABLE.at(opcode).gas_l2_gas_fixed_table);
    auto da_gas_cost = static_cast<uint32_t>(GAS_COST_TABLE.at(opcode).gas_da_gas_fixed_table);

    remaining_l2_gas -= l2_gas_cost;
    remaining_da_gas -= da_gas_cost;

    // Decrease the gas left
    // Create a gas trace entry
    GasTraceEntry entry = {
        .clk = clk,
        .opcode = opcode,
        .l2_gas_cost = l2_gas_cost,
        .da_gas_cost = da_gas_cost,
        .remaining_l2_gas = remaining_l2_gas,
        .remaining_da_gas = remaining_da_gas,
    };

    gas_trace.push_back(entry);
}

void AvmGasTraceBuilder::constrain_gas_for_external_call(uint32_t clk,
                                                         uint32_t nested_l2_gas_cost,
                                                         uint32_t nested_da_gas_cost)
{
    const OpCode opcode = OpCode::CALL;

    // TODO: increase lookup counter for the opcode we are looking up into
    // gas_opcode_lookup_counter[opcode]++;

    // Get the gas prices for this opcode
    const auto& GAS_COST_TABLE = FixedGasTable::get();
    auto opcode_l2_gas_cost = static_cast<uint32_t>(GAS_COST_TABLE.at(opcode).gas_l2_gas_fixed_table);
    auto opcode_da_gas_cost = static_cast<uint32_t>(GAS_COST_TABLE.at(opcode).gas_da_gas_fixed_table);

    remaining_l2_gas -= opcode_l2_gas_cost + nested_l2_gas_cost;
    remaining_da_gas -= opcode_da_gas_cost + nested_da_gas_cost;

    // Decrease the gas left
    // Create a gas trace entry
    GasTraceEntry entry = {
        .clk = clk,
        .opcode = OpCode::CALL,
        .l2_gas_cost = 0, // We need 0 in this case because we do not activate the gas_cost_active selector to satisfy
                          // #[L2_GAS_INACTIVE].
        .da_gas_cost = 0, // We need 0 in this case because we do not activate the gas_cost_active selector to satisfy
                          // #[DA_GAS_INACTIVE].
        .remaining_l2_gas = remaining_l2_gas,
        .remaining_da_gas = remaining_da_gas,
    };

    gas_trace.push_back(entry);
}

} // namespace bb::avm_trace