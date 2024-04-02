#pragma once
#include "barretenberg/circuit_checker/standard_circuit_checker.hpp"
#include "barretenberg/circuit_checker/ultra_circuit_checker.hpp"
#include "barretenberg/stdlib_circuit_builders/standard_circuit_builder.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_circuit_builder.hpp"

// TODO(https://github.com/AztecProtocol/barretenberg/issues/928): Reorganize

namespace bb {
template <typename T>
concept IsCheckable = bb::IsAnyOf<T,
                                  StandardCircuitBuilder_<bb::fr>,
                                  StandardCircuitBuilder_<bb::fq>,
                                  UltraCircuitBuilder,
                                  GoblinUltraCircuitBuilder>;

/**
 * @brief The unified interface for check circuit functionality implemented in the specialized CircuitChecker classes
 *
 */
class CircuitChecker {
  public:
    template <typename Builder> static bool check(const Builder& builder)
    {
        static_assert(IsCheckable<Builder>);

        if constexpr (IsUltraBuilder<Builder>) {
            return UltraCircuitChecker::check(builder);
        } else if constexpr (IsStandardBuilder<Builder>) {
            return StandardCircuitChecker::check(builder);
        } else {
            return false;
        }
    }
};

} // namespace bb
