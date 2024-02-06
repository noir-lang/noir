#include "AvmMini_deserialization.hpp"
#include "barretenberg/vm/avm_trace/AvmMini_common.hpp"
#include "barretenberg/vm/avm_trace/AvmMini_instructions.hpp"
#include "barretenberg/vm/avm_trace/AvmMini_opcode.hpp"
#include <cassert>
#include <cstddef>
#include <cstdint>
#include <iostream>
#include <string>
#include <vector>

namespace avm_trace {

namespace {

const std::vector<OperandType> three_operand_format = {
    OperandType::TAG,
    OperandType::UINT32,
    OperandType::UINT32,
    OperandType::UINT32,
};

// Contrary to TS, the format does not contain the opcode byte which prefixes any instruction.
// The format for OpCode::SET has to be handled separately as it is variable based on the tag.
const std::unordered_map<OpCode, std::vector<OperandType>> OPCODE_WIRE_FORMAT = {
    // Compute
    // Compute - Arithmetic
    { OpCode::ADD, three_operand_format },
    { OpCode::SUB, three_operand_format },
    { OpCode::MUL, three_operand_format },
    { OpCode::DIV, three_operand_format },
    // Execution Environment - Calldata
    { OpCode::CALLDATACOPY, { OperandType::UINT32, OperandType::UINT32, OperandType::UINT32 } },
    // Machine State - Internal Control Flow
    { OpCode::JUMP, { OperandType::UINT32 } },
    { OpCode::INTERNALCALL, { OperandType::UINT32 } },
    { OpCode::INTERNALRETURN, {} },
    // Machine State - Memory
    // OpCode::SET is handled differently
    // Control Flow - Contract Calls
    { OpCode::RETURN, { OperandType::UINT32, OperandType::UINT32 } },
};

const std::unordered_map<OperandType, size_t> OPERAND_TYPE_SIZE = {
    { OperandType::TAG, 1 },    { OperandType::UINT8, 1 },  { OperandType::UINT16, 2 },
    { OperandType::UINT32, 4 }, { OperandType::UINT64, 8 }, { OperandType::UINT128, 16 },
};

} // Anonymous namespace

/**
 * @brief Parsing of the supplied bytecode into a vector of instructions. It essentially
 *        checks that each opcode value is in the defined range and extracts the operands
 *        for each opcode based on the specification from OPCODE_WIRE_FORMAT.
 *
 * @param bytecode The bytecode to be parsed as a vector of bytes/uint8_t
 * @throws runtime_error exception when the bytecode is invalid.
 * @return Vector of instructions
 */
std::vector<Instruction> Deserialization::parse(std::vector<uint8_t> const& bytecode)
{
    std::vector<Instruction> instructions;
    size_t pos = 0;
    const auto length = bytecode.size();

    while (pos < length) {
        const uint8_t opcode_byte = bytecode.at(pos);

        if (!Bytecode::is_valid(opcode_byte)) {
            throw_or_abort("Invalid opcode byte: " + std::to_string(opcode_byte) +
                           " at position: " + std::to_string(pos));
        }
        pos++;

        auto const opcode = static_cast<OpCode>(opcode_byte);
        std::vector<OperandType> inst_format;

        if (opcode == OpCode::SET) {
            if (pos == length) {
                throw_or_abort("Operand for SET opcode is missing at position " + std::to_string(pos));
            }

            std::set<uint8_t> const valid_tags = { static_cast<uint8_t>(AvmMemoryTag::U8),
                                                   static_cast<uint8_t>(AvmMemoryTag::U16),
                                                   static_cast<uint8_t>(AvmMemoryTag::U32),
                                                   static_cast<uint8_t>(AvmMemoryTag::U64),
                                                   static_cast<uint8_t>(AvmMemoryTag::U128) };
            uint8_t set_tag_u8 = bytecode.at(pos);

            if (!valid_tags.contains(set_tag_u8)) {
                throw_or_abort("Instruction tag for SET opcode is invalid at position " + std::to_string(pos) +
                               " value: " + std::to_string(set_tag_u8));
            }

            auto in_tag = static_cast<AvmMemoryTag>(set_tag_u8);
            switch (in_tag) {
            case AvmMemoryTag::U8:
                inst_format = { OperandType::TAG, OperandType::UINT8, OperandType::UINT32 };
                break;
            case AvmMemoryTag::U16:
                inst_format = { OperandType::TAG, OperandType::UINT16, OperandType::UINT32 };
                break;
            case AvmMemoryTag::U32:
                inst_format = { OperandType::TAG, OperandType::UINT32, OperandType::UINT32 };
                break;
            case AvmMemoryTag::U64:
                inst_format = { OperandType::TAG, OperandType::UINT64, OperandType::UINT32 };
                break;
            case AvmMemoryTag::U128:
                inst_format = { OperandType::TAG, OperandType::UINT128, OperandType::UINT32 };
                break;
            default: // This branch is guarded above.
                std::cerr << "This code branch must have been guarded by the tag validation. \n";
                assert(false);
            }
        } else {
            inst_format = OPCODE_WIRE_FORMAT.at(opcode);
        }

        std::vector<Operand> operands;

        for (OperandType const& opType : inst_format) {
            // No underflow as while condition guarantees pos <= length (after pos++)
            if (length - pos < OPERAND_TYPE_SIZE.at(opType)) {
                throw_or_abort("Operand is missing at position " + std::to_string(pos));
            }

            switch (opType) {
            case OperandType::TAG: {
                uint8_t tag_u8 = bytecode.at(pos);
                if (tag_u8 == static_cast<uint8_t>(AvmMemoryTag::U0) || tag_u8 > MAX_MEM_TAG) {
                    throw_or_abort("Instruction tag is invalid at position " + std::to_string(pos) +
                                   " value: " + std::to_string(tag_u8));
                }
                operands.emplace_back(static_cast<AvmMemoryTag>(tag_u8));
                break;
            }
            case OperandType::UINT8:
                operands.emplace_back(bytecode.at(pos));
                break;
            case OperandType::UINT16: {
                uint16_t operand_u16 = 0;
                uint8_t const* pos_ptr = &bytecode.at(pos);
                serialize::read(pos_ptr, operand_u16);
                operands.emplace_back(operand_u16);
                break;
            }
            case OperandType::UINT32: {
                uint32_t operand_u32 = 0;
                uint8_t const* pos_ptr = &bytecode.at(pos);
                serialize::read(pos_ptr, operand_u32);
                operands.emplace_back(operand_u32);
                break;
            }
            case OperandType::UINT64: {
                uint64_t operand_u64 = 0;
                uint8_t const* pos_ptr = &bytecode.at(pos);
                serialize::read(pos_ptr, operand_u64);
                operands.emplace_back(operand_u64);
                break;
            }
            case OperandType::UINT128: {
                uint128_t operand_u128 = 0;
                uint8_t const* pos_ptr = &bytecode.at(pos);
                serialize::read(pos_ptr, operand_u128);
                operands.emplace_back(operand_u128);
                break;
            }
            }
            pos += OPERAND_TYPE_SIZE.at(opType);
        }
        instructions.emplace_back(opcode, operands);
    }
    return instructions;
};
} // namespace avm_trace