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
    uint32_t proof_lengths;
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
};

template <typename B>
inline void read(B& buf, rollup_tx& tx)
{
    ::read(buf, tx.rollup_id);
    ::read(buf, tx.num_txs);
    ::read(buf, tx.proof_lengths);
    ::read(buf, tx.data_start_index);
    read(buf, tx.txs);
    read(buf, tx.old_data_root);
    // read(buf, tx.old_null_root);
    // read(buf, tx.old_data_paths);
    // read(buf, tx.old_null_paths);
    // read(buf, tx.new_data_root);
    // read(buf, tx.new_null_root);
    // read(buf, tx.new_data_paths);
    // read(buf, tx.new_null_paths);
}

template <typename B>
inline void write(B& buf, rollup_tx const& tx)
{
    ::write(buf, tx.rollup_id);
    ::write(buf, tx.num_txs);
    ::write(buf, tx.proof_lengths);
    ::write(buf, tx.data_start_index);
    write(buf, tx.txs);
    write(buf, tx.old_data_root);
    // write(buf, tx.old_null_root);
    // write(buf, tx.old_data_paths);
    // write(buf, tx.old_null_paths);
    // write(buf, tx.new_data_root);
    // write(buf, tx.new_null_root);
    // write(buf, tx.new_data_paths);
    // write(buf, tx.new_null_paths);
}
/*
inline bool operator==(rollup_tx const& lhs, rollup_tx const& rhs){
    return lhs.rollup_id == rhs.rollup_id && lhs.num_txs == rhs.num_txs && lhs.proof_lengths == rhs.proof_lengths &&
           lhs.txs == rhs.txs && lhs.old_data_root == rhs.old_data_root && lhs.old_null_root == rhs.old_null_root &&
           lhs.old_data_paths == rhs.old_data_paths && lhs.old_null_paths == rhs.old_null_paths &&
           lhs.new_data_root == rhs.new_data_root && lhs.new_null_root == rhs.new_null_root &&
           lhs.new_data_paths == rhs.new_data_paths && lhs.new_null_paths == rhs.new_null_paths;
}

inline std::ostream&
operator<<(std::ostream& os, rollup_tx const& tx)
{
    os << "rollup_id: " << tx.rollup_id << "\n";
    os << "num_txs: " << tx.num_txs << "\n";
    os << "proof_lengths: " << tx.proof_lengths << "\n";
    os << "proof_data:\n";
    for (auto p : tx.txs) {
        os << p << "\n";
    }
    os << "old_data_root: " << tx.old_data_root << "\n";
    os << "old_null_root: " << tx.old_null_root << "\n";
    os << "old_data_paths:\n";
    for (auto e : tx.old_data_paths) {
        os << e.first << ": " << e.second;
    }
    os << "old_null_paths:\n";
    for (auto e : tx.old_null_paths) {
        os << e.first << ": " << e.second;
    }
    os << "new_data_root: " << tx.new_data_root << "\n";
    os << "new_null_root: " << tx.new_null_root << "\n";
    os << "new_data_paths:\n";
    for (auto e : tx.new_data_paths) {
        os << e.first << ": " << e.second;
    }
    os << "new_null_paths:\n";
    for (auto e : tx.new_null_paths) {
        os << e.first << ": " << e.second;
    }
    return os;
}
*/
} // namespace rollup_proofs
} // namespace rollup