#include "subcircuits.hpp"

namespace smt_subcircuits {

CircuitSchema get_standard_range_constraint_circuit(size_t n)
{
    bb::StandardCircuitBuilder builder = bb::StandardCircuitBuilder();
    uint32_t a_idx = builder.add_variable(bb::fr::random_element());
    builder.set_variable_name(a_idx, "a");
    builder.create_range_constraint(a_idx, n);
    return unpack_from_buffer(builder.export_circuit());
}

CircuitSchema get_standard_logic_circuit(size_t n, bool is_xor)
{
    bb::StandardCircuitBuilder builder = bb::StandardCircuitBuilder();
    uint32_t a_idx = builder.add_variable(bb::fr::random_element());
    uint32_t b_idx = builder.add_variable(bb::fr::random_element());
    builder.set_variable_name(a_idx, "a");
    builder.set_variable_name(b_idx, "b");
    builder.create_logic_constraint(a_idx, b_idx, n, is_xor);
    return unpack_from_buffer(builder.export_circuit());
}
} // namespace smt_subcircuits