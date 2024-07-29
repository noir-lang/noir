#pragma once
#include "barretenberg/dsl/acir_format/witness_constant.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include <array>
#include <cstdint>
#include <vector>

namespace acir_format {

struct AES128Input {
    uint32_t witness;
    uint32_t num_bits;

    // For serialization, update with any new fields
    MSGPACK_FIELDS(witness, num_bits);
    friend bool operator==(AES128Input const& lhs, AES128Input const& rhs) = default;
};

struct AES128Constraint {
    std::vector<WitnessOrConstant<bb::fr>> inputs;
    std::array<WitnessOrConstant<bb::fr>, 16> iv;
    std::array<WitnessOrConstant<bb::fr>, 16> key;
    std::vector<uint32_t> outputs;

    // For serialization, update with any new fields
    MSGPACK_FIELDS(inputs, iv, key, outputs);
    friend bool operator==(AES128Constraint const& lhs, AES128Constraint const& rhs) = default;
};

template <typename Builder> void create_aes128_constraints(Builder& builder, const AES128Constraint& constraint);

} // namespace acir_format
