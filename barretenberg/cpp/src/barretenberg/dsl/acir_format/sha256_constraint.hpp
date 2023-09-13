#pragma once
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include <cstdint>
#include <vector>

namespace acir_format {

struct Sha256Input {
    uint32_t witness;
    uint32_t num_bits;

    friend bool operator==(Sha256Input const& lhs, Sha256Input const& rhs) = default;
    // for serialization, update with any new fields
    MSGPACK_FIELDS(witness, num_bits);
};

struct Sha256Constraint {
    std::vector<Sha256Input> inputs;
    std::vector<uint32_t> result;

    friend bool operator==(Sha256Constraint const& lhs, Sha256Constraint const& rhs) = default;
    // for serialization, update with any new fields
    MSGPACK_FIELDS(inputs, result);
};

// This function does not work (properly) because the stdlib:sha256 function is not working correctly for 512 bits
// pair<witness_index, bits>
void create_sha256_constraints(Builder& builder, const Sha256Constraint& constraint);
} // namespace acir_format
