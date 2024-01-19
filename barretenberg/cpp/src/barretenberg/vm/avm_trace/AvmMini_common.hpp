#pragma once

#include "barretenberg/proof_system/circuit_builder/circuit_builder_base.hpp"
#include "barretenberg/proof_system/circuit_builder/generated/AvmMini_circuit_builder.hpp"

using Flavor = bb::honk::flavor::AvmMiniFlavor;
using FF = Flavor::FF;
using Row = bb::AvmMiniFullRow<bb::fr>;

namespace avm_trace {

// Number of rows
static const size_t AVM_TRACE_SIZE = 256;
enum class IntermRegister : uint32_t { ia = 0, ib = 1, ic = 2 };
enum class AvmMemoryTag : uint32_t { u0 = 0, u8 = 1, u16 = 2, u32 = 3, u64 = 4, u128 = 5, ff = 6 };

} // namespace avm_trace