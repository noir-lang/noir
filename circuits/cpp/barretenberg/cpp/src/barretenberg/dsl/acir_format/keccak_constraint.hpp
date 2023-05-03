#pragma once
#include <cstdint>
#include <vector>
#include "barretenberg/dsl/types.hpp"

namespace acir_format {

struct HashInput {
    uint32_t witness;
    uint32_t num_bits;

    friend bool operator==(HashInput const& lhs, HashInput const& rhs) = default;
};

struct KeccakConstraint {
    std::vector<HashInput> inputs;
    std::vector<uint32_t> result;

    friend bool operator==(KeccakConstraint const& lhs, KeccakConstraint const& rhs) = default;
};

void create_keccak_constraints(Composer& composer, const KeccakConstraint& constraint);

template <typename B> inline void read(B& buf, HashInput& constraint)
{
    using serialize::read;
    read(buf, constraint.witness);
    read(buf, constraint.num_bits);
}

template <typename B> inline void write(B& buf, HashInput const& constraint)
{
    using serialize::write;
    write(buf, constraint.witness);
    write(buf, constraint.num_bits);
}

template <typename B> inline void read(B& buf, KeccakConstraint& constraint)
{
    using serialize::read;
    read(buf, constraint.inputs);
    read(buf, constraint.result);
}

template <typename B> inline void write(B& buf, KeccakConstraint const& constraint)
{
    using serialize::write;
    write(buf, constraint.inputs);
    write(buf, constraint.result);
}

} // namespace acir_format
