#pragma once
#include "barretenberg/plonk/proof_system/proving_key/proving_key.hpp"
#include "barretenberg/proof_system/flavor/flavor.hpp"
#include "barretenberg/proof_system/circuit_constructors/standard_circuit_constructor.hpp"
#include "barretenberg/proof_system/circuit_constructors/turbo_circuit_constructor.hpp"
#include "barretenberg/proof_system/circuit_constructors/ultra_circuit_constructor.hpp"

namespace proof_system::plonk::flavor {
class Standard {
  public:
    using CircuitConstructor = proof_system::StandardCircuitConstructor;
    using ProvingKey = plonk::proving_key;
    static constexpr size_t NUM_WIRES = CircuitConstructor::NUM_WIRES;
};

class Turbo {
  public:
    using CircuitConstructor = proof_system::TurboCircuitConstructor;
    using ProvingKey = plonk::proving_key;
    static constexpr size_t NUM_WIRES = CircuitConstructor::NUM_WIRES;
};

class Ultra {
  public:
    using CircuitConstructor = proof_system::UltraCircuitConstructor;
    using ProvingKey = plonk::proving_key;
    static constexpr size_t NUM_WIRES = CircuitConstructor::NUM_WIRES;
};
} // namespace proof_system::plonk::flavor