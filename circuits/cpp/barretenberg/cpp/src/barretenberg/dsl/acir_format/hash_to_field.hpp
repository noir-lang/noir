#pragma once
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include <cstdint>
#include <vector>

namespace acir_format {

struct HashToFieldInput {
    uint32_t witness;
    uint32_t num_bits;

    // For serialization, update with any new fields
    MSGPACK_FIELDS(witness, num_bits);
    friend bool operator==(HashToFieldInput const& lhs, HashToFieldInput const& rhs) = default;
};

struct HashToFieldConstraint {
    std::vector<HashToFieldInput> inputs;
    uint32_t result;

    // For serialization, update with any new fields
    MSGPACK_FIELDS(inputs, result);
    friend bool operator==(HashToFieldConstraint const& lhs, HashToFieldConstraint const& rhs) = default;
};

void create_hash_to_field_constraints(Builder& builder, HashToFieldConstraint constraint);

} // namespace acir_format
