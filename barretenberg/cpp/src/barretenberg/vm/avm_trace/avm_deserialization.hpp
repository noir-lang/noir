#pragma once

#include "barretenberg/numeric/uint128/uint128.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/avm_trace/avm_instructions.hpp"
#include "barretenberg/vm/avm_trace/avm_opcode.hpp"
#include <cstddef>
#include <cstdint>
#include <unordered_map>
#include <variant>
#include <vector>

namespace bb::avm_trace {

// Possible types for an instruction's operand in its wire format. (Keep in sync with TS code.
// See avm/serialization/instruction_serialization.ts).
// Note that the TAG enum value is not supported in TS and is parsed as UINT8.
// INDIRECT is parsed as UINT8 where the bits represent the operands that have indirect mem access.
enum class OperandType : uint8_t { INDIRECT, TAG, UINT8, UINT16, UINT32, UINT64, UINT128 };

class Deserialization {
  public:
    Deserialization() = default;

    static std::vector<Instruction> parse(std::vector<uint8_t> const& bytecode);
};

} // namespace bb::avm_trace
