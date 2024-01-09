#pragma once
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include <cstdint>

namespace acir_format {

struct EcAdd {
    uint32_t input1_x;
    uint32_t input1_y;
    uint32_t input2_x;
    uint32_t input2_y;
    uint32_t result_x;
    uint32_t result_y;

    // for serialization, update with any new fields
    MSGPACK_FIELDS(input1_x, input1_y, input2_x, input2_y, result_x, result_y);
    friend bool operator==(EcAdd const& lhs, EcAdd const& rhs) = default;
};

template <typename Builder> void create_ec_add_constraint(Builder& builder, const EcAdd& input);

struct EcDouble {
    uint32_t input_x;
    uint32_t input_y;
    uint32_t result_x;
    uint32_t result_y;

    // for serialization, update with any new fields
    MSGPACK_FIELDS(input_x, input_y, result_x, result_y);
    friend bool operator==(EcDouble const& lhs, EcDouble const& rhs) = default;
};

template <typename Builder> void create_ec_double_constraint(Builder& builder, const EcDouble& input);
} // namespace acir_format
