#pragma once
#include "barretenberg/crypto/merkle_tree/types.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include <cstdint>
#include <string>

namespace bb::crypto::merkle_tree {

struct TreeMeta {
    std::string name;
    uint32_t depth;
    index_t size;
    bb::fr root;

    MSGPACK_FIELDS(name, depth, size, root)
};

struct LeavesMeta {
    index_t size;

    MSGPACK_FIELDS(size)
};

} // namespace bb::crypto::merkle_tree
