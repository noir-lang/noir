#include "logic_constraint.hpp"
#include "barretenberg/stdlib/primitives/logic/logic.hpp"

namespace acir_format {

using namespace bb::plonk;

template <typename Builder>
void create_logic_gate(Builder& builder,
                       const WitnessOrConstant<bb::fr> a,
                       const WitnessOrConstant<bb::fr> b,
                       const uint32_t result,
                       const size_t num_bits,
                       const bool is_xor_gate)
{
    using field_ct = bb::stdlib::field_t<Builder>;

    field_ct left = to_field_ct(a, builder);
    field_ct right = to_field_ct(b, builder);

    field_ct res = bb::stdlib::logic<Builder>::create_logic_constraint(left, right, num_bits, is_xor_gate);
    field_ct our_res = field_ct::from_witness_index(&builder, result);
    res.assert_equal(our_res);
}

template void create_logic_gate<bb::MegaCircuitBuilder>(bb::MegaCircuitBuilder& builder,
                                                        const WitnessOrConstant<bb::fr> a,
                                                        const WitnessOrConstant<bb::fr> b,
                                                        const uint32_t result,
                                                        const size_t num_bits,
                                                        const bool is_xor_gate);
template void create_logic_gate<bb::UltraCircuitBuilder>(bb::UltraCircuitBuilder& builder,
                                                         const WitnessOrConstant<bb::fr> a,
                                                         const WitnessOrConstant<bb::fr> b,
                                                         const uint32_t result,
                                                         const size_t num_bits,
                                                         const bool is_xor_gate);

} // namespace acir_format
