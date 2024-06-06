#include <iostream>
#include <string>

#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "barretenberg/stdlib_circuit_builders/standard_circuit_builder.hpp"

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
    for (size_t i = 1; i < 255; i++) {
        smt_subcircuits::CircuitProps range_props = smt_subcircuits::get_standard_range_constraint_circuit(i);
        smt_circuit_schema::CircuitSchema circuit = range_props.circuit;

        size_t a_gate = range_props.gate_idxs[0];
        uint32_t a_gate_idx = range_props.idxs[0];
        size_t start_gate = range_props.start_gate;

        ASSERT_EQ(
            "a",
            circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[0][start_gate + a_gate][a_gate_idx]]]);

        size_t num_accs = range_props.gate_idxs.size() - 1;
        for (size_t j = 1; j < num_accs + 1; j++) {
            size_t acc_gate = range_props.gate_idxs[j];
            uint32_t acc_gate_idx = range_props.idxs[j];
            ASSERT_EQ("acc_" + std::to_string(num_accs - j),
                      circuit.vars_of_interest
                          [circuit.real_variable_index[circuit.wires[0][start_gate + acc_gate][acc_gate_idx]]]);
        }
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
            "a",
            circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[0][start_gate + a_gate][a_gate_idx]]]);

        size_t b_gate = logic_props.gate_idxs[1];
        uint32_t b_gate_idx = logic_props.idxs[1];
        ASSERT_EQ(
            "b",
            circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[0][start_gate + b_gate][b_gate_idx]]]);

        size_t c_gate = logic_props.gate_idxs[2];
        uint32_t c_gate_idx = logic_props.idxs[2];
        ASSERT_EQ(
            "c",
            circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[0][start_gate + c_gate][c_gate_idx]]]);
    }
}

// Check that all the relative offsets are calculated correctly.
// I.e. I can find all three operands at the indices, given by get_standard_logic_circuit
TEST(subcircuits, ror_circuit)
{
    for (uint32_t r = 1; r < 8; r += 1) {
        unsigned char n = 8;
        smt_subcircuits::CircuitProps ror_props = smt_subcircuits::get_standard_ror_circuit(n, r);
        smt_circuit_schema::CircuitSchema circuit = ror_props.circuit;

        size_t a_gate = ror_props.gate_idxs[0];
        uint32_t a_gate_idx = ror_props.idxs[0];
        size_t start_gate = ror_props.start_gate;
        ASSERT_EQ(
            "a",
            circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[0][start_gate + a_gate][a_gate_idx]]]);

        size_t b_gate = ror_props.gate_idxs[1];
        uint32_t b_gate_idx = ror_props.idxs[1];
        ASSERT_EQ(
            "b",
            circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[0][start_gate + b_gate][b_gate_idx]]]);
    }
    for (uint32_t r = 1; r < 16; r += 1) {
        uint16_t n = 16;
        smt_subcircuits::CircuitProps ror_props = smt_subcircuits::get_standard_ror_circuit(n, r);
        smt_circuit_schema::CircuitSchema circuit = ror_props.circuit;

        size_t a_gate = ror_props.gate_idxs[0];
        uint32_t a_gate_idx = ror_props.idxs[0];
        size_t start_gate = ror_props.start_gate;
        ASSERT_EQ(
            "a",
            circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[0][start_gate + a_gate][a_gate_idx]]]);

        size_t b_gate = ror_props.gate_idxs[1];
        uint32_t b_gate_idx = ror_props.idxs[1];
        ASSERT_EQ(
            "b",
            circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[0][start_gate + b_gate][b_gate_idx]]]);
    }
    for (uint32_t r = 1; r < 32; r += 1) {
        uint32_t n = 16;
        smt_subcircuits::CircuitProps ror_props = smt_subcircuits::get_standard_ror_circuit(n, r);
        smt_circuit_schema::CircuitSchema circuit = ror_props.circuit;

        size_t a_gate = ror_props.gate_idxs[0];
        uint32_t a_gate_idx = ror_props.idxs[0];
        size_t start_gate = ror_props.start_gate;
        ASSERT_EQ(
            "a",
            circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[0][start_gate + a_gate][a_gate_idx]]]);

        size_t b_gate = ror_props.gate_idxs[1];
        uint32_t b_gate_idx = ror_props.idxs[1];
        ASSERT_EQ(
            "b",
            circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[0][start_gate + b_gate][b_gate_idx]]]);
    }
    for (uint32_t r = 1; r < 64; r += 1) {
        uint64_t n = 16;
        smt_subcircuits::CircuitProps ror_props = smt_subcircuits::get_standard_ror_circuit(n, r);
        smt_circuit_schema::CircuitSchema circuit = ror_props.circuit;

        size_t a_gate = ror_props.gate_idxs[0];
        uint32_t a_gate_idx = ror_props.idxs[0];
        size_t start_gate = ror_props.start_gate;
        ASSERT_EQ(
            "a",
            circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[0][start_gate + a_gate][a_gate_idx]]]);

        size_t b_gate = ror_props.gate_idxs[1];
        uint32_t b_gate_idx = ror_props.idxs[1];
        ASSERT_EQ(
            "b",
            circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[0][start_gate + b_gate][b_gate_idx]]]);
    }
}

