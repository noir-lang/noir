#pragma once
#include <algorithm>
#include <arpa/inet.h>
#include <sstream>
#include <common/serialize.hpp>
#include <common/streams.hpp>
#include <ecc/curves/bn254/fr.hpp>
#include <stdlib/merkle_tree/hash_path.hpp>

namespace rollup {
namespace rollup_proofs {

using namespace barretenberg;
using namespace plonk::stdlib::merkle_tree;

struct rollup_tx {
    uint32_t rollup_id;
    uint32_t num_txs;
    uint32_t data_start_index;
    std::vector<std::vector<uint8_t>> txs;

    fr rollup_root;
    fr old_data_root;
    fr new_data_root;
    fr_hash_path old_data_path;
    fr_hash_path new_data_path;

    fr old_null_root;
    std::vector<fr> new_null_roots;
    std::vector<fr_hash_path> old_null_paths;
    std::vector<fr_hash_path> new_null_paths;

    bool operator==(rollup_tx const&) const = default;
};

template <typename B> inline void read(B& buf, rollup_tx& tx)
{
    ::read(buf, tx.rollup_id);
    ::read(buf, tx.num_txs);
    ::read(buf, tx.data_start_index);
    read(buf, tx.txs);

    read(buf, tx.rollup_root);
    read(buf, tx.old_data_root);
    read(buf, tx.new_data_root);
    read(buf, tx.old_data_path);
    read(buf, tx.new_data_path);

    read(buf, tx.old_null_root);
    read(buf, tx.new_null_roots);
    read(buf, tx.old_null_paths);
    read(buf, tx.new_null_paths);
}

template <typename B> inline void write(B& buf, rollup_tx const& tx)
{
    ::write(buf, tx.rollup_id);
    ::write(buf, tx.num_txs);
    ::write(buf, tx.data_start_index);

    write(buf, tx.txs);
    write(buf, tx.rollup_root);
    write(buf, tx.old_data_root);
    write(buf, tx.new_data_root);
    write(buf, tx.old_data_path);
    write(buf, tx.new_data_path);

    write(buf, tx.old_null_root);
    write(buf, tx.new_null_roots);
    write(buf, tx.old_null_paths);
    write(buf, tx.new_null_paths);
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
    os << "rollup_root: " << tx.rollup_root << "\n";
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
    return os;
}

} // namespace rollup_proofs
} // namespace rollup
