#include "avm_execution.hpp"
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/avm_trace/avm_deserialization.hpp"
#include "barretenberg/vm/avm_trace/avm_instructions.hpp"
#include "barretenberg/vm/avm_trace/avm_opcode.hpp"
#include "barretenberg/vm/avm_trace/avm_trace.hpp"
#include "barretenberg/vm/generated/avm_circuit_builder.hpp"
#include "barretenberg/vm/generated/avm_composer.hpp"
#include "barretenberg/vm/generated/avm_flavor.hpp"
#include <cassert>
#include <cstddef>
#include <cstdint>
#include <string>
#include <tuple>
#include <variant>
#include <vector>

using namespace bb;

namespace bb::avm_trace {

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
    auto circuit_builder = bb::AvmCircuitBuilder();
    circuit_builder.set_trace(std::move(trace));

    auto composer = AvmComposer();
    auto prover = composer.create_prover(circuit_builder);
    auto verifier = composer.create_verifier(circuit_builder);
    auto proof = prover.construct_proof();
    return proof;
}

std::tuple<AvmFlavor::VerificationKey, HonkProof> Execution::prove(std::vector<uint8_t> const& bytecode,
                                                                   std::vector<FF> const& calldata)
{
    auto instructions = Deserialization::parse(bytecode);
    auto trace = gen_trace(instructions, calldata);
    auto circuit_builder = bb::AvmCircuitBuilder();
    circuit_builder.set_trace(std::move(trace));

    // Temporarily use this until #4954 is resolved
    assert(circuit_builder.check_circuit());

    auto composer = AvmComposer();
    auto prover = composer.create_prover(circuit_builder);
    auto verifier = composer.create_verifier(circuit_builder);
    auto proof = prover.construct_proof();
    // TODO(#4887): Might need to return PCS vk when full verify is supported
    return std::make_tuple(*verifier.key, proof);
}

