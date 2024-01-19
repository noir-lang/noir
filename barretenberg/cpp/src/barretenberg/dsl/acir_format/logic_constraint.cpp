#include "logic_constraint.hpp"
#include "barretenberg/stdlib/primitives/logic/logic.hpp"

namespace acir_format {

using namespace bb::plonk;

template <typename Builder>
void create_logic_gate(Builder& builder,
                       const uint32_t a,
                       const uint32_t b,
                       const uint32_t result,
                       const size_t num_bits,
                       const bool is_xor_gate)
{
    using field_ct = bb::plonk::stdlib::field_t<Builder>;

    field_ct left = field_ct::from_witness_index(&builder, a);
    field_ct right = field_ct::from_witness_index(&builder, b);

    field_ct res = stdlib::logic<Builder>::create_logic_constraint(left, right, num_bits, is_xor_gate);
    field_ct our_res = field_ct::from_witness_index(&builder, result);
    res.assert_equal(our_res);
}

template void create_logic_gate<GoblinUltraCircuitBuilder>(GoblinUltraCircuitBuilder& builder,
                                                           const uint32_t a,
                                                           const uint32_t b,
                                                           const uint32_t result,
                                                           const size_t num_bits,
                                                           const bool is_xor_gate);
template void create_logic_gate<UltraCircuitBuilder>(UltraCircuitBuilder& builder,
                                                     const uint32_t a,
                                                     const uint32_t b,
                                                     const uint32_t result,
                                                     const size_t num_bits,
                                                     const bool is_xor_gate);

} // namespace acir_format
