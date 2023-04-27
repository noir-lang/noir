#pragma once
#include <vector>
#include "barretenberg/dsl/types.hpp"

namespace acir_format {

struct ComputeMerkleRootConstraint {
    std::vector<uint32_t> hash_path; // Vector of pairs of hashpaths. eg indices 0,1 denotes the pair (0,1)
    uint32_t leaf;                   // Single field element -- field_t
    uint32_t result;                 // Single field element -- field_t
    uint32_t index;

    friend bool operator==(ComputeMerkleRootConstraint const& lhs, ComputeMerkleRootConstraint const& rhs) = default;
};

void create_compute_merkle_root_constraint(Composer& composer, const ComputeMerkleRootConstraint& input);

template <typename B> inline void read(B& buf, ComputeMerkleRootConstraint& constraint)
{
    using serialize::read;
    read(buf, constraint.hash_path);
    read(buf, constraint.leaf);
    read(buf, constraint.result);
    read(buf, constraint.index);
}

template <typename B> inline void write(B& buf, ComputeMerkleRootConstraint const& constraint)
{
    using serialize::write;
    write(buf, constraint.hash_path);
    write(buf, constraint.leaf);
    write(buf, constraint.result);
    write(buf, constraint.index);
}

} // namespace acir_format
