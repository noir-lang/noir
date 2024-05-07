#pragma once
#include "eccvm_circuit_builder.hpp"

namespace bb {
class ECCVMTraceChecker {
  public:
    static bool check(ECCVMCircuitBuilder&, numeric::RNG* engine_ptr = nullptr);
};
} // namespace bb