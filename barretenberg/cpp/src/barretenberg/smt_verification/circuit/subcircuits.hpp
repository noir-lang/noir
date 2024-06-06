#pragma once
#include "barretenberg/stdlib/primitives/uint/uint.hpp"
#include "barretenberg/stdlib_circuit_builders/standard_circuit_builder.hpp"

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

template <typename T> CircuitProps get_standard_ror_circuit(T n, size_t sh)
{
    using witness_ct = bb::stdlib::witness_t<bb::StandardCircuitBuilder>;
    bb::StandardCircuitBuilder builder = bb::StandardCircuitBuilder();
    bb::stdlib::uint<bb::StandardCircuitBuilder, T> a = witness_ct(&builder, n);

    size_t start_gate = builder.get_num_gates();
    auto b = a.ror(sh);
    builder.set_variable_name(a.get_witness_index(), "a");
    builder.set_variable_name(b.get_witness_index(), "b");

    size_t num_gates = builder.get_num_gates() - start_gate;
    CircuitSchema exported = unpack_from_buffer(builder.export_circuit());

    // relative offsets in the circuit are calculated manually, according to decompose_into_base4_accumulators method
    // lhs, rhs positions in the gate
    uint32_t a_position = (sh & 1) == 1 ? 1 : 0;
    uint32_t b_position = (sh & 1) == 1 ? 0 : 2;
    // number of the gate that contains lhs, rhs
    size_t a_gate_number = (sh & 1) == 1 ? num_gates - 2 : num_gates - 1;
    size_t b_gate_number = (sh & 1) == 1 ? num_gates - 2 : num_gates - 1;

    return { exported, start_gate, num_gates, { a_position, b_position }, { a_gate_number, b_gate_number } };
}

template <typename T> CircuitProps get_standard_shift_right_circuit(T n, uint32_t sh)
{
    // sh have to be even, otherwise the resulting circuit will be empty
    if ((sh & 1) == 0) {
        throw std::invalid_argument("sh value should be odd");
    }
    using witness_ct = bb::stdlib::witness_t<bb::StandardCircuitBuilder>;
    bb::StandardCircuitBuilder builder = bb::StandardCircuitBuilder();
    bb::stdlib::uint<bb::StandardCircuitBuilder, T> a = witness_ct(&builder, n);
    size_t start_gate = builder.get_num_gates();
    auto b = a >> sh;
    builder.set_variable_name(a.get_witness_index(), "a");
    builder.set_variable_name(b.get_witness_index(), "b");

    size_t num_gates = builder.get_num_gates() - start_gate;
    CircuitSchema exported = unpack_from_buffer(builder.export_circuit());

    // relative offsets in the circuit are calculated manually, according to decompose_into_base4_accumulators method
    // lhs, rhs positions in the gate
    uint32_t acc_position = 0;
    uint32_t b_position = 1;
    // number of the gate that contains lhs, rhs
    size_t acc_gate_number = num_gates - 1;
    size_t b_gate_number = num_gates - 2;

    return { exported, start_gate, num_gates, { acc_position, b_position }, { acc_gate_number, b_gate_number } };
}

template <typename T> CircuitProps get_standard_shift_left_circuit(T n, uint32_t sh)
{
    using witness_ct = bb::stdlib::witness_t<bb::StandardCircuitBuilder>;
    bb::StandardCircuitBuilder builder = bb::StandardCircuitBuilder();
    bb::stdlib::uint<bb::StandardCircuitBuilder, T> a = witness_ct(&builder, n);
    size_t start_gate = builder.get_num_gates();
    auto b = a << sh;
    builder.set_variable_name(a.get_witness_index(), "a");
    builder.set_variable_name(b.get_witness_index(), "b");

    size_t num_gates = builder.get_num_gates() - start_gate;
    CircuitSchema exported = unpack_from_buffer(builder.export_circuit());

    // relative offsets in the circuit are calculated manually, according to decompose_into_base4_accumulators method
    // lhs, rhs positions in the gate
    uint32_t a_position = (sh & 1) == 1 ? 1 : 0;
    uint32_t b_position = (sh & 1) == 1 ? 0 : 2;
    // number of the gate that contains lhs, rhs
    size_t a_gate_number = (sh & 1) == 1 ? num_gates - 2 : num_gates - 1;
    size_t b_gate_number = (sh & 1) == 1 ? num_gates - 2 : num_gates - 1;

    return { exported, start_gate, num_gates, { a_position, b_position }, { a_gate_number, b_gate_number } };
}
} // namespace smt_subcircuits
