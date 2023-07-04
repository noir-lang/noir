#pragma once
#include "barretenberg/dsl/types.hpp"
#include <vector>

namespace acir_format {

// P = xG + bH
struct PedersenConstraint {
    std::vector<uint32_t> scalars;
    uint32_t hash_index;

    uint32_t result_x;
    uint32_t result_y;

    friend bool operator==(PedersenConstraint const& lhs, PedersenConstraint const& rhs) = default;
};

void create_pedersen_constraint(Builder& builder, const PedersenConstraint& input);

template <typename B> inline void read(B& buf, PedersenConstraint& constraint)
{
    using serialize::read;
    read(buf, constraint.scalars);
    read(buf, constraint.hash_index);
    read(buf, constraint.result_x);
    read(buf, constraint.result_y);
}

template <typename B> inline void write(B& buf, PedersenConstraint const& constraint)
{
    using serialize::write;
    write(buf, constraint.scalars);
    write(buf, constraint.hash_index);
    write(buf, constraint.result_x);
    write(buf, constraint.result_y);
}

} // namespace acir_format
