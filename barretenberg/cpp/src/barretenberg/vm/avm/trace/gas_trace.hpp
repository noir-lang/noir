#pragma once

#include <cstdint>
#include <vector>

#include "barretenberg/vm/avm/generated/full_row.hpp"
#include "barretenberg/vm/avm/trace/common.hpp"
#include "barretenberg/vm/avm/trace/opcode.hpp"

namespace bb::avm_trace {

class AvmGasTraceBuilder {
  public:
    struct GasTraceEntry {
        uint32_t clk = 0;
        OpCode opcode;
        uint32_t base_l2_gas_cost = 0;
        uint32_t base_da_gas_cost = 0;
        uint32_t dyn_l2_gas_cost = 0;
        uint32_t dyn_da_gas_cost = 0;
        uint32_t dyn_gas_multiplier = 0;
        uint32_t remaining_l2_gas = 0;
        uint32_t remaining_da_gas = 0;
    };

    AvmGasTraceBuilder() = default;

    size_t size() const { return gas_trace.size(); }
    void reset();
    // These two have to be separate, because the lookup counts have to be
    // finalized after the extra first row gets added.
    void finalize(std::vector<AvmFullRow<FF>>& trace);
    void finalize_lookups(std::vector<AvmFullRow<FF>>& trace);

    void constrain_gas(uint32_t clk, OpCode opcode, uint32_t dyn_gas_multiplier = 0);
    void constrain_gas_for_external_call(uint32_t clk,
                                         uint32_t dyn_gas_multiplier,
                                         uint32_t nested_l2_gas_cost,
                                         uint32_t nested_da_gas_cost);
    void set_initial_gas(uint32_t l2_gas, uint32_t da_gas);

    uint32_t get_l2_gas_left() const;
    uint32_t get_da_gas_left() const;

    // Counts each time an opcode is read: opcode -> count
    std::unordered_map<OpCode, uint32_t> gas_opcode_lookup_counter;
    // Data structure to collect all lookup counts pertaining to 16-bit range checks related to remaining gas
    std::array<std::unordered_map<uint16_t, uint32_t>, 4> rem_gas_rng_check_counts;

  private:
    std::vector<GasTraceEntry> gas_trace;

    uint32_t initial_l2_gas = 0;
    uint32_t initial_da_gas = 0;
    uint32_t remaining_l2_gas = 0;
    uint32_t remaining_da_gas = 0;
};

} // namespace bb::avm_trace
