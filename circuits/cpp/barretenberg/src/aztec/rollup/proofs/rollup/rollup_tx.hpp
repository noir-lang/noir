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

    fr old_null_root;
    std::vector<fr> new_null_roots;
    std::vector<fr_hash_path> old_null_paths;

    fr data_roots_root;
    // Paths at indicies proving each tx proofs data root is valid.
    std::vector<fr_hash_path> data_roots_paths;
    std::vector<uint32_t> data_roots_indicies;

    // The defi root after inserting the interaction notes.
    fr new_defi_root;
    // All defi deposits must match one of these.
    std::vector<uint256_t> bridge_ids;

    // Each asset must match one of these.
    std::vector<uint256_t> asset_ids;

    bool operator==(rollup_tx const&) const = default;

    // Not serialized or known about externally. Populated before the tx is padded.
    size_t num_defi_interactions;

    // Not serialized or known about externally. Number of assets (< NUM_ASSETS) allowed in this rollup.
    size_t num_asset_ids;
};

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

    read(buf, tx.old_null_root);
    read(buf, tx.new_null_roots);
    read(buf, tx.old_null_paths);

    read(buf, tx.data_roots_root);
    read(buf, tx.data_roots_paths);
    read(buf, tx.data_roots_indicies);

    read(buf, tx.new_defi_root);
    read(buf, tx.bridge_ids);
    read(buf, tx.asset_ids);
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

    write(buf, tx.old_null_root);
    write(buf, tx.new_null_roots);
    write(buf, tx.old_null_paths);

    write(buf, tx.data_roots_root);
    write(buf, tx.data_roots_paths);
    write(buf, tx.data_roots_indicies);

    write(buf, tx.new_defi_root);
    write(buf, tx.bridge_ids);
    write(buf, tx.asset_ids);
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

    os << "data_roots_root: " << tx.data_roots_root << "\n";
    os << "data_roots_paths:\n";
    for (auto e : tx.data_roots_paths) {
        os << e << "\n";
    }
    os << "data_roots_indicies: " << tx.data_roots_indicies;
    os << "new_defi_root: " << tx.new_defi_root << "\n";
    os << "bridge_ids: " << tx.bridge_ids;
    os << "asset_ids: " << tx.asset_ids;
    return os;
}

} // namespace rollup
} // namespace proofs
} // namespace rollup
