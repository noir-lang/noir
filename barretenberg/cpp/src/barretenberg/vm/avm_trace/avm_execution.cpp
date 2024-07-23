#include "barretenberg/vm/avm_trace/avm_execution.hpp"
#include "barretenberg/bb/log.hpp"
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/common/throw_or_abort.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/vm/avm_trace/avm_common.hpp"
#include "barretenberg/vm/avm_trace/avm_deserialization.hpp"
#include "barretenberg/vm/avm_trace/avm_helper.hpp"
#include "barretenberg/vm/avm_trace/avm_instructions.hpp"
#include "barretenberg/vm/avm_trace/avm_kernel_trace.hpp"
#include "barretenberg/vm/avm_trace/avm_opcode.hpp"
#include "barretenberg/vm/avm_trace/avm_trace.hpp"
#include "barretenberg/vm/avm_trace/aztec_constants.hpp"
#include "barretenberg/vm/avm_trace/constants.hpp"
#include "barretenberg/vm/avm_trace/stats.hpp"
#include "barretenberg/vm/generated/avm_circuit_builder.hpp"
#include "barretenberg/vm/generated/avm_composer.hpp"
#include "barretenberg/vm/generated/avm_flavor.hpp"

#include <cassert>
#include <cstddef>
#include <cstdint>
#include <filesystem>
#include <string>
#include <tuple>
#include <unordered_map>
#include <variant>
#include <vector>

using namespace bb;

// Set in BB's main.cpp.
std::filesystem::path avm_dump_trace_path;

