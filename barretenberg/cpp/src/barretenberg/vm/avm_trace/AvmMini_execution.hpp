#pragma once

#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/vm/avm_trace/AvmMini_common.hpp"
#include "barretenberg/vm/avm_trace/AvmMini_instructions.hpp"
#include "barretenberg/vm/avm_trace/AvmMini_trace.hpp"
#include <cstddef>
#include <cstdint>
#include <vector>

namespace avm_trace {

class Execution {
  public:
    Execution() = default;

    static size_t const AVM_OPERAND_BYTE_LENGTH = 4; // Keep in sync with TS code
    static_assert(sizeof(uint32_t) / sizeof(uint8_t) == AVM_OPERAND_BYTE_LENGTH);

    static size_t const AVM_OPCODE_BYTE_LENGTH = 1; // Keep in sync with TS code
    static size_t const AVM_IN_TAG_BYTE_LENGTH = 1; // Keep in sync with TS code

    static std::vector<Instruction> parse(std::vector<uint8_t> const& bytecode);
    static std::vector<Row> gen_trace(std::vector<Instruction> const& instructions, std::vector<FF> const& calldata);
    static plonk::proof run_and_prove(std::vector<uint8_t> const& bytecode, std::vector<FF> const& calldata);
};

} // namespace avm_trace