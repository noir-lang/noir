#pragma once

#include "barretenberg/numeric/uint128/uint128.hpp"
#include "barretenberg/vm/avm_trace/AvmMini_common.hpp"
#include "barretenberg/vm/avm_trace/AvmMini_instructions.hpp"
#include "barretenberg/vm/avm_trace/AvmMini_opcode.hpp"
#include <cstddef>
#include <cstdint>
#include <unordered_map>
#include <variant>
#include <vector>

namespace avm_trace {

// Possible types for an instruction's operand in its wire format. (Keep in sync with TS code.
// See avm/serialization/instruction_serialization.ts).
// Note that the TAG enum value is not supported in TS and is parsed as UINT8.
enum class OperandType : uint8_t { TAG, UINT8, UINT16, UINT32, UINT64, UINT128 };

class Deserialization {
  public:
    Deserialization() = default;

    static std::vector<Instruction> parse(std::vector<uint8_t> const& bytecode);
};

} // namespace avm_trace