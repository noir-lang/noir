#pragma once
#include "eccvm_circuit_builder.hpp"

namespace bb {
class ECCVMTraceChecker {
  public:
    static bool check(ECCVMCircuitBuilder&);
};
} // namespace bb