namespace bb::avm_trace {
namespace {

template <typename K, typename V>
std::vector<std::pair<K, V>> sorted_entries(const std::unordered_map<K, V>& map, bool invert = false)
{
    std::vector<std::pair<K, V>> entries;
    entries.reserve(map.size());
    for (const auto& [key, value] : map) {
        entries.emplace_back(key, value);
    }
    std::sort(entries.begin(), entries.end(), [invert](const auto& a, const auto& b) {
        bool r = a.first < b.first;
        if (invert) {
            r = !r;
        }
        return r;
    });
    return entries;
}

// Returns degree distribution information for main relations (e.g., not lookups/perms).
std::unordered_map</*relation*/ std::string, /*degrees*/ std::string> get_relations_degrees()
{
    std::unordered_map<std::string, std::string> relations_degrees;

    bb::constexpr_for<0, std::tuple_size_v<AvmFlavor::MainRelations>, 1>([&]<size_t i>() {
        std::unordered_map<int, int> degree_distribution;
        using Relation = std::tuple_element_t<i, AvmFlavor::Relations>;
        for (const auto& len : Relation::SUBRELATION_PARTIAL_LENGTHS) {
            degree_distribution[static_cast<int>(len - 1)]++;
        }
        std::string degrees_string;
        auto entries = sorted_entries(degree_distribution, /*invert=*/true);
        for (size_t n = 0; n < entries.size(); n++) {
            const auto& [degree, count] = entries[n];
            if (n > 0) {
                degrees_string += ", ";
            }
            degrees_string += std::to_string(degree) + "°: " + std::to_string(count);
        }
        relations_degrees.insert({ Relation::NAME, std::move(degrees_string) });
    });

    return relations_degrees;
}

} // namespace

/**
 * @brief Temporary routine to generate default public inputs (gas values) until we get
 *        proper integration of public inputs.
 */
std::vector<FF> Execution::getDefaultPublicInputs()
{
    std::vector<FF> public_inputs_vec(PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH);
    public_inputs_vec.at(DA_START_GAS_LEFT_PCPI_OFFSET) = 1000000000;
    public_inputs_vec.at(L2_START_GAS_LEFT_PCPI_OFFSET) = 1000000000;
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
                                                                   std::vector<FF> const& calldata,
                                                                   std::vector<FF> const& public_inputs_vec,
                                                                   ExecutionHints const& execution_hints)
{
    if (public_inputs_vec.size() != PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH) {
        throw_or_abort("Public inputs vector is not of PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH");
    }

    auto instructions = Deserialization::parse(bytecode);
    vinfo("Deserialized " + std::to_string(instructions.size()) + " instructions");

    std::vector<FF> returndata;
    std::vector<Row> trace;
    AVM_TRACK_TIME("prove/gen_trace",
                   (trace = gen_trace(instructions, returndata, calldata, public_inputs_vec, execution_hints)));
    if (!avm_dump_trace_path.empty()) {
        info("Dumping trace as CSV to: " + avm_dump_trace_path.string());
        dump_trace_as_csv(trace, avm_dump_trace_path);
    }
    auto circuit_builder = bb::AvmCircuitBuilder();
    circuit_builder.set_trace(std::move(trace));

    if (circuit_builder.get_circuit_subgroup_size() > SRS_SIZE) {
        throw_or_abort("Circuit subgroup size (" + std::to_string(circuit_builder.get_circuit_subgroup_size()) +
                       ") exceeds SRS_SIZE (" + std::to_string(SRS_SIZE) + ")");
    }

    AVM_TRACK_TIME("prove/check_circuit", circuit_builder.check_circuit());

    auto composer = AvmComposer();
    auto prover = composer.create_prover(circuit_builder);
    auto verifier = composer.create_verifier(circuit_builder);

    vinfo("------- PROVING EXECUTION -------");
    // Proof structure: public_inputs | calldata_size | calldata | returndata_size | returndata | raw proof
    HonkProof proof(public_inputs_vec);
    proof.emplace_back(calldata.size());
    proof.insert(proof.end(), calldata.begin(), calldata.end());
    proof.emplace_back(returndata.size());
    proof.insert(proof.end(), returndata.begin(), returndata.end());
    auto raw_proof = prover.construct_proof();
    proof.insert(proof.end(), raw_proof.begin(), raw_proof.end());
    // TODO(#4887): Might need to return PCS vk when full verify is supported
    return std::make_tuple(*verifier.key, proof);
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
                                      std::vector<FF> const& public_inputs,
                                      ExecutionHints const& execution_hints)
{
    std::vector<FF> returndata{};
    return gen_trace(instructions, returndata, calldata, public_inputs, execution_hints);
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
    VmPublicInputs public_inputs;

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

    // Copy items from PublicCircuitPublicInputs vector to public input columns
    // PublicCircuitPublicInputs - CallContext
    kernel_inputs[SENDER_SELECTOR] = public_inputs_vec[SENDER_SELECTOR]; // Sender
    // NOTE: address has same position as storage address (they are the same for now...)
    // kernel_inputs[ADDRESS_SELECTOR] = public_inputs_vec[ADDRESS_SELECTOR];                 // Address
    kernel_inputs[STORAGE_ADDRESS_SELECTOR] = public_inputs_vec[STORAGE_ADDRESS_SELECTOR]; // Storage Address
    kernel_inputs[FUNCTION_SELECTOR_SELECTOR] = public_inputs_vec[FUNCTION_SELECTOR_SELECTOR];

    // PublicCircuitPublicInputs - GlobalVariables
    kernel_inputs[CHAIN_ID_SELECTOR] = public_inputs_vec[CHAIN_ID_OFFSET];         // Chain ID
    kernel_inputs[VERSION_SELECTOR] = public_inputs_vec[VERSION_OFFSET];           // Version
    kernel_inputs[BLOCK_NUMBER_SELECTOR] = public_inputs_vec[BLOCK_NUMBER_OFFSET]; // Block Number
    kernel_inputs[TIMESTAMP_SELECTOR] = public_inputs_vec[TIMESTAMP_OFFSET];       // Timestamp
    kernel_inputs[COINBASE_SELECTOR] = public_inputs_vec[COINBASE_OFFSET];         // Coinbase
    // PublicCircuitPublicInputs - GlobalVariables - GasFees
    kernel_inputs[FEE_PER_DA_GAS_SELECTOR] = public_inputs_vec[FEE_PER_DA_GAS_OFFSET];
    kernel_inputs[FEE_PER_L2_GAS_SELECTOR] = public_inputs_vec[FEE_PER_L2_GAS_OFFSET];

    // Transaction fee
    kernel_inputs[TRANSACTION_FEE_SELECTOR] = public_inputs_vec[TRANSACTION_FEE_OFFSET];

    kernel_inputs[DA_GAS_LEFT_CONTEXT_INPUTS_OFFSET] = public_inputs_vec[DA_START_GAS_LEFT_PCPI_OFFSET];
    kernel_inputs[L2_GAS_LEFT_CONTEXT_INPUTS_OFFSET] = public_inputs_vec[L2_START_GAS_LEFT_PCPI_OFFSET];

    // Copy the output columns
    std::array<FF, KERNEL_OUTPUTS_LENGTH>& ko_values = std::get<KERNEL_OUTPUTS_VALUE>(public_inputs);
    std::array<FF, KERNEL_OUTPUTS_LENGTH>& ko_side_effect = std::get<KERNEL_OUTPUTS_SIDE_EFFECT_COUNTER>(public_inputs);
    std::array<FF, KERNEL_OUTPUTS_LENGTH>& ko_metadata = std::get<KERNEL_OUTPUTS_METADATA>(public_inputs);

    // We copy each type of the kernel outputs into their respective columns, each has differeing lengths / data
    // For NOTEHASHEXISTS
    for (size_t i = 0; i < MAX_NOTE_HASH_READ_REQUESTS_PER_CALL; i++) {
        size_t dest_offset = START_NOTE_HASH_EXISTS_WRITE_OFFSET + i;
        size_t pcpi_offset = PCPI_NOTE_HASH_EXISTS_OFFSET + (i * READ_REQUEST_LENGTH);

        ko_values[dest_offset] = public_inputs_vec[pcpi_offset];
        ko_side_effect[dest_offset] = public_inputs_vec[pcpi_offset + 1];
    }
    // For NULLIFIEREXISTS
    for (size_t i = 0; i < MAX_NULLIFIER_READ_REQUESTS_PER_CALL; i++) {
        size_t dest_offset = START_NULLIFIER_EXISTS_OFFSET + i;
        size_t pcpi_offset = PCPI_NULLIFIER_EXISTS_OFFSET + (i * READ_REQUEST_LENGTH);

        ko_values[dest_offset] = public_inputs_vec[pcpi_offset];
        ko_side_effect[dest_offset] = public_inputs_vec[pcpi_offset + 1];
        ko_metadata[dest_offset] = FF(1);
    }
    // For NULLIFIEREXISTS - non existent
    for (size_t i = 0; i < MAX_NULLIFIER_NON_EXISTENT_READ_REQUESTS_PER_CALL; i++) {
        size_t dest_offset = START_NULLIFIER_NON_EXISTS_OFFSET + i;
        size_t pcpi_offset = PCPI_NULLIFIER_NON_EXISTS_OFFSET + (i * READ_REQUEST_LENGTH);

        ko_values[dest_offset] = public_inputs_vec[pcpi_offset];
        ko_side_effect[dest_offset] = public_inputs_vec[pcpi_offset + 1];
        ko_metadata[dest_offset] = FF(0);
    }
    // For L1TOL2MSGEXISTS
    for (size_t i = 0; i < MAX_L1_TO_L2_MSG_READ_REQUESTS_PER_CALL; i++) {
        size_t dest_offset = START_L1_TO_L2_MSG_EXISTS_WRITE_OFFSET + i;
        size_t pcpi_offset = PCPI_L1_TO_L2_MSG_READ_REQUESTS_OFFSET + (i * READ_REQUEST_LENGTH);

        ko_values[dest_offset] = public_inputs_vec[pcpi_offset];
        ko_side_effect[dest_offset] = public_inputs_vec[pcpi_offset + 1];
    }
    // For SSTORE
    for (size_t i = 0; i < MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL; i++) {
        size_t dest_offset = START_SSTORE_WRITE_OFFSET + i;
        size_t pcpi_offset = PCPI_PUBLIC_DATA_UPDATE_OFFSET + (i * CONTRACT_STORAGE_UPDATE_REQUEST_LENGTH);

        // slot, value, side effect
        ko_metadata[dest_offset] = public_inputs_vec[pcpi_offset];
        ko_values[dest_offset] = public_inputs_vec[pcpi_offset + 1];
        ko_side_effect[dest_offset] = public_inputs_vec[pcpi_offset + 2];
    }
    // For SLOAD
    for (size_t i = 0; i < MAX_PUBLIC_DATA_READS_PER_CALL; i++) {
        size_t dest_offset = START_SLOAD_WRITE_OFFSET + i;
        size_t pcpi_offset = PCPI_PUBLIC_DATA_READ_OFFSET + (i * CONTRACT_STORAGE_READ_LENGTH);

        // slot, value, side effect
        ko_metadata[dest_offset] = public_inputs_vec[pcpi_offset];
        ko_values[dest_offset] = public_inputs_vec[pcpi_offset + 1];
        ko_side_effect[dest_offset] = public_inputs_vec[pcpi_offset + 2];
    }
    // For EMITNOTEHASH
    for (size_t i = 0; i < MAX_NOTE_HASHES_PER_CALL; i++) {
        size_t dest_offset = START_EMIT_NOTE_HASH_WRITE_OFFSET + i;
        size_t pcpi_offset = PCPI_NEW_NOTE_HASHES_OFFSET + (i * NOTE_HASH_LENGTH);

        ko_values[dest_offset] = public_inputs_vec[pcpi_offset];
        ko_side_effect[dest_offset] = public_inputs_vec[pcpi_offset + 1];
    }
    // For EMITNULLIFIER
    for (size_t i = 0; i < MAX_NULLIFIERS_PER_CALL; i++) {
        size_t dest_offset = START_EMIT_NULLIFIER_WRITE_OFFSET + i;
        size_t pcpi_offset = PCPI_NEW_NULLIFIERS_OFFSET + (i * NULLIFIER_LENGTH);

        ko_values[dest_offset] = public_inputs_vec[pcpi_offset];
        ko_side_effect[dest_offset] = public_inputs_vec[pcpi_offset + 1];
    }
    // For EMITL2TOL1MSG
    for (size_t i = 0; i < MAX_L2_TO_L1_MSGS_PER_CALL; i++) {
        size_t dest_offset = START_EMIT_L2_TO_L1_MSG_WRITE_OFFSET + i;
        size_t pcpi_offset = PCPI_NEW_L2_TO_L1_MSGS_OFFSET + (i * L2_TO_L1_MESSAGE_LENGTH);

        // Note: unorthadox order
        ko_metadata[dest_offset] = public_inputs_vec[pcpi_offset];
        ko_values[dest_offset] = public_inputs_vec[pcpi_offset + 1];
        ko_side_effect[dest_offset] = public_inputs_vec[pcpi_offset + 2];
    }
    // For EMITUNENCRYPTEDLOG
    for (size_t i = 0; i < MAX_UNENCRYPTED_LOGS_PER_CALL; i++) {
        size_t dest_offset = START_EMIT_UNENCRYPTED_LOG_WRITE_OFFSET + i;
        size_t pcpi_offset = PCPI_NEW_UNENCRYPTED_LOGS_OFFSET + (i * 2);

        ko_values[dest_offset] = public_inputs_vec[pcpi_offset];
        ko_side_effect[dest_offset] = public_inputs_vec[pcpi_offset + 1];
    }

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

    // Proof structure: public_inputs | calldata_size | calldata | returndata_size | returndata | raw proof
    std::vector<FF> public_inputs_vec;
    std::vector<FF> calldata;
    std::vector<FF> returndata;
    std::vector<FF> raw_proof;

    // This can be made nicer using BB's serialize::read, probably.
    const auto public_inputs_offset = proof.begin();
    const auto calldata_size_offset = public_inputs_offset + PUBLIC_CIRCUIT_PUBLIC_INPUTS_LENGTH;
    const auto calldata_offset = calldata_size_offset + 1;
    const auto returndata_size_offset = calldata_offset + static_cast<int64_t>(uint64_t(*calldata_size_offset));
    const auto returndata_offset = returndata_size_offset + 1;
    const auto raw_proof_offset = returndata_offset + static_cast<int64_t>(uint64_t(*returndata_size_offset));

    std::copy(public_inputs_offset, calldata_size_offset, std::back_inserter(public_inputs_vec));
    std::copy(calldata_offset, returndata_size_offset, std::back_inserter(calldata));
    std::copy(returndata_offset, raw_proof_offset, std::back_inserter(returndata));
    std::copy(raw_proof_offset, proof.end(), std::back_inserter(raw_proof));

    VmPublicInputs public_inputs = convert_public_inputs(public_inputs_vec);
    std::vector<std::vector<FF>> public_inputs_columns =
        copy_public_inputs_columns(public_inputs, calldata, returndata);
    return verifier.verify_proof(raw_proof, public_inputs_columns);
}

/**
 * @brief Generate the execution trace pertaining to the supplied instructions.
 *
 * @param instructions A vector of the instructions to be executed.
 * @param calldata expressed as a vector of finite field elements.
 * @return The trace as a vector of Row.
 */
std::vector<Row> Execution::gen_trace(std::vector<Instruction> const& instructions,
                                      std::vector<FF> const& calldata,
                                      std::vector<FF> const& public_inputs_vec)
{
    std::vector<FF> returndata{};
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
                                      std::vector<FF> const& public_inputs_vec,
                                      ExecutionHints const& execution_hints)

{
    vinfo("------- GENERATING TRACE -------");
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/6718): construction of the public input columns
    // should be done in the kernel - this is stubbed and underconstrained
    VmPublicInputs public_inputs = convert_public_inputs(public_inputs_vec);
    uint32_t start_side_effect_counter =
        !public_inputs_vec.empty() ? static_cast<uint32_t>(public_inputs_vec[PCPI_START_SIDE_EFFECT_COUNTER_OFFSET])
                                   : 0;
    AvmTraceBuilder trace_builder(public_inputs, execution_hints, start_side_effect_counter, calldata);

