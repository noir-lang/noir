#pragma once

#include <cstddef>
#include <vector>

#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/relations/generated/avm/gas.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/avm_trace/avm_opcode.hpp"

namespace bb::avm_trace {

class FixedGasTable {
  public:
    using GasRow = bb::Avm_vm::GasRow<FF>;

    static const FixedGasTable& get();

    size_t size() const { return table_rows.size(); }
    const GasRow& at(size_t i) const { return table_rows.at(i); }
    const GasRow& at(OpCode o) const { return at(static_cast<size_t>(o)); }

  private:
    FixedGasTable();

    std::vector<GasRow> table_rows;
};

template <typename DestRow> void merge_into(DestRow& dest, FixedGasTable::GasRow const& src)
{
    dest.gas_sel_gas_cost = src.gas_sel_gas_cost;
    dest.gas_l2_gas_fixed_table = src.gas_l2_gas_fixed_table;
    dest.gas_da_gas_fixed_table = src.gas_da_gas_fixed_table;
}

} // namespace bb::avm_trace