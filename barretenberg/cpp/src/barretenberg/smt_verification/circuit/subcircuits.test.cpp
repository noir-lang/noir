#include <iostream>
#include <string>

#include "barretenberg/proof_system/circuit_builder/standard_circuit_builder.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"

#include "barretenberg/smt_verification/circuit/subcircuits.hpp"

#include <gtest/gtest.h>

using namespace bb;

namespace {
auto& engine = numeric::get_debug_randomness();
}

// Check that all the relative offsets are calculated correctly.
// I.e. I can find an operand at the index, given by get_standard_range_constraint_circuit
TEST(subcircuits, range_circuit)
{
    for (size_t i = 1; i < 256; i++) {
        smt_subcircuits::CircuitProps range_props = smt_subcircuits::get_standard_range_constraint_circuit(i);
        smt_circuit_schema::CircuitSchema circuit = range_props.circuit;

        size_t a_gate = range_props.gate_idxs[0];
        uint32_t a_gate_idx = range_props.idxs[0];
        size_t start_gate = range_props.start_gate;

        ASSERT_EQ(
            "a", circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[start_gate + a_gate][a_gate_idx]]]);
    }
}
// Check that all the relative offsets are calculated correctly.
// I.e. I can find all three operands at the indices, given by get_standard_logic_circuit
TEST(subcircuits, logic_circuit)
{
    for (size_t i = 2; i < 256; i += 2) {
        smt_subcircuits::CircuitProps logic_props = smt_subcircuits::get_standard_logic_circuit(i, true);
        smt_circuit_schema::CircuitSchema circuit = logic_props.circuit;

        size_t a_gate = logic_props.gate_idxs[0];
        uint32_t a_gate_idx = logic_props.idxs[0];
        size_t start_gate = logic_props.start_gate;
        ASSERT_EQ(
            "a", circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[start_gate + a_gate][a_gate_idx]]]);

        size_t b_gate = logic_props.gate_idxs[1];
        uint32_t b_gate_idx = logic_props.idxs[1];
        ASSERT_EQ(
            "b", circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[start_gate + b_gate][b_gate_idx]]]);

        size_t c_gate = logic_props.gate_idxs[2];
        uint32_t c_gate_idx = logic_props.idxs[2];
        ASSERT_EQ(
            "c", circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[start_gate + c_gate][c_gate_idx]]]);
    }
}