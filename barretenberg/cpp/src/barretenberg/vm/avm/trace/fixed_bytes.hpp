#pragma once

#include <cstddef>
#include <cstdint>

#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/vm/avm/trace/common.hpp"
#include "barretenberg/vm/avm/trace/opcode.hpp"

namespace bb::avm_trace {

class FixedBytesTable {
  public:
    static const FixedBytesTable& get();

    void finalize(std::vector<AvmFullRow<FF>>& main_trace) const;
    void finalize_for_testing(std::vector<AvmFullRow<FF>>& main_trace,
                              const std::unordered_map<uint32_t, uint32_t>& byte_operation_counter) const;

  private:
    FixedBytesTable() = default;
    static void finalize_byte_length(std::vector<AvmFullRow<FF>>& main_trace);
};

} // namespace bb::avm_trace