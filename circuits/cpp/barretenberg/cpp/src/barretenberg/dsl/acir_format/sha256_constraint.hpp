#pragma once
#include <cstdint>
#include <vector>
#include "barretenberg/dsl/types.hpp"

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
void create_sha256_constraints(Composer& composer, const Sha256Constraint& constraint);

template <typename B> inline void read(B& buf, Sha256Input& constraint)
{
    using serialize::read;
    read(buf, constraint.witness);
    read(buf, constraint.num_bits);
}

template <typename B> inline void write(B& buf, Sha256Input const& constraint)
{
    using serialize::write;
    write(buf, constraint.witness);
    write(buf, constraint.num_bits);
}

template <typename B> inline void read(B& buf, Sha256Constraint& constraint)
{
    using serialize::read;
    read(buf, constraint.inputs);
    read(buf, constraint.result);
}

template <typename B> inline void write(B& buf, Sha256Constraint const& constraint)
{
    using serialize::write;
    write(buf, constraint.inputs);
    write(buf, constraint.result);
}

} // namespace acir_format
