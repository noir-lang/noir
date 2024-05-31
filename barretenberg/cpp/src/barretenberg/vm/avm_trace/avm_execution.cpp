#include "barretenberg/vm/avm_trace/avm_execution.hpp"
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/avm_trace/avm_deserialization.hpp"
#include "barretenberg/vm/avm_trace/avm_helper.hpp"
#include "barretenberg/vm/avm_trace/avm_instructions.hpp"
#include "barretenberg/vm/avm_trace/avm_kernel_trace.hpp"
#include "barretenberg/vm/avm_trace/avm_opcode.hpp"
#include "barretenberg/vm/avm_trace/avm_trace.hpp"
#include "barretenberg/vm/avm_trace/aztec_constants.hpp"
#include "barretenberg/vm/avm_trace/constants.hpp"
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
 * @brief Temporary routine to generate default public inputs (gas values) until we get
 *        proper integration of public inputs.
 */
std::vector<FF> Execution::getDefaultPublicInputs()
{
    std::vector<FF> public_inputs_vec(PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH);
    public_inputs_vec.at(DA_GAS_LEFT_CONTEXT_INPUTS_OFFSET) = 1000000000;
    public_inputs_vec.at(L2_GAS_LEFT_CONTEXT_INPUTS_OFFSET) = 1000000000;
    return public_inputs_vec;
}

/**
 * @brief Run the bytecode, generate the corresponding execution trace and prove the correctness
 *        of the execution of the supplied bytecode.
 *
 * @param bytecode A vector of bytes representing the bytecode to execute.
 * @param calldata expressed as a vector of finite field elements.
 * @throws runtime_error exception when the bytecode is invalid.
 * @return The verifier key and zk proof of the execution.
 */
std::tuple<AvmFlavor::VerificationKey, HonkProof> Execution::prove(std::vector<uint8_t> const& bytecode,
                                                                   std::vector<FF> const& calldata)
{
    auto instructions = Deserialization::parse(bytecode);
    std::vector<FF> returndata{};
    auto trace = gen_trace(instructions, returndata, calldata, getDefaultPublicInputs());
    auto circuit_builder = bb::AvmCircuitBuilder();
    circuit_builder.set_trace(std::move(trace));

    auto composer = AvmComposer();
    auto prover = composer.create_prover(circuit_builder);
    auto verifier = composer.create_verifier(circuit_builder);
    auto proof = prover.construct_proof();
    // TODO(#4887): Might need to return PCS vk when full verify is supported
    return std::make_tuple(*verifier.key, proof);
}

/**
 * @brief Convert Public Inputs
 *
 * **Transitional**
 * Converts public inputs from the public inputs vec (PublicCircuitPublicInputs) into it's respective public input
 * columns Which are represented by the `VmPublicInputs` object.
 *
 * @param public_inputs_vec
 * @return VmPublicInputs
 */
VmPublicInputs Execution::convert_public_inputs(std::vector<FF> const& public_inputs_vec)
{
    VmPublicInputs public_inputs = {};

    // Case where we pass in empty public inputs - this will be used in tests where they are not required
    if (public_inputs_vec.empty()) {
        return public_inputs;
    }

    // Convert the public inputs into the VmPublicInputs object, the public inputs vec must be the correct length - else
    // we throw an exception
    if (public_inputs_vec.size() != PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH) {
        throw_or_abort("Public inputs vector is not of PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH");
    }

    std::array<FF, KERNEL_INPUTS_LENGTH>& kernel_inputs = std::get<KERNEL_INPUTS>(public_inputs);

    // Copy the call context items
    kernel_inputs[SENDER_SELECTOR] = public_inputs_vec[SENDER_SELECTOR];   // Sender
    kernel_inputs[ADDRESS_SELECTOR] = public_inputs_vec[ADDRESS_SELECTOR]; // Address

    // Global variables
    kernel_inputs[CHAIN_ID_SELECTOR] = public_inputs_vec[CHAIN_ID_OFFSET];         // Chain ID
    kernel_inputs[VERSION_SELECTOR] = public_inputs_vec[VERSION_OFFSET];           // Version
    kernel_inputs[BLOCK_NUMBER_SELECTOR] = public_inputs_vec[BLOCK_NUMBER_OFFSET]; // Block Number
    kernel_inputs[TIMESTAMP_SELECTOR] = public_inputs_vec[TIMESTAMP_OFFSET];       // Timestamp
    kernel_inputs[COINBASE_SELECTOR] = public_inputs_vec[COINBASE_OFFSET];         // Coinbase

    // Fees
    kernel_inputs[FEE_PER_DA_GAS_SELECTOR] = public_inputs_vec[FEE_PER_DA_GAS_OFFSET];
    kernel_inputs[FEE_PER_L2_GAS_SELECTOR] = public_inputs_vec[FEE_PER_L2_GAS_OFFSET];

    // Transaction fee
    kernel_inputs[TRANSACTION_FEE_SELECTOR] = public_inputs_vec[TRANSACTION_FEE_OFFSET];

    kernel_inputs[DA_GAS_LEFT_CONTEXT_INPUTS_OFFSET] = public_inputs_vec[DA_GAS_LEFT_CONTEXT_INPUTS_OFFSET];
    kernel_inputs[L2_GAS_LEFT_CONTEXT_INPUTS_OFFSET] = public_inputs_vec[L2_GAS_LEFT_CONTEXT_INPUTS_OFFSET];

    return public_inputs;
}

