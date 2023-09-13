#pragma once
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include <cstdint>

namespace acir_format {

struct FixedBaseScalarMul {
    uint32_t low;
    uint32_t high;
    uint32_t pub_key_x;
    uint32_t pub_key_y;

    // for serialization, update with any new fields
    MSGPACK_FIELDS(low, high, pub_key_x, pub_key_y);
    friend bool operator==(FixedBaseScalarMul const& lhs, FixedBaseScalarMul const& rhs) = default;
};

void create_fixed_base_constraint(Builder& builder, const FixedBaseScalarMul& input);

} // namespace acir_format
