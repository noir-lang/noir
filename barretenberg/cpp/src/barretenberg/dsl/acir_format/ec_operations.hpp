#pragma once
#include "barretenberg/dsl/acir_format/witness_constant.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include <cstdint>

namespace acir_format {

struct EcAdd {
    WitnessOrConstant<bb::fr> input1_x;
    WitnessOrConstant<bb::fr> input1_y;
    WitnessOrConstant<bb::fr> input1_infinite;
    WitnessOrConstant<bb::fr> input2_x;
    WitnessOrConstant<bb::fr> input2_y;
    WitnessOrConstant<bb::fr> input2_infinite;
    uint32_t result_x;
    uint32_t result_y;
    uint32_t result_infinite;

    // for serialization, update with any new fields
    MSGPACK_FIELDS(
        input1_x, input1_y, input1_infinite, input2_x, input2_y, input2_infinite, result_x, result_y, result_infinite);
    friend bool operator==(EcAdd const& lhs, EcAdd const& rhs) = default;
};

template <typename Builder>
void create_ec_add_constraint(Builder& builder, const EcAdd& input, bool has_valid_witness_assignments);
} // namespace acir_format
