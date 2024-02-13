#pragma once
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include <cstdint>
#include <vector>

namespace acir_format {

struct Poseidon2Constraint {
    std::vector<uint32_t> state;
    std::vector<uint32_t> result;
    uint32_t len;

    // For serialization, update with any new fields
    MSGPACK_FIELDS(state, result, len);
    friend bool operator==(Poseidon2Constraint const& lhs, Poseidon2Constraint const& rhs) = default;
};

template <typename Builder> void create_poseidon2_permutations(Builder& builder, const Poseidon2Constraint& constraint);

} // namespace acir_format