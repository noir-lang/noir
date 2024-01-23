#pragma once

#include "barretenberg/proof_system/circuit_builder/circuit_builder_base.hpp"
#include "barretenberg/proof_system/circuit_builder/generated/AvmMini_circuit_builder.hpp"

using Flavor = bb::honk::flavor::AvmMiniFlavor;
using FF = Flavor::FF;
using Row = bb::AvmMiniFullRow<bb::fr>;

namespace avm_trace {

// Number of rows
static const size_t AVM_TRACE_SIZE = 256;
enum class IntermRegister : uint32_t { IA = 0, IB = 1, IC = 2 };
enum class AvmMemoryTag : uint32_t { U0 = 0, U8 = 1, U16 = 2, U32 = 3, U64 = 4, U128 = 5, FF = 6 };

} // namespace avm_trace