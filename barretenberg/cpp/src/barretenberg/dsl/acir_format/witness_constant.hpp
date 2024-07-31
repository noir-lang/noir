#pragma once
#include "barretenberg/dsl/acir_format/ecdsa_secp256k1.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "barretenberg/stdlib/primitives/group/cycle_group.hpp"

namespace acir_format {
template <typename FF> struct WitnessOrConstant {

    uint32_t index;
    FF value;
    bool is_constant;
    MSGPACK_FIELDS(index, value, is_constant);
    friend bool operator==(WitnessOrConstant const& lhs, WitnessOrConstant const& rhs) = default;
    static WitnessOrConstant from_index(uint32_t index)
    {
        return WitnessOrConstant{
            .index = index,
            .value = FF::zero(),
            .is_constant = false,
        };
    }
};

template <typename Builder, typename FF>
bb::stdlib::field_t<Builder> to_field_ct(const WitnessOrConstant<FF>& input, Builder& builder)
{
    using field_ct = bb::stdlib::field_t<Builder>;
    if (input.is_constant) {
        return field_ct(input.value);
    }
    return field_ct::from_witness_index(&builder, input.index);
}

template <typename Builder, typename FF>
bb::stdlib::cycle_group<Builder> to_grumpkin_point(const WitnessOrConstant<FF>& input_x,
                                                   const WitnessOrConstant<FF>& input_y,
                                                   const WitnessOrConstant<FF>& input_infinite,
                                                   bool has_valid_witness_assignments,
                                                   Builder& builder);

} // namespace acir_format