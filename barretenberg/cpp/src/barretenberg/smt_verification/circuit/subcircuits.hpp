#pragma once
#include "barretenberg/proof_system/circuit_builder/standard_circuit_builder.hpp"

#include "barretenberg/smt_verification/circuit/circuit_schema.hpp"

namespace smt_subcircuits {
using namespace smt_circuit_schema;

CircuitSchema get_standard_range_constraint_circuit(size_t n);
CircuitSchema get_standard_logic_circuit(size_t n, bool is_xor);
} // namespace smt_subcircuits