#pragma once
#include "barretenberg/proof_system/circuit_builder/standard_circuit_builder.hpp"

#include <optional>

namespace bb {

class StandardCircuitChecker {
  public:
    using FF = bb::fr;

    /**
     * @brief Specialized circuit checker for the Standard builder
     *
     * @tparam FF Allows for use with scalar field for bn254 or grumpkin
     * @param builder
     */
    template <typename FF> static bool check(const StandardCircuitBuilder_<FF>& builder)
    {
        const auto& block = builder.blocks.arithmetic;
        for (size_t i = 0; i < builder.num_gates; i++) {
            FF left = builder.get_variable(block.w_l()[i]);
            FF right = builder.get_variable(block.w_r()[i]);
            FF output = builder.get_variable(block.w_o()[i]);
            FF gate_sum = block.q_m()[i] * left * right + block.q_1()[i] * left + block.q_2()[i] * right +
                          block.q_3()[i] * output + block.q_c()[i];
            if (!gate_sum.is_zero()) {
                info("gate number", i);
                return false;
            }
        }
        return true;
    }
};
} // namespace bb
