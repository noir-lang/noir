#pragma once
#include <cstdint>
#include <vector>
#include "barretenberg/dsl/types.hpp"

namespace acir_format {

struct HashToFieldInput {
    uint32_t witness;
    uint32_t num_bits;

    friend bool operator==(HashToFieldInput const& lhs, HashToFieldInput const& rhs) = default;
};

struct HashToFieldConstraint {
    std::vector<HashToFieldInput> inputs;
    uint32_t result;

    friend bool operator==(HashToFieldConstraint const& lhs, HashToFieldConstraint const& rhs) = default;
};

void create_hash_to_field_constraints(Composer& composer, HashToFieldConstraint constraint);

template <typename B> inline void read(B& buf, HashToFieldInput& constraint)
{
    using serialize::read;
    read(buf, constraint.witness);
    read(buf, constraint.num_bits);
}

template <typename B> inline void write(B& buf, HashToFieldInput const& constraint)
{
    using serialize::write;
    write(buf, constraint.witness);
    write(buf, constraint.num_bits);
}

template <typename B> inline void read(B& buf, HashToFieldConstraint& constraint)
{
    using serialize::read;
    read(buf, constraint.inputs);
    read(buf, constraint.result);
}

template <typename B> inline void write(B& buf, HashToFieldConstraint const& constraint)
{
    using serialize::write;
    write(buf, constraint.inputs);
    write(buf, constraint.result);
}

} // namespace acir_format
