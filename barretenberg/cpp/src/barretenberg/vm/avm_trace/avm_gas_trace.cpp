#include "avm_gas_trace.hpp"

namespace bb::avm_trace {

AvmGasTraceBuilder::AvmGasTraceBuilder() {}

void AvmGasTraceBuilder::reset()
{
    gas_trace.clear();
}

std::vector<AvmGasTraceBuilder::GasTraceEntry> AvmGasTraceBuilder::finalize()
{
    return std::move(gas_trace);
};

void AvmGasTraceBuilder::set_initial_gas(uint32_t l2_gas, uint32_t da_gas)
{
    initial_l2_gas = l2_gas;
    initial_da_gas = da_gas;

    // Remaining gas will be mutated on each opcode
    remaining_l2_gas = l2_gas;
    remaining_da_gas = da_gas;
}

void AvmGasTraceBuilder::constrain_gas_lookup(uint32_t clk, OpCode opcode)
{
    // TODO: increase lookup counter for the opcode we are looking up into
    gas_opcode_lookup_counter[opcode]++;

    // Get the gas prices for this opcode
    uint32_t l2_gas_cost = GAS_COST_TABLE.at(opcode).l2_fixed_gas_cost;
    uint32_t da_gas_cost = GAS_COST_TABLE.at(opcode).da_fixed_gas_cost;

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

} // namespace bb::avm_trace