#pragma once
#include <algorithm>
#include <arpa/inet.h>
#include <common/serialize.hpp>
#include <common/streams.hpp>
#include <ecc/curves/bn254/fr.hpp>
#include <sstream>
#include <stdlib/merkle_tree/hash_path.hpp>

namespace rollup {
namespace proofs {
namespace rollup {

using namespace barretenberg;
using namespace plonk::stdlib::merkle_tree;

struct rollup_tx {
    uint32_t rollup_id;
    uint32_t num_txs;
    uint32_t data_start_index;
    std::vector<std::vector<uint8_t>> txs;

    fr old_data_root;
    fr new_data_root;
    fr_hash_path old_data_path;
    fr_hash_path new_data_path;

    fr old_null_root;
    std::vector<fr> new_null_roots;
    std::vector<fr_hash_path> old_null_paths;
    std::vector<fr_hash_path> new_null_paths;

    fr old_data_roots_root;
    fr new_data_roots_root;
    fr_hash_path old_data_roots_path;
    fr_hash_path new_data_roots_path;
    std::vector<fr_hash_path> data_roots_paths;
    std::vector<uint32_t> data_roots_indicies;

    // bool operator==(rollup_tx const&) const = default;
};

inline bool operator==(rollup_tx const& lhs, rollup_tx const& rhs)
{
    // clang-format off
    return
        lhs.rollup_id == rhs.rollup_id &&
        lhs.num_txs == rhs.num_txs &&
        lhs.data_start_index == rhs.data_start_index &&
        lhs.txs == rhs.txs &&

        lhs.old_data_root == rhs.old_data_root &&
        lhs.new_data_root == rhs.new_data_root &&
        lhs.old_data_path == rhs.old_data_path &&
        lhs.new_data_path == rhs.new_data_path &&

        lhs.old_null_root == rhs.old_null_root &&
        lhs.new_null_roots == rhs.new_null_roots &&
        lhs.old_null_paths == rhs.old_null_paths &&
        lhs.new_null_paths == rhs.new_null_paths &&

        lhs.old_data_roots_root == rhs.old_data_roots_root &&
        lhs.new_data_roots_root == rhs.new_data_roots_root &&
        lhs.old_data_roots_path == rhs.old_data_roots_path &&
        lhs.new_data_roots_path == rhs.new_data_roots_path &&
        lhs.data_roots_paths == rhs.data_roots_paths &&
        lhs.data_roots_indicies == rhs.data_roots_indicies;
    // clang-format on
}

template <typename B> inline void read(B& buf, rollup_tx& tx)
{
    using serialize::read;
    read(buf, tx.rollup_id);
    read(buf, tx.num_txs);
    read(buf, tx.data_start_index);
    read(buf, tx.txs);

    read(buf, tx.old_data_root);
    read(buf, tx.new_data_root);
    read(buf, tx.old_data_path);
    read(buf, tx.new_data_path);

    read(buf, tx.old_null_root);
    read(buf, tx.new_null_roots);
    read(buf, tx.old_null_paths);
    read(buf, tx.new_null_paths);

    read(buf, tx.old_data_roots_root);
    read(buf, tx.new_data_roots_root);
    read(buf, tx.old_data_roots_path);
    read(buf, tx.new_data_roots_path);
    read(buf, tx.data_roots_paths);
    read(buf, tx.data_roots_indicies);
}

template <typename B> inline void write(B& buf, rollup_tx const& tx)
{
    using serialize::write;
    write(buf, tx.rollup_id);
    write(buf, tx.num_txs);
    write(buf, tx.data_start_index);

    write(buf, tx.txs);
    write(buf, tx.old_data_root);
    write(buf, tx.new_data_root);
    write(buf, tx.old_data_path);
    write(buf, tx.new_data_path);

    write(buf, tx.old_null_root);
    write(buf, tx.new_null_roots);
    write(buf, tx.old_null_paths);
    write(buf, tx.new_null_paths);

    write(buf, tx.old_data_roots_root);
    write(buf, tx.new_data_roots_root);
    write(buf, tx.old_data_roots_path);
    write(buf, tx.new_data_roots_path);
    write(buf, tx.data_roots_paths);
    write(buf, tx.data_roots_indicies);
}

inline std::ostream& operator<<(std::ostream& os, rollup_tx const& tx)
{
    os << "rollup_id: " << tx.rollup_id << "\n";
    os << "num_txs: " << tx.num_txs << "\n";
    os << "data_start_index: " << tx.data_start_index << "\n";
    os << "proof_data:\n";
    for (auto p : tx.txs) {
        os << p << "\n";
    }

    os << "\nDATA TREE UPDATE CONTEXT:\n";
    os << "old_data_root: " << tx.old_data_root << "\n";
    os << "new_data_root: " << tx.new_data_root << "\n";
    os << "old_data_path: " << tx.old_data_path << "\n";
    os << "new_data_path: " << tx.new_data_path << "\n";

    os << "\nNULL TREE UPDATE CONTEXT:\n";
    os << "old_null_root: " << tx.old_null_root << "\n";
    os << "new_null_roots:\n";
    for (auto e : tx.new_null_roots) {
        os << e << "\n";
    }
    os << "old_null_paths:\n";
    for (auto e : tx.old_null_paths) {
        os << e << "\n";
    }
    os << "new_null_paths:\n";
    for (auto e : tx.new_null_paths) {
        os << e << "\n";
    }

    os << "old_data_roots_root: " << tx.old_data_roots_root << "\n";
    os << "new_data_roots_root: " << tx.new_data_roots_root << "\n";
    os << "old_data_roots_path: " << tx.old_data_roots_path << "\n";
    os << "new_data_roots_path: " << tx.new_data_roots_path << "\n";
    os << "data_roots_paths:\n";
    for (auto e : tx.data_roots_paths) {
        os << e << "\n";
    }
    os << "data_roots_indicies:\n";
    for (auto e : tx.data_roots_indicies) {
        os << e << "\n";
    }
    return os;
}

} // namespace rollup
} // namespace proofs
} // namespace rollup
