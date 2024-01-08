#pragma once
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include <cstdint>
#include <vector>

namespace acir_format {

struct Blake3Input {
    uint32_t witness;
    uint32_t num_bits;

    // For serialization, update with any new fields
    MSGPACK_FIELDS(witness, num_bits);
    friend bool operator==(Blake3Input const& lhs, Blake3Input const& rhs) = default;
};

struct Blake3Constraint {
    std::vector<Blake3Input> inputs;
    std::vector<uint32_t> result;

    // For serialization, update with any new fields
    MSGPACK_FIELDS(inputs, result);
    friend bool operator==(Blake3Constraint const& lhs, Blake3Constraint const& rhs) = default;
};

template <typename Builder> void create_blake3_constraints(Builder& builder, const Blake3Constraint& constraint);

} // namespace acir_format
