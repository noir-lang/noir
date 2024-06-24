#pragma once

#include <cstdint>

#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/avm_trace/avm_opcode.hpp"

namespace bb::avm_trace {

class AvmGasTraceBuilder {
  public:
    struct GasTraceEntry {
        uint32_t clk = 0;
        OpCode opcode;
        uint32_t l2_gas_cost = 0;
        uint32_t da_gas_cost = 0;
        uint32_t remaining_l2_gas = 0;
        uint32_t remaining_da_gas = 0;
    };

    // Counts each time an opcode is read
    // opcode -> count
    std::unordered_map<OpCode, uint32_t> gas_opcode_lookup_counter;

    AvmGasTraceBuilder() = default;

    void reset();
    std::vector<GasTraceEntry> finalize();

    void constrain_gas_lookup(uint32_t clk, OpCode opcode);
    void constrain_gas_for_external_call(uint32_t clk, uint32_t nested_l2_gas_cost, uint32_t nested_da_gas_cost);
    void set_initial_gas(uint32_t l2_gas, uint32_t da_gas);

    uint32_t get_l2_gas_left();
    uint32_t get_da_gas_left();

    std::vector<GasTraceEntry> gas_trace;

    uint32_t initial_l2_gas = 0;
    uint32_t initial_da_gas = 0;

  private:
    uint32_t remaining_l2_gas = 0;
    uint32_t remaining_da_gas = 0;
};

} // namespace bb::avm_trace
