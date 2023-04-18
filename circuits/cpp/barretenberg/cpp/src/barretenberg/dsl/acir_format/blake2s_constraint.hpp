#pragma once
#include <cstdint>
#include <vector>
#include "barretenberg/dsl/types.hpp"

namespace acir_format {

struct Blake2sInput {
    uint32_t witness;
    uint32_t num_bits;

    friend bool operator==(Blake2sInput const& lhs, Blake2sInput const& rhs) = default;
};

struct Blake2sConstraint {
    std::vector<Blake2sInput> inputs;
    std::vector<uint32_t> result;

    friend bool operator==(Blake2sConstraint const& lhs, Blake2sConstraint const& rhs) = default;
};

void create_blake2s_constraints(Composer& composer, const Blake2sConstraint& constraint);

template <typename B> inline void read(B& buf, Blake2sInput& constraint)
{
    using serialize::read;
    read(buf, constraint.witness);
    read(buf, constraint.num_bits);
}

template <typename B> inline void write(B& buf, Blake2sInput const& constraint)
{
    using serialize::write;
    write(buf, constraint.witness);
    write(buf, constraint.num_bits);
}

template <typename B> inline void read(B& buf, Blake2sConstraint& constraint)
{
    using serialize::read;
    read(buf, constraint.inputs);
    read(buf, constraint.result);
}

template <typename B> inline void write(B& buf, Blake2sConstraint const& constraint)
{
    using serialize::write;
    write(buf, constraint.inputs);
    write(buf, constraint.result);
}

} // namespace acir_format