// Check that all the relative offsets are calculated correctly.
// I.e. I can find all three operands at the indices, given by get_standard_logic_circuit
TEST(subcircuits, shl_circuit)
{
    for (uint32_t r = 1; r < 8; r += 1) {
        unsigned char n = 8;
        smt_subcircuits::CircuitProps shift_left_props = smt_subcircuits::get_standard_shift_left_circuit(n, r);
        smt_circuit_schema::CircuitSchema circuit = shift_left_props.circuit;

        size_t a_gate = shift_left_props.gate_idxs[0];
        uint32_t a_gate_idx = shift_left_props.idxs[0];
        size_t start_gate = shift_left_props.start_gate;
        ASSERT_EQ(
            "a",
            circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[0][start_gate + a_gate][a_gate_idx]]]);

        size_t b_gate = shift_left_props.gate_idxs[1];
        uint32_t b_gate_idx = shift_left_props.idxs[1];
        ASSERT_EQ(
            "b",
            circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[0][start_gate + b_gate][b_gate_idx]]]);
    }
    for (uint32_t r = 1; r < 16; r += 1) {
        uint16_t n = 16;
        smt_subcircuits::CircuitProps shift_left_props = smt_subcircuits::get_standard_shift_left_circuit(n, r);
        smt_circuit_schema::CircuitSchema circuit = shift_left_props.circuit;

        size_t a_gate = shift_left_props.gate_idxs[0];
        uint32_t a_gate_idx = shift_left_props.idxs[0];
        size_t start_gate = shift_left_props.start_gate;
        ASSERT_EQ(
            "a",
            circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[0][start_gate + a_gate][a_gate_idx]]]);

        size_t b_gate = shift_left_props.gate_idxs[1];
        uint32_t b_gate_idx = shift_left_props.idxs[1];
        ASSERT_EQ(
            "b",
            circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[0][start_gate + b_gate][b_gate_idx]]]);
    }
    for (uint32_t r = 1; r < 32; r += 1) {
        uint32_t n = 16;
        smt_subcircuits::CircuitProps shift_left_props = smt_subcircuits::get_standard_shift_left_circuit(n, r);
        smt_circuit_schema::CircuitSchema circuit = shift_left_props.circuit;

        size_t a_gate = shift_left_props.gate_idxs[0];
        uint32_t a_gate_idx = shift_left_props.idxs[0];
        size_t start_gate = shift_left_props.start_gate;
        ASSERT_EQ(
            "a",
            circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[0][start_gate + a_gate][a_gate_idx]]]);

        size_t b_gate = shift_left_props.gate_idxs[1];
        uint32_t b_gate_idx = shift_left_props.idxs[1];
        ASSERT_EQ(
            "b",
            circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[0][start_gate + b_gate][b_gate_idx]]]);
    }
    for (uint32_t r = 1; r < 64; r += 1) {
        uint64_t n = 16;
        smt_subcircuits::CircuitProps shift_left_props = smt_subcircuits::get_standard_shift_left_circuit(n, r);
        smt_circuit_schema::CircuitSchema circuit = shift_left_props.circuit;

        size_t a_gate = shift_left_props.gate_idxs[0];
        uint32_t a_gate_idx = shift_left_props.idxs[0];
        size_t start_gate = shift_left_props.start_gate;
        ASSERT_EQ(
            "a",
            circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[0][start_gate + a_gate][a_gate_idx]]]);

        size_t b_gate = shift_left_props.gate_idxs[1];
        uint32_t b_gate_idx = shift_left_props.idxs[1];
        ASSERT_EQ(
            "b",
            circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[0][start_gate + b_gate][b_gate_idx]]]);
    }
}

