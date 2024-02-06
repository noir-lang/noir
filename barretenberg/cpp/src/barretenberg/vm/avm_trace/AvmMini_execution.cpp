#include "AvmMini_execution.hpp"
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/proof_system/circuit_builder/generated/AvmMini_circuit_builder.hpp"
#include "barretenberg/vm/avm_trace/AvmMini_common.hpp"
#include "barretenberg/vm/avm_trace/AvmMini_deserialization.hpp"
#include "barretenberg/vm/avm_trace/AvmMini_instructions.hpp"
#include "barretenberg/vm/avm_trace/AvmMini_opcode.hpp"
#include "barretenberg/vm/avm_trace/AvmMini_trace.hpp"
#include "barretenberg/vm/generated/AvmMini_composer.hpp"
#include <cstddef>
#include <cstdint>
#include <string>
#include <variant>
#include <vector>

using namespace bb;

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
HonkProof Execution::run_and_prove(std::vector<uint8_t> const& bytecode, std::vector<FF> const& calldata)
{
    auto instructions = Deserialization::parse(bytecode);
    auto trace = gen_trace(instructions, calldata);
    auto circuit_builder = bb::AvmMiniCircuitBuilder();
    circuit_builder.set_trace(std::move(trace));

    auto composer = AvmMiniComposer();
    auto prover = composer.create_prover(circuit_builder);
    return prover.construct_proof();
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
    AvmMiniTraceBuilder trace_builder;

    // Copied version of pc maintained in trace builder. The value of pc is evolving based
    // on opcode logic and therefore is not maintained here. However, the next opcode in the execution
    // is determined by this value which require read access to the code below.
    uint32_t pc = 0;
    auto const inst_size = instructions.size();

    while ((pc = trace_builder.getPc()) < inst_size) {
        auto inst = instructions.at(pc);

        switch (inst.op_code) {
            // Compute
            // Compute - Arithmetic
        case OpCode::ADD:
            trace_builder.add(std::get<uint32_t>(inst.operands.at(1)),
                              std::get<uint32_t>(inst.operands.at(2)),
                              std::get<uint32_t>(inst.operands.at(3)),
                              std::get<AvmMemoryTag>(inst.operands.at(0)));
            break;
        case OpCode::SUB:
            trace_builder.sub(std::get<uint32_t>(inst.operands.at(1)),
                              std::get<uint32_t>(inst.operands.at(2)),
                              std::get<uint32_t>(inst.operands.at(3)),
                              std::get<AvmMemoryTag>(inst.operands.at(0)));
            break;
        case OpCode::MUL:
            trace_builder.mul(std::get<uint32_t>(inst.operands.at(1)),
                              std::get<uint32_t>(inst.operands.at(2)),
                              std::get<uint32_t>(inst.operands.at(3)),
                              std::get<AvmMemoryTag>(inst.operands.at(0)));
            break;
        case OpCode::DIV:
            trace_builder.div(std::get<uint32_t>(inst.operands.at(1)),
                              std::get<uint32_t>(inst.operands.at(2)),
                              std::get<uint32_t>(inst.operands.at(3)),
                              std::get<AvmMemoryTag>(inst.operands.at(0)));
            break;
            // Execution Environment - Calldata
        case OpCode::CALLDATACOPY:
            trace_builder.calldata_copy(std::get<uint32_t>(inst.operands.at(0)),
                                        std::get<uint32_t>(inst.operands.at(1)),
                                        std::get<uint32_t>(inst.operands.at(2)),
                                        calldata);
            break;
            // Machine State - Internal Control Flow
        case OpCode::JUMP:
            trace_builder.jump(std::get<uint32_t>(inst.operands.at(0)));
            break;
        case OpCode::INTERNALCALL:
            trace_builder.internal_call(std::get<uint32_t>(inst.operands.at(0)));
            break;
        case OpCode::INTERNALRETURN:
            trace_builder.internal_return();
            break;
            // Machine State - Memory
        case OpCode::SET: {
            uint32_t dst_offset = 0;
            uint128_t val = 0;
            AvmMemoryTag in_tag = std::get<AvmMemoryTag>(inst.operands.at(0));
            dst_offset = std::get<uint32_t>(inst.operands.at(2));

            switch (in_tag) {
            case AvmMemoryTag::U8:
                val = std::get<uint8_t>(inst.operands.at(1));
                break;
            case AvmMemoryTag::U16:
                val = std::get<uint16_t>(inst.operands.at(1));
                break;
            case AvmMemoryTag::U32:
                val = std::get<uint32_t>(inst.operands.at(1));
                break;
            case AvmMemoryTag::U64:
                val = std::get<uint64_t>(inst.operands.at(1));
                break;
            case AvmMemoryTag::U128:
                val = std::get<uint128_t>(inst.operands.at(1));
                break;
            default:
                break;
            }

            trace_builder.set(val, dst_offset, in_tag);
            break;
        }
            // Control Flow - Contract Calls
        case OpCode::RETURN:
            trace_builder.return_op(std::get<uint32_t>(inst.operands.at(0)), std::get<uint32_t>(inst.operands.at(1)));
            break;
        default:
            break;
        }
    }
    return trace_builder.finalize();
}

} // namespace avm_trace