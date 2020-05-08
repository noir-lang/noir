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
    std::vector<std::vector<uint8_t>> txs;
    fr old_data_root;
    fr old_null_root;
    std::vector<std::array<fr_hash_path, 2>> old_data_paths;
    std::vector<std::array<fr_hash_path, 2>> old_null_paths;
    fr new_data_root;
    fr new_null_root;
    std::vector<std::array<fr_hash_path, 2>> new_data_paths;
    std::vector<std::array<fr_hash_path, 2>> new_null_paths;
};

template <typename B> void read(B& buf, rollup_tx& tx)
{
    ::read(buf, tx.rollup_id);
    ::read(buf, tx.num_txs);
    ::read(buf, tx.proof_lengths);
    read(buf, tx.txs);
    read(buf, tx.old_data_root);
    read(buf, tx.old_null_root);
    read(buf, tx.old_data_paths);
    read(buf, tx.old_null_paths);
    read(buf, tx.new_data_root);
    read(buf, tx.new_null_root);
    read(buf, tx.new_data_paths);
    read(buf, tx.new_null_paths);
}

template <typename B> void write(B& buf, rollup_tx const& tx)
{
    ::write(buf, tx.rollup_id);
    ::write(buf, tx.num_txs);
    ::write(buf, tx.proof_lengths);
    write(buf, tx.txs);
    write(buf, tx.old_data_root);
    write(buf, tx.old_null_root);
    write(buf, tx.old_data_paths);
    write(buf, tx.old_null_paths);
    write(buf, tx.new_data_root);
    write(buf, tx.new_null_root);
    write(buf, tx.new_data_paths);
    write(buf, tx.new_null_paths);
}

bool operator==(rollup_tx const& lhs, rollup_tx const& rhs){
    return lhs.rollup_id == rhs.rollup_id && lhs.num_txs == rhs.num_txs && lhs.proof_lengths == rhs.proof_lengths &&
           lhs.txs == rhs.txs && lhs.old_data_root == rhs.old_data_root && lhs.old_null_root == rhs.old_null_root &&
           lhs.old_data_paths == rhs.old_data_paths && lhs.old_null_paths == rhs.old_null_paths &&
           lhs.new_data_root == rhs.new_data_root && lhs.new_null_root == rhs.new_null_root &&
           lhs.new_data_paths == rhs.new_data_paths && lhs.new_null_paths == rhs.new_null_paths;
}

std::ostream&
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
    for (auto tx_data_paths : tx.old_data_paths) {
        os << tx_data_paths[0];
        os << tx_data_paths[1];
    }
    os << "old_null_paths:\n";
    for (auto tx_null_paths : tx.old_null_paths) {
        os << tx_null_paths[0];
        os << tx_null_paths[1];
    }
    os << "new_data_root: " << tx.new_data_root << "\n";
    os << "new_null_root: " << tx.new_null_root << "\n";
    os << "new_data_paths:\n";
    for (auto tx_data_paths : tx.new_data_paths) {
        os << tx_data_paths[0];
        os << tx_data_paths[1];
    }
    os << "new_null_paths:\n";
    for (auto tx_null_paths : tx.new_null_paths) {
        os << tx_null_paths[0];
        os << tx_null_paths[1];
    }
    return os;
}

} // namespace rollup_proofs
} // namespace rollup