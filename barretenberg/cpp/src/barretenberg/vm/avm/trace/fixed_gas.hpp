#pragma once

#include <cstddef>
#include <cstdint>

#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/vm/avm/trace/common.hpp"
#include "barretenberg/vm/avm/trace/opcode.hpp"

namespace bb::avm_trace {

class FixedGasTable {
  public:
    struct GasRow {
        // Using uint16_t here because it's the smallest type that can hold the largest value in the table.
        // The idea is to save memory when generating the events/trace.
        uint16_t base_l2_gas_fixed_table;
        uint16_t base_da_gas_fixed_table;
        uint16_t dyn_l2_gas_fixed_table;
        uint16_t dyn_da_gas_fixed_table;
    };

    static const FixedGasTable& get();

    size_t size() const;
    const GasRow& at(size_t i) const { return at(static_cast<OpCode>(i)); }
    const GasRow& at(OpCode o) const;

  private:
    FixedGasTable() = default;
};

template <typename DestRow> void merge_into(DestRow& dest, FixedGasTable::GasRow const& src)
{
    dest.gas_sel_gas_cost = FF(1);
    dest.gas_base_l2_gas_fixed_table = src.base_l2_gas_fixed_table;
    dest.gas_base_da_gas_fixed_table = src.base_da_gas_fixed_table;
    dest.gas_dyn_l2_gas_fixed_table = src.dyn_l2_gas_fixed_table;
    dest.gas_dyn_da_gas_fixed_table = src.dyn_da_gas_fixed_table;
}

} // namespace bb::avm_trace