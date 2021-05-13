#pragma once
#include <algorithm>
#include <arpa/inet.h>
#include <common/serialize.hpp>
#include <common/streams.hpp>
#include <ecc/curves/bn254/fr.hpp>
#include <sstream>
#include <stdlib/merkle_tree/hash_path.hpp>
#include "../notes/circuit/defi_interaction/defi_interaction_note.hpp"

namespace rollup {
namespace proofs {
namespace root_rollup {

using namespace barretenberg;
using namespace plonk::stdlib::merkle_tree;
using namespace notes::circuit::defi_interaction;

struct root_rollup_tx {
    uint32_t num_inner_proofs;
    uint32_t rollup_id;
    std::vector<std::vector<uint8_t>> rollups;
    fr old_data_roots_root;
    fr new_data_roots_root;
    fr_hash_path old_data_roots_path;
    fr_hash_path new_data_roots_path;

    // We need a new data root since we'd be adding defi_interaction_notes in the data tree.
    // We intend to add a height 2 subtree of defi_interaction_notes in the last 4 leaves of
    // the data tree.
    fr new_data_root;
    fr_hash_path old_data_path;
    fr_hash_path new_data_path;

    // The bridge_ids and the interaction nonce is passed as private input to the root rollup circuit.
    // The rollup provider would mix-in the interaction nonce in the claim notes of defi deposits.
    std::vector<field_ct> bridge_ids;
    uint32_ct interaction_nonce;

    // The defi interaction notes would also be private input to a root rollup.
    std::vector<defi_interaction_note> defi_interaction_notes;

    bool operator==(root_rollup_tx const&) const = default;
};

template <typename B> inline void read(B& buf, root_rollup_tx& tx)
{
    using serialize::read;
    read(buf, tx.num_inner_proofs);
    read(buf, tx.rollup_id);
    read(buf, tx.rollups);
    read(buf, tx.old_data_roots_root);
    read(buf, tx.new_data_roots_root);
    read(buf, tx.old_data_roots_path);
    read(buf, tx.new_data_roots_path);
}

template <typename B> inline void write(B& buf, root_rollup_tx const& tx)
{
    using serialize::write;
    write(buf, tx.num_inner_proofs);
    write(buf, tx.rollup_id);
    write(buf, tx.rollups);
    write(buf, tx.old_data_roots_root);
    write(buf, tx.new_data_roots_root);
    write(buf, tx.old_data_roots_path);
    write(buf, tx.new_data_roots_path);
}

inline std::ostream& operator<<(std::ostream& os, root_rollup_tx const& tx)
{
    os << "num_inner_proofs: " << tx.num_inner_proofs << "\n";
    os << "rollup_id: " << tx.rollup_id << "\n";
    os << "proof_data:\n";
    for (auto p : tx.rollups) {
        os << p << "\n";
    }
    os << "old_data_roots_root: " << tx.old_data_roots_root << "\n";
    os << "new_data_roots_root: " << tx.new_data_roots_root << "\n";
    os << "old_data_roots_path: " << tx.old_data_roots_path << "\n";
    os << "new_data_roots_path: " << tx.new_data_roots_path << "\n";
    return os;
}

} // namespace root_rollup
} // namespace proofs
} // namespace rollup
