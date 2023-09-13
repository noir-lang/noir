#pragma once
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include <cstdint>
#include <vector>

namespace acir_format {

struct HashInput {
    uint32_t witness;
    uint32_t num_bits;

    // For serialization, update with any new fields
    MSGPACK_FIELDS(witness, num_bits);
    friend bool operator==(HashInput const& lhs, HashInput const& rhs) = default;
};

struct KeccakConstraint {
    std::vector<HashInput> inputs;
    std::vector<uint32_t> result;

    // For serialization, update with any new fields
    MSGPACK_FIELDS(inputs, result);
    friend bool operator==(KeccakConstraint const& lhs, KeccakConstraint const& rhs) = default;
};

struct KeccakVarConstraint {
    std::vector<HashInput> inputs;
    std::vector<uint32_t> result;
    uint32_t var_message_size;

    // For serialization, update with any new fields
    MSGPACK_FIELDS(inputs, result, var_message_size);
    friend bool operator==(KeccakVarConstraint const& lhs, KeccakVarConstraint const& rhs) = default;
};

void create_keccak_constraints(Builder& builder, const KeccakConstraint& constraint);
void create_keccak_var_constraints(Builder& builder, const KeccakVarConstraint& constraint);

} // namespace acir_format
