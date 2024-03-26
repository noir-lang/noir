#pragma once

#include "barretenberg/relations/generated/avm/avm_binary.hpp"
#include "barretenberg/stdlib_circuit_builders/circuit_builder_base.hpp"
#include "barretenberg/vm/generated/avm_circuit_builder.hpp"
#include <cstdint>

namespace bb::avm_trace {

using Flavor = bb::AvmFlavor;
using FF = Flavor::FF;
using Row = bb::AvmFullRow<bb::fr>;

// Number of rows
static const size_t AVM_TRACE_SIZE = 1 << 18;
enum class IntermRegister : uint32_t { IA = 0, IB = 1, IC = 2 };
enum class IndirectRegister : uint32_t { IND_A = 0, IND_B = 1, IND_C = 2 };

// Keep following enum in sync with MAX_NEM_TAG below
enum class AvmMemoryTag : uint32_t { U0 = 0, U8 = 1, U16 = 2, U32 = 3, U64 = 4, U128 = 5, FF = 6 };
static const uint32_t MAX_MEM_TAG = 6;

} // namespace bb::avm_trace