bool Execution::verify(AvmFlavor::VerificationKey vk, HonkProof const& proof)
{
    auto verification_key = std::make_shared<AvmFlavor::VerificationKey>(vk);
    AvmVerifier verifier(verification_key);

    // todo: not needed for now until we verify the PCS/pairing of the proof
    // auto pcs_verification_key = std::make_unique<VerifierCommitmentKey>(verification_key->circuit_size,
    // crs_factory_);
    // output_state.pcs_verification_key = std::move(pcs_verification_key);

    // TODO: We hardcode public inputs for now
    VmPublicInputs public_inputs = convert_public_inputs(getDefaultPublicInputs());
    std::vector<std::vector<FF>> public_inputs_vec = copy_public_inputs_columns(public_inputs);
    return verifier.verify_proof(proof, public_inputs_vec);
}

/**
 * @brief Generate the execution trace pertaining to the supplied instructions.
 *
 * @param instructions A vector of the instructions to be executed.
 * @param calldata expressed as a vector of finite field elements.
 * @param public_inputs expressed as a vector of finite field elements.
 * @return The trace as a vector of Row.
 */
std::vector<Row> Execution::gen_trace(std::vector<Instruction> const& instructions,
                                      std::vector<FF> const& calldata,
                                      std::vector<FF> const& public_inputs)
{
    std::vector<FF> returndata{};
    return gen_trace(instructions, returndata, calldata, public_inputs);
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
    std::vector<FF> returndata{};
    std::vector<FF> public_inputs_vec = {};
    return gen_trace(instructions, returndata, calldata, public_inputs_vec);
}

/**
 * @brief Generate the execution trace pertaining to the supplied instructions returns the return data.
 *
 * @param instructions A vector of the instructions to be executed.
 * @param calldata expressed as a vector of finite field elements.
 * @param public_inputs expressed as a vector of finite field elements.
 * @return The trace as a vector of Row.
 */
std::vector<Row> Execution::gen_trace(std::vector<Instruction> const& instructions,
                                      std::vector<FF>& returndata,
                                      std::vector<FF> const& calldata,
                                      std::vector<FF> const& public_inputs_vec)