bool Execution::verify(AvmFlavor::VerificationKey vk, HonkProof const& proof)
{
    auto verification_key = std::make_shared<AvmFlavor::VerificationKey>(vk);
    AvmVerifier verifier(verification_key);

    // todo: not needed for now until we verify the PCS/pairing of the proof
    // auto pcs_verification_key = std::make_unique<VerifierCommitmentKey>(verification_key->circuit_size,
    // crs_factory_);
    // output_state.pcs_verification_key = std::move(pcs_verification_key);

    return verifier.verify_proof(proof);
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
    AvmTraceBuilder trace_builder;

    // Copied version of pc maintained in trace builder. The value of pc is evolving based
    // on opcode logic and therefore is not maintained here. However, the next opcode in the execution
    // is determined by this value which require read access to the code below.
    uint32_t pc = 0;
    auto const inst_size = instructions.size();

    while ((pc = trace_builder.getPc()) < inst_size) {
        auto inst = instructions.at(pc);

        // TODO: We do not yet support the indirect flag. Therefore we do not extract
        // inst.operands(0) (i.e. the indirect flag) when processiing the instructions.
        switch (inst.op_code) {
            // Compute
            // Compute - Arithmetic
        case OpCode::ADD:
            trace_builder.op_add(std::get<uint8_t>(inst.operands.at(0)),
                                 std::get<uint32_t>(inst.operands.at(2)),
                                 std::get<uint32_t>(inst.operands.at(3)),
                                 std::get<uint32_t>(inst.operands.at(4)),
                                 std::get<AvmMemoryTag>(inst.operands.at(1)));
            break;
        case OpCode::SUB:
            trace_builder.op_sub(std::get<uint8_t>(inst.operands.at(0)),
                                 std::get<uint32_t>(inst.operands.at(2)),
                                 std::get<uint32_t>(inst.operands.at(3)),
                                 std::get<uint32_t>(inst.operands.at(4)),
                                 std::get<AvmMemoryTag>(inst.operands.at(1)));
            break;
        case OpCode::MUL:
            trace_builder.op_mul(std::get<uint8_t>(inst.operands.at(0)),
                                 std::get<uint32_t>(inst.operands.at(2)),
                                 std::get<uint32_t>(inst.operands.at(3)),
                                 std::get<uint32_t>(inst.operands.at(4)),
                                 std::get<AvmMemoryTag>(inst.operands.at(1)));
            break;
        case OpCode::DIV:
            trace_builder.op_div(std::get<uint8_t>(inst.operands.at(0)),
                                 std::get<uint32_t>(inst.operands.at(2)),
                                 std::get<uint32_t>(inst.operands.at(3)),
                                 std::get<uint32_t>(inst.operands.at(4)),
                                 std::get<AvmMemoryTag>(inst.operands.at(1)));
            break;
        // Compute - Comparators
        case OpCode::EQ:
            trace_builder.op_eq(std::get<uint8_t>(inst.operands.at(0)),
                                std::get<uint32_t>(inst.operands.at(2)),
                                std::get<uint32_t>(inst.operands.at(3)),
                                std::get<uint32_t>(inst.operands.at(4)),
                                std::get<AvmMemoryTag>(inst.operands.at(1)));
            break;
        // Compute - Bitwise
        case OpCode::NOT:
            trace_builder.op_not(std::get<uint8_t>(inst.operands.at(0)),
                                 std::get<uint32_t>(inst.operands.at(2)),
                                 std::get<uint32_t>(inst.operands.at(3)),
                                 std::get<AvmMemoryTag>(inst.operands.at(1)));
            break;

        case OpCode::AND:
            trace_builder.op_and(std::get<uint8_t>(inst.operands.at(0)),
                                 std::get<uint32_t>(inst.operands.at(2)),
                                 std::get<uint32_t>(inst.operands.at(3)),
                                 std::get<uint32_t>(inst.operands.at(4)),
                                 std::get<AvmMemoryTag>(inst.operands.at(1)));
            break;
        case OpCode::OR:
            trace_builder.op_or(std::get<uint8_t>(inst.operands.at(0)),
                                std::get<uint32_t>(inst.operands.at(2)),
                                std::get<uint32_t>(inst.operands.at(3)),
                                std::get<uint32_t>(inst.operands.at(4)),
                                std::get<AvmMemoryTag>(inst.operands.at(1)));
            break;

        case OpCode::XOR:
            trace_builder.op_xor(std::get<uint8_t>(inst.operands.at(0)),
                                 std::get<uint32_t>(inst.operands.at(2)),
                                 std::get<uint32_t>(inst.operands.at(3)),
                                 std::get<uint32_t>(inst.operands.at(4)),
                                 std::get<AvmMemoryTag>(inst.operands.at(1)));
            break;
            // Execution Environment - Calldata
        case OpCode::CALLDATACOPY:
            trace_builder.calldata_copy(std::get<uint8_t>(inst.operands.at(0)),
                                        std::get<uint32_t>(inst.operands.at(1)),
                                        std::get<uint32_t>(inst.operands.at(2)),
                                        std::get<uint32_t>(inst.operands.at(3)),
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
            uint128_t val = 0;
            AvmMemoryTag in_tag = std::get<AvmMemoryTag>(inst.operands.at(1));

            switch (in_tag) {
            case AvmMemoryTag::U8:
                val = std::get<uint8_t>(inst.operands.at(2));
                break;
            case AvmMemoryTag::U16:
                val = std::get<uint16_t>(inst.operands.at(2));
                break;
            case AvmMemoryTag::U32:
                val = std::get<uint32_t>(inst.operands.at(2));
                break;
            case AvmMemoryTag::U64:
                val = std::get<uint64_t>(inst.operands.at(2));
                break;
            case AvmMemoryTag::U128:
                val = std::get<uint128_t>(inst.operands.at(2));
                break;
            default:
                break;
            }

            trace_builder.op_set(
                std::get<uint8_t>(inst.operands.at(0)), val, std::get<uint32_t>(inst.operands.at(3)), in_tag);
            break;
        }
        case OpCode::MOV:
            trace_builder.op_mov(std::get<uint8_t>(inst.operands.at(0)),
                                 std::get<uint32_t>(inst.operands.at(1)),
                                 std::get<uint32_t>(inst.operands.at(2)));
            break;
            // Control Flow - Contract Calls
        case OpCode::RETURN:
            trace_builder.return_op(std::get<uint8_t>(inst.operands.at(0)),
                                    std::get<uint32_t>(inst.operands.at(1)),
                                    std::get<uint32_t>(inst.operands.at(2)));
            break;
        default:
            break;
        }
    }
    return trace_builder.finalize();
}

} // namespace bb::avm_trace