// Check that all the relative offsets are calculated correctly.
// I.e. I can find all three operands at the indices, given by get_standard_logic_circuit
// I can't check the position of the lhs here, since shr doesn't use the witness directly but
// it's accumulators.
// However, according to standard_circuit test they seem fine.
TEST(subcircuits, shr_circuit)
{
    for (uint32_t r = 1; r < 8; r += 2) {
        unsigned char n = 8;
        smt_subcircuits::CircuitProps shift_right_props = smt_subcircuits::get_standard_shift_right_circuit(n, r);
        smt_circuit_schema::CircuitSchema circuit = shift_right_props.circuit;

        size_t start_gate = shift_right_props.start_gate;
        size_t b_gate = shift_right_props.gate_idxs[1];
        uint32_t b_gate_idx = shift_right_props.idxs[1];
        ASSERT_EQ(
            "b",
            circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[0][start_gate + b_gate][b_gate_idx]]]);
    }
    for (uint32_t r = 1; r < 16; r += 2) {
        uint16_t n = 16;
        smt_subcircuits::CircuitProps shift_right_props = smt_subcircuits::get_standard_shift_right_circuit(n, r);
        smt_circuit_schema::CircuitSchema circuit = shift_right_props.circuit;

        size_t start_gate = shift_right_props.start_gate;
        size_t b_gate = shift_right_props.gate_idxs[1];
        uint32_t b_gate_idx = shift_right_props.idxs[1];
        ASSERT_EQ(
            "b",
            circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[0][start_gate + b_gate][b_gate_idx]]]);
    }
    for (uint32_t r = 1; r < 32; r += 2) {
        uint32_t n = 16;
        smt_subcircuits::CircuitProps shift_right_props = smt_subcircuits::get_standard_shift_right_circuit(n, r);
        smt_circuit_schema::CircuitSchema circuit = shift_right_props.circuit;

        size_t start_gate = shift_right_props.start_gate;
        size_t b_gate = shift_right_props.gate_idxs[1];
        uint32_t b_gate_idx = shift_right_props.idxs[1];
        ASSERT_EQ(
            "b",
            circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[0][start_gate + b_gate][b_gate_idx]]]);
    }
    for (uint32_t r = 1; r < 64; r += 2) {
        uint64_t n = 16;
        smt_subcircuits::CircuitProps shift_right_props = smt_subcircuits::get_standard_shift_right_circuit(n, r);
        smt_circuit_schema::CircuitSchema circuit = shift_right_props.circuit;

        size_t start_gate = shift_right_props.start_gate;
        size_t b_gate = shift_right_props.gate_idxs[1];
        uint32_t b_gate_idx = shift_right_props.idxs[1];
        ASSERT_EQ(
            "b",
            circuit.vars_of_interest[circuit.real_variable_index[circuit.wires[0][start_gate + b_gate][b_gate_idx]]]);
    }
}
