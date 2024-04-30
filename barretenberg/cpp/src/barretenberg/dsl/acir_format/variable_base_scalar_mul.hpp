#pragma once
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include <cstdint>

namespace acir_format {

struct VariableBaseScalarMul {
    uint32_t point_x;
    uint32_t point_y;
    uint32_t scalar_low;
    uint32_t scalar_high;
    uint32_t out_point_x;
    uint32_t out_point_y;

    // for serialization, update with any new fields
    MSGPACK_FIELDS(point_x, point_y, scalar_low, scalar_high, out_point_x, out_point_y);
    friend bool operator==(VariableBaseScalarMul const& lhs, VariableBaseScalarMul const& rhs) = default;
};

template <typename Builder> void create_variable_base_constraint(Builder& builder, const VariableBaseScalarMul& input);

} // namespace acir_format
