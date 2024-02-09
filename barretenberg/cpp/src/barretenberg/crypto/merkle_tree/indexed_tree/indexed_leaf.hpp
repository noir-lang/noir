#pragma once

#include "barretenberg/stdlib/primitives/field/field.hpp"

namespace bb::crypto::merkle_tree {

typedef uint256_t index_t;

struct indexed_leaf {
    fr value;
    index_t nextIndex;
    fr nextValue;

    bool operator==(indexed_leaf const&) const = default;

    std::ostream& operator<<(std::ostream& os)
    {
        os << "value = " << value << "\nnextIdx = " << nextIndex << "\nnextVal = " << nextValue;
        return os;
    }

    std::vector<fr> get_hash_inputs() const { return std::vector<fr>({ value, nextIndex, nextValue }); }
};

} // namespace bb::crypto::merkle_tree