#pragma once

#include "barretenberg/numeric/uint128/uint128.hpp"
#include "barretenberg/vm/avm_trace/AvmMini_common.hpp"
#include "barretenberg/vm/avm_trace/AvmMini_opcode.hpp"
#include <cstdint>
#include <vector>

namespace avm_trace {

using Operand = std::variant<AvmMemoryTag, uint8_t, uint16_t, uint32_t, uint64_t, uint128_t>;

class Instruction {
  public:
    OpCode op_code;
    std::vector<Operand> operands;

    Instruction() = delete;
    explicit Instruction(OpCode op_code, std::vector<Operand> operands)
        : op_code(op_code)
        , operands(std::move(operands)){};
};

} // namespace avm_trace