#pragma once

#include "barretenberg/numeric/uint128/uint128.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/avm_trace/avm_opcode.hpp"

#include <cstdint>
#include <string>
#include <variant>
#include <vector>

namespace bb::avm_trace {

using Operand = std::variant<AvmMemoryTag, uint8_t, uint16_t, uint32_t, uint64_t, uint128_t>;

class Instruction {
  public:
    OpCode op_code;
    std::vector<Operand> operands;

    Instruction() = delete;
    explicit Instruction(OpCode op_code, std::vector<Operand> operands)
        : op_code(op_code)
        , operands(std::move(operands)){};

    std::string to_string() const
    {
        std::string str = bb::avm_trace::to_string(op_code);
        for (const auto& operand : operands) {
            str += " ";
            if (std::holds_alternative<AvmMemoryTag>(operand)) {
                str += std::to_string(static_cast<int>(std::get<AvmMemoryTag>(operand)));
            } else if (std::holds_alternative<uint8_t>(operand)) {
                str += std::to_string(std::get<uint8_t>(operand));
            } else if (std::holds_alternative<uint16_t>(operand)) {
                str += std::to_string(std::get<uint16_t>(operand));
            } else if (std::holds_alternative<uint32_t>(operand)) {
                str += std::to_string(std::get<uint32_t>(operand));
            } else if (std::holds_alternative<uint64_t>(operand)) {
                str += std::to_string(std::get<uint64_t>(operand));
            } else if (std::holds_alternative<uint128_t>(operand)) {
                str += "someu128";
            } else {
                str += "unknown operand type";
            }
        }
        return str;
    }
};

} // namespace bb::avm_trace
