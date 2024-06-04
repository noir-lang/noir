#pragma once

#include "barretenberg/serialize/msgpack.hpp"
#include "barretenberg/vm/avm_trace/constants.hpp"
#include "barretenberg/vm/generated/avm_flavor.hpp"

#include <array>
#include <cstdint>
#include <unordered_map>

namespace bb::avm_trace {

using Flavor = bb::AvmFlavor;
using FF = Flavor::FF;

} // namespace bb::avm_trace

namespace bb::avm_trace {

// There are 4 public input columns, 1 for context inputs, and 3 for emitting side effects
using VmPublicInputs = std::tuple<std::array<FF, KERNEL_INPUTS_LENGTH>,   // Input: Kernel context inputs
                                  std::array<FF, KERNEL_OUTPUTS_LENGTH>,  // Output: Kernel outputs data
                                  std::array<FF, KERNEL_OUTPUTS_LENGTH>,  // Output: Kernel outputs side effects
                                  std::array<FF, KERNEL_OUTPUTS_LENGTH>>; // Output: Kernel outputs metadata
// Constants for indexing into the tuple above
static const size_t KERNEL_INPUTS = 0;
static const size_t KERNEL_OUTPUTS_VALUE = 1;
static const size_t KERNEL_OUTPUTS_SIDE_EFFECT_COUNTER = 2;
static const size_t KERNEL_OUTPUTS_METADATA = 3;

// Number of rows
static const size_t AVM_TRACE_SIZE = 1 << 18;
enum class IntermRegister : uint32_t { IA = 0, IB = 1, IC = 2, ID = 3 };
enum class IndirectRegister : uint32_t { IND_A = 0, IND_B = 1, IND_C = 2, IND_D = 3 };

// Keep following enum in sync with MAX_NEM_TAG below
enum class AvmMemoryTag : uint32_t { U0 = 0, U8 = 1, U16 = 2, U32 = 3, U64 = 4, U128 = 5, FF = 6 };
static const uint32_t MAX_MEM_TAG = 6;

static const size_t NUM_MEM_SPACES = 256;
static const uint8_t INTERNAL_CALL_SPACE_ID = 255;
static const uint32_t MAX_SIZE_INTERNAL_STACK = 1 << 16;

struct ContractInstanceHint {
    FF instance_found_in_address;
    FF salt;
    FF deployer_addr;
    FF contract_class_id;
    FF initialisation_hash;
    FF public_key_hash;
};
struct ExecutionHints {
    std::unordered_map<uint32_t, FF> side_effect_hints;

    std::vector<std::vector<FF>> returndata_hints;
    std::map<FF, ContractInstanceHint> contract_instance_hints;
};

} // namespace bb::avm_trace