    // Copied version of pc maintained in trace builder. The value of pc is evolving based
    // on opcode logic and therefore is not maintained here. However, the next opcode in the execution
    // is determined by this value which require read access to the code below.
    uint32_t pc = 0;
    while ((pc = trace_builder.getPc()) < instructions.size()) {
        auto inst = instructions.at(pc);
        debug("[@" + std::to_string(pc) + "] " + inst.to_string());

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
        case OpCode::FDIV:
            trace_builder.op_fdiv(std::get<uint8_t>(inst.operands.at(0)),
                                  std::get<uint32_t>(inst.operands.at(1)),
                                  std::get<uint32_t>(inst.operands.at(2)),
                                  std::get<uint32_t>(inst.operands.at(3)));
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
        case OpCode::NOT:
            trace_builder.op_not(std::get<uint8_t>(inst.operands.at(0)),
                                 std::get<uint32_t>(inst.operands.at(2)),
                                 std::get<uint32_t>(inst.operands.at(3)),
                                 std::get<AvmMemoryTag>(inst.operands.at(1)));
            break;
        case OpCode::SHL:
            trace_builder.op_shl(std::get<uint8_t>(inst.operands.at(0)),
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

            // Compute - Type Conversions
        case OpCode::CAST:
            trace_builder.op_cast(std::get<uint8_t>(inst.operands.at(0)),
                                  std::get<uint32_t>(inst.operands.at(2)),
                                  std::get<uint32_t>(inst.operands.at(3)),
                                  std::get<AvmMemoryTag>(inst.operands.at(1)));
            break;

            // Execution Environment
            // TODO(https://github.com/AztecProtocol/aztec-packages/issues/6284): support indirect for below
        case OpCode::ADDRESS:
            trace_builder.op_address(std::get<uint8_t>(inst.operands.at(0)), std::get<uint32_t>(inst.operands.at(1)));
            break;
        case OpCode::STORAGEADDRESS:
            trace_builder.op_storage_address(std::get<uint8_t>(inst.operands.at(0)),
                                             std::get<uint32_t>(inst.operands.at(1)));
            break;
        case OpCode::SENDER:
            trace_builder.op_sender(std::get<uint8_t>(inst.operands.at(0)), std::get<uint32_t>(inst.operands.at(1)));
            break;
        case OpCode::FUNCTIONSELECTOR:
            trace_builder.op_function_selector(std::get<uint8_t>(inst.operands.at(0)),
                                               std::get<uint32_t>(inst.operands.at(1)));
            break;
        case OpCode::TRANSACTIONFEE:
            trace_builder.op_transaction_fee(std::get<uint8_t>(inst.operands.at(0)),
                                             std::get<uint32_t>(inst.operands.at(1)));
            break;

            // Execution Environment - Globals
        case OpCode::CHAINID:
            trace_builder.op_chain_id(std::get<uint8_t>(inst.operands.at(0)), std::get<uint32_t>(inst.operands.at(1)));
            break;
        case OpCode::VERSION:
            trace_builder.op_version(std::get<uint8_t>(inst.operands.at(0)), std::get<uint32_t>(inst.operands.at(1)));
            break;
        case OpCode::BLOCKNUMBER:
            trace_builder.op_block_number(std::get<uint8_t>(inst.operands.at(0)),
                                          std::get<uint32_t>(inst.operands.at(1)));
            break;
        case OpCode::TIMESTAMP:
            trace_builder.op_timestamp(std::get<uint8_t>(inst.operands.at(0)), std::get<uint32_t>(inst.operands.at(1)));
            break;
        case OpCode::COINBASE:
            trace_builder.op_coinbase(std::get<uint8_t>(inst.operands.at(0)), std::get<uint32_t>(inst.operands.at(1)));
            break;
        case OpCode::FEEPERL2GAS:
            trace_builder.op_fee_per_l2_gas(std::get<uint8_t>(inst.operands.at(0)),
                                            std::get<uint32_t>(inst.operands.at(1)));
            break;
        case OpCode::FEEPERDAGAS:
            trace_builder.op_fee_per_da_gas(std::get<uint8_t>(inst.operands.at(0)),
                                            std::get<uint32_t>(inst.operands.at(1)));
            break;

            // Execution Environment - Calldata
        case OpCode::CALLDATACOPY:
            trace_builder.op_calldata_copy(std::get<uint8_t>(inst.operands.at(0)),
                                           std::get<uint32_t>(inst.operands.at(1)),
                                           std::get<uint32_t>(inst.operands.at(2)),
                                           std::get<uint32_t>(inst.operands.at(3)));
            break;

            // Machine State - Gas
        case OpCode::L2GASLEFT:
            trace_builder.op_l2gasleft(std::get<uint8_t>(inst.operands.at(0)), std::get<uint32_t>(inst.operands.at(1)));
            break;
        case OpCode::DAGASLEFT:
            trace_builder.op_dagasleft(std::get<uint8_t>(inst.operands.at(0)), std::get<uint32_t>(inst.operands.at(1)));
            break;

            // Machine State - Internal Control Flow
        case OpCode::JUMP:
            trace_builder.op_jump(std::get<uint32_t>(inst.operands.at(0)));
            break;
        case OpCode::JUMPI:
            trace_builder.op_jumpi(std::get<uint8_t>(inst.operands.at(0)),
                                   std::get<uint32_t>(inst.operands.at(1)),
                                   std::get<uint32_t>(inst.operands.at(2)));
            break;
        case OpCode::INTERNALCALL:
            trace_builder.op_internal_call(std::get<uint32_t>(inst.operands.at(0)));
            break;
        case OpCode::INTERNALRETURN:
            trace_builder.op_internal_return();
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

            // World State
        case OpCode::SLOAD:
            trace_builder.op_sload(std::get<uint8_t>(inst.operands.at(0)),
                                   std::get<uint32_t>(inst.operands.at(1)),
                                   std::get<uint32_t>(inst.operands.at(2)),
                                   std::get<uint32_t>(inst.operands.at(3)));
            break;
        case OpCode::SSTORE:
            trace_builder.op_sstore(std::get<uint8_t>(inst.operands.at(0)),
                                    std::get<uint32_t>(inst.operands.at(1)),
                                    std::get<uint32_t>(inst.operands.at(2)),
                                    std::get<uint32_t>(inst.operands.at(3)));
            break;
        case OpCode::NOTEHASHEXISTS:
            trace_builder.op_note_hash_exists(std::get<uint8_t>(inst.operands.at(0)),
                                              std::get<uint32_t>(inst.operands.at(1)),
                                              // TODO: leaf offset exists
                                              // std::get<uint32_t>(inst.operands.at(2))
                                              std::get<uint32_t>(inst.operands.at(3)));
            break;
        case OpCode::EMITNOTEHASH:
            trace_builder.op_emit_note_hash(std::get<uint8_t>(inst.operands.at(0)),
                                            std::get<uint32_t>(inst.operands.at(1)));
            break;
        case OpCode::NULLIFIEREXISTS:
            trace_builder.op_nullifier_exists(std::get<uint8_t>(inst.operands.at(0)),
                                              std::get<uint32_t>(inst.operands.at(1)),
                                              // std::get<uint32_t>(inst.operands.at(2))
                                              /**TODO: Address offset for siloing */
                                              std::get<uint32_t>(inst.operands.at(3)));
            break;
        case OpCode::EMITNULLIFIER:
            trace_builder.op_emit_nullifier(std::get<uint8_t>(inst.operands.at(0)),
                                            std::get<uint32_t>(inst.operands.at(1)));
            break;

        case OpCode::L1TOL2MSGEXISTS:
            trace_builder.op_l1_to_l2_msg_exists(std::get<uint8_t>(inst.operands.at(0)),
                                                 std::get<uint32_t>(inst.operands.at(1)),
                                                 // TODO: leaf offset exists
                                                 // std::get<uint32_t>(inst.operands.at(2))
                                                 std::get<uint32_t>(inst.operands.at(3)));
            break;
        case OpCode::GETCONTRACTINSTANCE:
            trace_builder.op_get_contract_instance(std::get<uint8_t>(inst.operands.at(0)),
                                                   std::get<uint32_t>(inst.operands.at(1)),
                                                   std::get<uint32_t>(inst.operands.at(2)));
            break;

            // Accrued Substate
        case OpCode::EMITUNENCRYPTEDLOG:
            trace_builder.op_emit_unencrypted_log(std::get<uint8_t>(inst.operands.at(0)),
                                                  std::get<uint32_t>(inst.operands.at(1)),
                                                  std::get<uint32_t>(inst.operands.at(2)));
            break;
        case OpCode::SENDL2TOL1MSG:
            trace_builder.op_emit_l2_to_l1_msg(std::get<uint8_t>(inst.operands.at(0)),
                                               std::get<uint32_t>(inst.operands.at(1)),
                                               std::get<uint32_t>(inst.operands.at(2)));
            break;

            // Control Flow - Contract Calls
        case OpCode::CALL:
            trace_builder.op_call(std::get<uint8_t>(inst.operands.at(0)),
                                  std::get<uint32_t>(inst.operands.at(1)),
                                  std::get<uint32_t>(inst.operands.at(2)),
                                  std::get<uint32_t>(inst.operands.at(3)),
                                  std::get<uint32_t>(inst.operands.at(4)),
                                  std::get<uint32_t>(inst.operands.at(5)),
                                  std::get<uint32_t>(inst.operands.at(6)),
                                  std::get<uint32_t>(inst.operands.at(7)),
                                  std::get<uint32_t>(inst.operands.at(8)));
            break;
        case OpCode::RETURN: {
            auto ret = trace_builder.op_return(std::get<uint8_t>(inst.operands.at(0)),
                                               std::get<uint32_t>(inst.operands.at(1)),
                                               std::get<uint32_t>(inst.operands.at(2)));
            returndata.insert(returndata.end(), ret.begin(), ret.end());

            break;
        }
        case OpCode::REVERT: {
            auto ret = trace_builder.op_revert(std::get<uint8_t>(inst.operands.at(0)),
                                               std::get<uint32_t>(inst.operands.at(1)),
                                               std::get<uint32_t>(inst.operands.at(2)));
            returndata.insert(returndata.end(), ret.begin(), ret.end());

            break;
        }

            // Misc
        case OpCode::DEBUGLOG:
            // We want a noop, but we need to execute something that both advances the PC,
            // and adds a valid row to the trace.
            trace_builder.op_jump(pc + 1);
            break;

            // Gadgets
        case OpCode::KECCAK:
            trace_builder.op_keccak(std::get<uint8_t>(inst.operands.at(0)),
                                    std::get<uint32_t>(inst.operands.at(1)),
                                    std::get<uint32_t>(inst.operands.at(2)),
                                    std::get<uint32_t>(inst.operands.at(3)));

            break;
        case OpCode::POSEIDON2:
            trace_builder.op_poseidon2_permutation(std::get<uint8_t>(inst.operands.at(0)),
                                                   std::get<uint32_t>(inst.operands.at(1)),
                                                   std::get<uint32_t>(inst.operands.at(2)));

            break;
        case OpCode::SHA256:
            trace_builder.op_sha256(std::get<uint8_t>(inst.operands.at(0)),
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
        case OpCode::ECADD:
            trace_builder.op_ec_add(std::get<uint8_t>(inst.operands.at(0)),
                                    std::get<uint32_t>(inst.operands.at(1)),
                                    std::get<uint32_t>(inst.operands.at(2)),
                                    std::get<uint32_t>(inst.operands.at(3)),
                                    std::get<uint32_t>(inst.operands.at(4)),
                                    std::get<uint32_t>(inst.operands.at(5)),
                                    std::get<uint32_t>(inst.operands.at(6)),
                                    std::get<uint32_t>(inst.operands.at(7)));
            break;
        case OpCode::MSM:
            trace_builder.op_variable_msm(std::get<uint8_t>(inst.operands.at(0)),
                                          std::get<uint32_t>(inst.operands.at(1)),
                                          std::get<uint32_t>(inst.operands.at(2)),
                                          std::get<uint32_t>(inst.operands.at(3)),
                                          std::get<uint32_t>(inst.operands.at(4)));
            break;

            // Conversions
        case OpCode::TORADIXLE:
            trace_builder.op_to_radix_le(std::get<uint8_t>(inst.operands.at(0)),
                                         std::get<uint32_t>(inst.operands.at(1)),
                                         std::get<uint32_t>(inst.operands.at(2)),
                                         std::get<uint32_t>(inst.operands.at(3)),
                                         std::get<uint32_t>(inst.operands.at(4)));
            break;

            // Future Gadgets -- pending changes in noir
        case OpCode::SHA256COMPRESSION:
            trace_builder.op_sha256_compression(std::get<uint8_t>(inst.operands.at(0)),
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
        default:
            throw_or_abort("Don't know how to execute opcode " + to_hex(inst.op_code) + " at pc " + std::to_string(pc) +
                           ".");
            break;
        }
    }

    auto trace = trace_builder.finalize();
    vinfo("Final trace size: ", trace.size());
    vinfo("Number of columns: ", trace.front().SIZE);
    const size_t total_elements = trace.front().SIZE * trace.size();
    const size_t nonzero_elements = [&]() {
        size_t count = 0;
        for (auto const& row : trace) {
            for (const auto& ff : row.as_vector()) {
                if (!ff.is_zero()) {
                    count++;
                }
            }
        }
        return count;
    }();
    vinfo("Number of non-zero elements: ",
          nonzero_elements,
          "/",
          total_elements,
          " (",
          100 * nonzero_elements / total_elements,
          "%)");
    vinfo("Relation degrees: ", []() {
        std::string result;
        for (const auto& [key, value] : sorted_entries(get_relations_degrees())) {
            result += "\n\t" + key + ": [" + value + "]";
        }
        return result;
    }());

    return trace;
}

} // namespace bb::avm_trace
