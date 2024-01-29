#include "AvmMini_execution.hpp"
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/proof_system/circuit_builder/generated/AvmMini_circuit_builder.hpp"
#include "barretenberg/vm/avm_trace/AvmMini_common.hpp"
#include "barretenberg/vm/avm_trace/AvmMini_instructions.hpp"
#include "barretenberg/vm/avm_trace/AvmMini_opcode.hpp"
#include "barretenberg/vm/avm_trace/AvmMini_trace.hpp"
#include "barretenberg/vm/generated/AvmMini_composer.hpp"
#include <cstddef>
#include <cstdint>
#include <string>
#include <vector>

namespace avm_trace {

/**
 * @brief Run the bytecode, generate the corresponding execution trace and prove the correctness
 *        of the execution of the supplied bytecode.
 *
 * @param bytecode A vector of bytes representing the bytecode to execute.
 * @param calldata expressed as a vector of finite field elements.
 * @throws runtime_error exception when the bytecode is invalid.
 * @return A zk proof of the execution.
 */
plonk::proof Execution::run_and_prove(std::vector<uint8_t> const& bytecode, std::vector<FF> const& calldata)
{
    auto instructions = parse(bytecode);
    auto trace = gen_trace(instructions, calldata);
    auto circuit_builder = bb::AvmMiniCircuitBuilder();
    circuit_builder.set_trace(std::move(trace));

    auto composer = bb::honk::AvmMiniComposer();
    auto prover = composer.create_prover(circuit_builder);
    return prover.construct_proof();
}

/**
 * @brief Parsing of the supplied bytecode into a vector of instructions. It essentially
 *        checks that each opcode value is in the defined range and extracts the operands
 *        for each opcode.
 *
 * @param bytecode The bytecode to be parsed as a vector of bytes/uint8_t
 * @throws runtime_error exception when the bytecode is invalid.
 * @return Vector of instructions
 */
std::vector<Instruction> Execution::parse(std::vector<uint8_t> const& bytecode)
{
    std::vector<Instruction> instructions;
    size_t pos = 0;
    const auto length = bytecode.size();

    while (pos < length) {
        const uint8_t opcode_byte = bytecode.at(pos);
        pos += AVM_OPCODE_BYTE_LENGTH;

        if (!Bytecode::is_valid(opcode_byte)) {
            throw std::runtime_error("Invalid opcode byte: " + std::to_string(opcode_byte));
        }

        const auto opcode = static_cast<OpCode>(opcode_byte);
        auto in_tag_u8 = static_cast<uint8_t>(AvmMemoryTag::U0);

        if (Bytecode::has_in_tag(opcode)) {
            if (pos + AVM_IN_TAG_BYTE_LENGTH > length) {
                throw std::runtime_error("Instruction tag missing at position " + std::to_string(pos));
            }
            in_tag_u8 = bytecode.at(pos);
            if (in_tag_u8 == static_cast<uint8_t>(AvmMemoryTag::U0) || in_tag_u8 > MAX_MEM_TAG) {
                throw std::runtime_error("Instruction tag is invalid at position " + std::to_string(pos) +
                                         " value: " + std::to_string(in_tag_u8));
            }
            pos += AVM_IN_TAG_BYTE_LENGTH;
        }

        auto const in_tag = static_cast<AvmMemoryTag>(in_tag_u8);
        std::vector<uint32_t> operands{};
        size_t num_of_operands{};
        size_t operands_size{};

        // SET opcode particularity about the number of operands depending on the
        // instruction tag. Namely, a constant of type instruction tag and not a
        // memory address is passed in the operands.
        // The bytecode of the operands is of the form CONSTANT || dst_offset
        // CONSTANT is of size k bits for type Uk, k=8,16,32,64,128
        // dst_offset is of size 32 bits
        // CONSTANT has to be decomposed into 32-bit chunks
        if (opcode == OpCode::SET) {
            switch (in_tag) {
            case AvmMemoryTag::U8:
                num_of_operands = 2;
                operands_size = 5;
                break;
            case AvmMemoryTag::U16:
                num_of_operands = 2;
                operands_size = 6;
                break;
            case AvmMemoryTag::U32:
                num_of_operands = 2;
                operands_size = 8;
                break;
            case AvmMemoryTag::U64:
                num_of_operands = 3;
                operands_size = 12;
                break;
            case AvmMemoryTag::U128:
                num_of_operands = 5;
                operands_size = 20;
                break;
            default:
                throw std::runtime_error("Instruction tag for SET opcode is invalid at position " +
                                         std::to_string(pos) + " value: " + std::to_string(in_tag_u8));
                break;
            }
        } else {
            num_of_operands = Bytecode::OPERANDS_NUM.at(opcode);
            operands_size = AVM_OPERAND_BYTE_LENGTH * num_of_operands;
        }

        if (pos + operands_size > length) {
            throw std::runtime_error("Operand is missing at position " + std::to_string(pos));
        }

        // We handle operands which are encoded with less than 4 bytes.
        // This occurs for opcode SET and tag U8 and U16.
        if (opcode == OpCode::SET && in_tag == AvmMemoryTag::U8) {
            operands.push_back(static_cast<uint32_t>(bytecode.at(pos)));
            pos++;
            num_of_operands--;
        } else if (opcode == OpCode::SET && in_tag == AvmMemoryTag::U16) {
            uint8_t const* ptr = &bytecode.at(pos);
            uint16_t operand{};
            serialize::read(ptr, operand);
            operands.push_back(static_cast<uint32_t>(operand));
            pos += 2;
            num_of_operands--;
        }

        // Operands of size of 32 bits.
        for (size_t i = 0; i < num_of_operands; i++) {
            uint8_t const* ptr = &bytecode.at(pos);
            uint32_t operand{};
            serialize::read(ptr, operand);
            operands.push_back(operand);
            pos += AVM_OPERAND_BYTE_LENGTH;
        }

        instructions.emplace_back(opcode, operands, static_cast<AvmMemoryTag>(in_tag));
    }

    return instructions;
}

/**
 * @brief Generate the execution trace pertaining to the supplied instructions.
 *
 * @param instructions A vector of the instructions to be executed.
 * @param calldata expressed as a vector of finite field elements.
 * @return The trace as a vector of Row.
 */
std::vector<Row> Execution::gen_trace(std::vector<Instruction> const& instructions, std::vector<FF> const& calldata)
{
    AvmMiniTraceBuilder trace_builder{};

    // copied version of pc maintained in trace builder. The value of pc is evolving based
    // on opcode logic and therefore is not maintained here. However, the next opcode in the execution
    // is determined by this value which require read access to the code below.
    uint32_t pc = 0;
    auto const inst_size = instructions.size();

    while ((pc = trace_builder.getPc()) < inst_size) {
        auto inst = instructions.at(pc);

        switch (inst.op_code) {
        case OpCode::ADD:
            trace_builder.add(inst.operands.at(0), inst.operands.at(1), inst.operands.at(2), inst.in_tag);
            break;
        case OpCode::SUB:
            trace_builder.sub(inst.operands.at(0), inst.operands.at(1), inst.operands.at(2), inst.in_tag);
            break;
        case OpCode::MUL:
            trace_builder.mul(inst.operands.at(0), inst.operands.at(1), inst.operands.at(2), inst.in_tag);
            break;
        case OpCode::DIV:
            trace_builder.div(inst.operands.at(0), inst.operands.at(1), inst.operands.at(2), inst.in_tag);
            break;
        case OpCode::CALLDATACOPY:
            trace_builder.calldata_copy(inst.operands.at(0), inst.operands.at(1), inst.operands.at(2), calldata);
            break;
        case OpCode::JUMP:
            trace_builder.jump(inst.operands.at(0));
            break;
        case OpCode::INTERNALCALL:
            trace_builder.internal_call(inst.operands.at(0));
            break;
        case OpCode::INTERNALRETURN:
            trace_builder.internal_return();
            break;
        case OpCode::SET: {
            uint32_t dst_offset{};
            uint128_t val{};
            switch (inst.in_tag) {
            case AvmMemoryTag::U8:
            case AvmMemoryTag::U16:
            case AvmMemoryTag::U32:
                // U8, U16, U32 value represented in a single uint32_t operand
                val = inst.operands.at(0);
                dst_offset = inst.operands.at(1);
                break;
            case AvmMemoryTag::U64: // value represented as 2 uint32_t operands
                val = inst.operands.at(0);
                val <<= 32;
                val += inst.operands.at(1);
                dst_offset = inst.operands.at(2);
                break;
            case AvmMemoryTag::U128: // value represented as 4 uint32_t operands
                for (size_t i = 0; i < 4; i++) {
                    val += inst.operands.at(i);
                    val <<= 32;
                }
                dst_offset = inst.operands.at(4);
                break;
            default:
                break;
            }
            trace_builder.set(val, dst_offset, inst.in_tag);
            break;
        }
        case OpCode::RETURN:
            trace_builder.return_op(inst.operands.at(0), inst.operands.at(1));
            break;
        default:
            break;
        }
    }
    return trace_builder.finalize();
}

} // namespace avm_trace