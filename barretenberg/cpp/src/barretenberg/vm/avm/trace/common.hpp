#pragma once

#include "barretenberg/common/throw_or_abort.hpp"
#include "barretenberg/vm/constants.hpp"

#include "barretenberg/vm/avm/generated/flavor_settings.hpp"
#include "barretenberg/vm/avm/generated/full_row.hpp"

#include <array>
#include <cstdint>

namespace bb::avm_trace {

using FF = AvmFlavorSettings::FF;

// To toggle all relevant unit tests with proving, set the env variable "AVM_ENABLE_FULL_PROVING".
static const bool ENABLE_PROVING = std::getenv("AVM_ENABLE_FULL_PROVING") != nullptr;

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

constexpr size_t L2_HI_GAS_COUNTS_IDX = 0;
constexpr size_t L2_LO_GAS_COUNTS_IDX = 1;
constexpr size_t DA_HI_GAS_COUNTS_IDX = 2;
constexpr size_t DA_LO_GAS_COUNTS_IDX = 3;

enum class IntermRegister : uint32_t { IA = 0, IB = 1, IC = 2, ID = 3 };
enum class IndirectRegister : uint32_t { IND_A = 0, IND_B = 1, IND_C = 2, IND_D = 3 };

// Keep following enum in sync with MAX_MEM_TAG below
enum class AvmMemoryTag : uint32_t { U0 = 0, U8 = 1, U16 = 2, U32 = 3, U64 = 4, U128 = 5, FF = 6 };
static const uint32_t MAX_MEM_TAG = 6;

static const size_t NUM_MEM_SPACES = 256;
static const uint8_t INTERNAL_CALL_SPACE_ID = 255;
static const uint32_t MAX_SIZE_INTERNAL_STACK = 1 << 16;

} // namespace bb::avm_trace
