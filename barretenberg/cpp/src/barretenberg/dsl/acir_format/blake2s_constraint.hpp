#pragma once
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include <cstdint>
#include <vector>

namespace acir_format {

struct Blake2sInput {
    uint32_t witness;
    uint32_t num_bits;

    // For serialization, update with any new fields
    MSGPACK_FIELDS(witness, num_bits);
    friend bool operator==(Blake2sInput const& lhs, Blake2sInput const& rhs) = default;
};

struct Blake2sConstraint {
    std::vector<Blake2sInput> inputs;
    std::array<uint32_t, 32> result;

    // For serialization, update with any new fields
    MSGPACK_FIELDS(inputs, result);
    friend bool operator==(Blake2sConstraint const& lhs, Blake2sConstraint const& rhs) = default;
};

template <typename Builder> void create_blake2s_constraints(Builder& builder, const Blake2sConstraint& constraint);

} // namespace acir_format
