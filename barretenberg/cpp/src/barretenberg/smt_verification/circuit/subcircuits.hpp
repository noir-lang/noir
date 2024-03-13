#pragma once
#include "barretenberg/proof_system/circuit_builder/standard_circuit_builder.hpp"

#include "barretenberg/smt_verification/circuit/circuit_schema.hpp"

namespace smt_subcircuits {
using namespace smt_circuit_schema;

/**
 * @brief Circuit stats to identify subcircuit
 *
 * @param circuit Schema of the whole subcircuit
 * @param start_gate Start of the needed subcircuit
 * @param num_gates The number of gates in the needed subcircuit
 * @param idxs Indices of the needed variables to calculate offset
 * @param gate_idxs Indices of the gates that use needed variables
 */
struct CircuitProps {
    CircuitSchema circuit;
    size_t start_gate;
    size_t num_gates;
    std::vector<uint32_t> idxs;
    std::vector<size_t> gate_idxs;
};

CircuitProps get_standard_range_constraint_circuit(size_t n);
CircuitProps get_standard_logic_circuit(size_t n, bool is_xor);
} // namespace smt_subcircuits
