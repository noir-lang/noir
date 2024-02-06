#pragma once

#include "barretenberg/honk/proof_system/types/proof.hpp"
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

    static std::vector<Row> gen_trace(std::vector<Instruction> const& instructions,
                                      std::vector<FF> const& calldata = {});
    static bb::HonkProof run_and_prove(std::vector<uint8_t> const& bytecode, std::vector<FF> const& calldata = {});
};

} // namespace avm_trace