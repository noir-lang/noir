#pragma once

#include <cstddef>
#include <vector>

#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/relations/generated/avm/powers.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"

namespace bb::avm_trace {

class FixedPowersTable {
  public:
    using PowersRow = bb::Avm_vm::PowersRow<FF>;

    static const FixedPowersTable& get();

    size_t size() const { return table_rows.size(); }
    const PowersRow& at(size_t i) const { return table_rows.at(i); }

  private:
    FixedPowersTable();

    std::vector<PowersRow> table_rows;
};

template <typename DestRow> void merge_into(DestRow& dest, FixedPowersTable::PowersRow const& src)
{
    dest.powers_power_of_2 = src.powers_power_of_2;
}

} // namespace bb::avm_trace