#pragma once
#include "barretenberg/serialize/msgpack.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "serde/index.hpp"
#include "witness_constant.hpp"
#include <cstdint>
#include <vector>

namespace acir_format {

struct MultiScalarMul {
    std::vector<WitnessOrConstant<bb::fr>> points;
    std::vector<WitnessOrConstant<bb::fr>> scalars;

    uint32_t out_point_x;
    uint32_t out_point_y;
    uint32_t out_point_is_infinite;

    // for serialization, update with any new fields
    MSGPACK_FIELDS(points, scalars, out_point_x, out_point_y, out_point_is_infinite);
    friend bool operator==(MultiScalarMul const& lhs, MultiScalarMul const& rhs) = default;
};

template <typename Builder> void create_multi_scalar_mul_constraint(Builder& builder, const MultiScalarMul& input);

} // namespace acir_format
