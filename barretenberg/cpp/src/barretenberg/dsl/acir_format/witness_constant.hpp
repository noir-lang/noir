#pragma once
#include "barretenberg/serialize/msgpack.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"

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

} // namespace acir_format