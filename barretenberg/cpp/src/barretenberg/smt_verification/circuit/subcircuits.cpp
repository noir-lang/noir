#include "subcircuits.hpp"

namespace smt_subcircuits {

CircuitProps get_standard_range_constraint_circuit(size_t n)
{
    bb::StandardCircuitBuilder builder = bb::StandardCircuitBuilder();
    uint32_t a_idx = builder.add_variable(bb::fr(0xabbba));
    builder.set_variable_name(a_idx, "a");

    size_t start_gate = builder.get_num_gates();
    auto accumulators = builder.decompose_into_base4_accumulators(a_idx, n);
    size_t num_gates = builder.get_num_gates() - start_gate;

    // give names to accumulators for tests
    size_t num_accs = accumulators.size();
    for (size_t i = 0; i < num_accs - 1; i++) {
        builder.set_variable_name(accumulators[i], "acc_" + std::to_string(i));
    }

    CircuitSchema exported = unpack_from_buffer(builder.export_circuit());

    // relative offsets in the circuit are calculated manually, according to decompose_into_base4_accumulators method
    // lhs position in the gate
    uint32_t lhs_position = 2;
    // number of the gate that contains lhs
    size_t gate_number = num_gates - 1;

    std::vector<uint32_t> idxs = { lhs_position };
    std::vector<size_t> gate_idxs = { gate_number };

    // same thing for accumulators
    for (size_t i = 0; i < num_accs - 1; i++) {
        idxs.push_back(0);
        gate_idxs.push_back(gate_number - 4 * i);
    }

    return { exported, start_gate, num_gates, idxs, gate_idxs };
}

CircuitProps get_standard_logic_circuit(size_t n, bool is_xor)
{
    bb::StandardCircuitBuilder builder = bb::StandardCircuitBuilder();
    uint32_t a_idx = builder.add_variable(bb::fr(0));
    uint32_t b_idx = builder.add_variable(bb::fr(0));
    builder.set_variable_name(a_idx, "a");
    builder.set_variable_name(b_idx, "b");

    size_t start_gate = builder.get_num_gates();
    auto acc = builder.create_logic_constraint(a_idx, b_idx, n, is_xor);
    size_t num_gates = builder.get_num_gates() - start_gate;

    builder.set_variable_name(acc.out.back(), "c");

    CircuitSchema exported = unpack_from_buffer(builder.export_circuit());

    // relative offsets in the circuit are calculated manually, according to create_logic_constraint method
    // lhs, rhs and out positions in the corresponding gates
    uint32_t lhs_position = 2;
    uint32_t rhs_position = 2;
    uint32_t out_position = 2;
    // numbers of the gates that contain lhs, rhs and out
    size_t lhs_gate_number = num_gates - 3;
    size_t rhs_gate_number = num_gates - 2;
    size_t out_gate_number = num_gates - 1;

    return { exported,
             start_gate,
             num_gates,
             { lhs_position, rhs_position, out_position },
             { lhs_gate_number, rhs_gate_number, out_gate_number } };
}
} // namespace smt_subcircuits
