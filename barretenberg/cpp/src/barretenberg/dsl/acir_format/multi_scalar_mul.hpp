#pragma once
#include "barretenberg/serialize/msgpack.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "serde/index.hpp"
#include <cstdint>
#include <vector>

namespace acir_format {

template <typename FF> struct WitnessConstant {
    uint32_t index;
    FF value;
    bool is_constant;
    MSGPACK_FIELDS(index, value, is_constant);
    friend bool operator==(WitnessConstant const& lhs, WitnessConstant const& rhs) = default;
};

struct MultiScalarMul {
    std::vector<WitnessConstant<bb::fr>> points;
    std::vector<WitnessConstant<bb::fr>> scalars;

    uint32_t out_point_x;
    uint32_t out_point_y;
    uint32_t out_point_is_infinite;

    // for serialization, update with any new fields
    MSGPACK_FIELDS(points, scalars, out_point_x, out_point_y, out_point_is_infinite);
    friend bool operator==(MultiScalarMul const& lhs, MultiScalarMul const& rhs) = default;
};

template <typename Builder> void create_multi_scalar_mul_constraint(Builder& builder, const MultiScalarMul& input);

} // namespace acir_format