{
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/6718): construction of the public input columns
    // should be done in the kernel - this is stubbed and underconstrained
    VmPublicInputs public_inputs = convert_public_inputs(public_inputs_vec);
    AvmTraceBuilder trace_builder(public_inputs);

    // Copied version of pc maintained in trace builder. The value of pc is evolving based
    // on opcode logic and therefore is not maintained here. However, the next opcode in the execution
    // is determined by this value which require read access to the code below.
    uint32_t pc = 0;
    while ((pc = trace_builder.getPc()) < instructions.size()) {
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
        case OpCode::FDIV:
            trace_builder.op_fdiv(std::get<uint8_t>(inst.operands.at(0)),
                                  std::get<uint32_t>(inst.operands.at(1)),
                                  std::get<uint32_t>(inst.operands.at(2)),
                                  std::get<uint32_t>(inst.operands.at(3)));
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
        case OpCode::LT:
            trace_builder.op_lt(std::get<uint8_t>(inst.operands.at(0)),
                                std::get<uint32_t>(inst.operands.at(2)),
                                std::get<uint32_t>(inst.operands.at(3)),
                                std::get<uint32_t>(inst.operands.at(4)),
                                std::get<AvmMemoryTag>(inst.operands.at(1)));
            break;
        case OpCode::LTE:
            trace_builder.op_lte(std::get<uint8_t>(inst.operands.at(0)),
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
        case OpCode::SHR:
            trace_builder.op_shr(std::get<uint8_t>(inst.operands.at(0)),
                                 std::get<uint32_t>(inst.operands.at(2)),
                                 std::get<uint32_t>(inst.operands.at(3)),
                                 std::get<uint32_t>(inst.operands.at(4)),
                                 std::get<AvmMemoryTag>(inst.operands.at(1)));
            break;
        case OpCode::SHL:
            trace_builder.op_shl(std::get<uint8_t>(inst.operands.at(0)),
                                 std::get<uint32_t>(inst.operands.at(2)),
                                 std::get<uint32_t>(inst.operands.at(3)),
                                 std::get<uint32_t>(inst.operands.at(4)),
                                 std::get<AvmMemoryTag>(inst.operands.at(1)));
            break;
            // Compute - Type Conversions
        case OpCode::CAST:
            trace_builder.op_cast(std::get<uint8_t>(inst.operands.at(0)),
                                  std::get<uint32_t>(inst.operands.at(2)),
                                  std::get<uint32_t>(inst.operands.at(3)),
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

        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/6284): support indirect for below
        case OpCode::SENDER:
            trace_builder.op_sender(std::get<uint32_t>(inst.operands.at(1)));
            break;
        case OpCode::ADDRESS:
            trace_builder.op_address(std::get<uint32_t>(inst.operands.at(1)));
            break;
        case OpCode::FEEPERL2GAS:
            trace_builder.op_fee_per_l2_gas(std::get<uint32_t>(inst.operands.at(1)));
            break;
        case OpCode::FEEPERDAGAS:
            trace_builder.op_fee_per_da_gas(std::get<uint32_t>(inst.operands.at(1)));
            break;
        case OpCode::TRANSACTIONFEE:
            trace_builder.op_transaction_fee(std::get<uint32_t>(inst.operands.at(1)));
            break;
        case OpCode::CHAINID:
            trace_builder.op_chain_id(std::get<uint32_t>(inst.operands.at(1)));
            break;
        case OpCode::VERSION:
            trace_builder.op_version(std::get<uint32_t>(inst.operands.at(1)));
            break;
        case OpCode::BLOCKNUMBER:
            trace_builder.op_block_number(std::get<uint32_t>(inst.operands.at(1)));
            break;
        case OpCode::COINBASE:
            trace_builder.op_coinbase(std::get<uint32_t>(inst.operands.at(1)));
            break;
        case OpCode::TIMESTAMP:
            trace_builder.op_timestamp(std::get<uint32_t>(inst.operands.at(1)));
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
        case OpCode::CMOV:
            trace_builder.op_cmov(std::get<uint8_t>(inst.operands.at(0)),
                                  std::get<uint32_t>(inst.operands.at(1)),
                                  std::get<uint32_t>(inst.operands.at(2)),
                                  std::get<uint32_t>(inst.operands.at(3)),
                                  std::get<uint32_t>(inst.operands.at(4)));
            break;
            // Control Flow - Contract Calls
        case OpCode::RETURN: {
            auto ret = trace_builder.return_op(std::get<uint8_t>(inst.operands.at(0)),
                                               std::get<uint32_t>(inst.operands.at(1)),
                                               std::get<uint32_t>(inst.operands.at(2)));
            returndata.insert(returndata.end(), ret.begin(), ret.end());
            break;
        }
        case OpCode::DEBUGLOG:
            // We want a noop, but we need to execute something that both advances the PC,
            // and adds a valid row to the trace.
            trace_builder.jump(pc + 1);
            break;
        case OpCode::TORADIXLE:
            trace_builder.op_to_radix_le(std::get<uint8_t>(inst.operands.at(0)),
                                         std::get<uint32_t>(inst.operands.at(1)),
                                         std::get<uint32_t>(inst.operands.at(2)),
                                         std::get<uint32_t>(inst.operands.at(3)),
                                         std::get<uint32_t>(inst.operands.at(4)));
            break;
        case OpCode::SHA256COMPRESSION:
            trace_builder.op_sha256_compression(std::get<uint8_t>(inst.operands.at(0)),
                                                std::get<uint32_t>(inst.operands.at(1)),
                                                std::get<uint32_t>(inst.operands.at(2)),
                                                std::get<uint32_t>(inst.operands.at(3)));
            break;
        case OpCode::SHA256:
            trace_builder.op_sha256(std::get<uint8_t>(inst.operands.at(0)),
                                    std::get<uint32_t>(inst.operands.at(1)),
                                    std::get<uint32_t>(inst.operands.at(2)),
                                    std::get<uint32_t>(inst.operands.at(3)));
            break;
        case OpCode::POSEIDON2:
            trace_builder.op_poseidon2_permutation(std::get<uint8_t>(inst.operands.at(0)),
                                                   std::get<uint32_t>(inst.operands.at(1)),
                                                   std::get<uint32_t>(inst.operands.at(2)));

            break;
        case OpCode::KECCAK:
            trace_builder.op_keccak(std::get<uint8_t>(inst.operands.at(0)),
                                    std::get<uint32_t>(inst.operands.at(1)),
                                    std::get<uint32_t>(inst.operands.at(2)),
                                    std::get<uint32_t>(inst.operands.at(3)));

            break;
        case OpCode::KECCAKF1600:
            trace_builder.op_keccakf1600(std::get<uint8_t>(inst.operands.at(0)),
                                         std::get<uint32_t>(inst.operands.at(1)),
                                         std::get<uint32_t>(inst.operands.at(2)),
                                         std::get<uint32_t>(inst.operands.at(3)));

            break;
        case OpCode::PEDERSEN:
            trace_builder.op_pedersen_hash(std::get<uint8_t>(inst.operands.at(0)),
                                           std::get<uint32_t>(inst.operands.at(1)),
                                           std::get<uint32_t>(inst.operands.at(2)),
                                           std::get<uint32_t>(inst.operands.at(3)),
                                           std::get<uint32_t>(inst.operands.at(4)));
            break;
        default:
            throw_or_abort("Don't know how to execute opcode " + to_hex(inst.op_code) + " at pc " + std::to_string(pc) +
                           ".");
            break;
        }
    }

    return trace_builder.finalize();
}

} // namespace bb::avm_trace
