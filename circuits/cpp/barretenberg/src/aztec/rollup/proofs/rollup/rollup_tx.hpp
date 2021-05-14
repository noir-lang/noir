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

    fr data_roots_root;
    std::vector<fr_hash_path> data_roots_paths;
    std::vector<uint32_t> data_roots_indicies;

    bool operator==(rollup_tx const&) const = default;
};

template <typename B> inline void read(B& buf, rollup_tx& tx)
{
    using serialize::read;
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

    read(buf, tx.data_roots_root);
    read(buf, tx.data_roots_paths);
    read(buf, tx.data_roots_indicies);
}

template <typename B> inline void write(B& buf, rollup_tx const& tx)
{
    using serialize::write;
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

    write(buf, tx.data_roots_root);
    write(buf, tx.data_roots_paths);
    write(buf, tx.data_roots_indicies);
}

inline std::ostream& operator<<(std::ostream& os, rollup_tx const& tx)
{
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

    os << "data_roots_root: " << tx.data_roots_root << "\n";
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
