#pragma once
#include <algorithm>
#include <arpa/inet.h>
#include <common/serialize.hpp>
#include <common/streams.hpp>
#include <ecc/curves/bn254/fr.hpp>
#include <sstream>
#include <stdlib/merkle_tree/hash_path.hpp>
#include "../notes/native/defi_interaction/defi_interaction_note.hpp"
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace root_rollup {

using namespace barretenberg;
using namespace plonk::stdlib::merkle_tree;
using namespace notes::native::defi_interaction;

struct root_rollup_tx {
    // The maximum number of inner rollups in this proof.
    uint32_t num_inner_proofs;

    // The rollup id, which increases incrementally.
    uint32_t rollup_id;

    // If the size < num_inner_proofs, it's padded to size num_inner_proofs with the padding proof.
    std::vector<std::vector<uint8_t>> rollups;

    // For updating the tree of data roots.
    fr old_data_roots_root;
    fr new_data_roots_root;
    fr_hash_path old_data_roots_path;
    fr_hash_path new_data_roots_path;

    // The number of defi interactions in this rollup.
    uint32_t num_defi_interactions;

    // For updating the defi_interaction tree.
    fr old_defi_interaction_root;
    fr new_defi_interaction_root;
    fr_hash_path old_defi_interaction_path;
    fr_hash_path new_defi_interaction_path;

    // num_defi_interactions bridge ids.
    // Will be padded to NUM_BRIDGE_CALLS_PER_BLOCK.
    // All defi deposits must match one of these.
    std::vector<fr> bridge_ids;

    // The interaction nonce is added to the partial claim notes in the defi deposits.
    uint32_t interaction_nonce;

    // num_defi_interactions defi_interaction_notes;
    // Will be padded to NUM_BRIDGE_CALLS_PER_BLOCK.
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
    read(buf, tx.num_defi_interactions);
    read(buf, tx.old_defi_interaction_root);
    read(buf, tx.new_defi_interaction_root);
    read(buf, tx.old_defi_interaction_path);
    read(buf, tx.new_defi_interaction_path);
    read(buf, tx.bridge_ids);
    read(buf, tx.defi_interaction_notes);
    read(buf, tx.interaction_nonce);
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
    write(buf, tx.num_defi_interactions);
    write(buf, tx.old_defi_interaction_root);
    write(buf, tx.new_defi_interaction_root);
    write(buf, tx.old_defi_interaction_path);
    write(buf, tx.new_defi_interaction_path);
    write(buf, tx.bridge_ids);
    write(buf, tx.defi_interaction_notes);
    write(buf, tx.interaction_nonce);
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

    os << "num_defi_interactions: " << tx.num_defi_interactions << "\n";
    os << "old_defi_interaction_root: " << tx.old_defi_interaction_root << "\n";
    os << "new_defi_interaction_root: " << tx.new_defi_interaction_root << "\n";
    os << "old_defi_interaction_path: " << tx.old_defi_interaction_path << "\n";
    os << "new_defi_interaction_path: " << tx.new_defi_interaction_path << "\n";

    os << "bridge_ids:\n";
    for (auto bridge_id : tx.bridge_ids) {
        os << bridge_id << "\n";
    }
    os << "interaction_nonce: " << tx.interaction_nonce << "\n";

    size_t i = 0;
    for (auto defi_note : tx.defi_interaction_notes) {
        os << "defi_interaction_" << i << ":\n";
        os << "    bridge_id: " << defi_note.bridge_id << "\n";
        os << "    interaction_nonce: " << defi_note.interaction_nonce << "\n";
        os << "    total_input_value: " << defi_note.total_input_value << "\n";
        os << "    total_output_a_value: " << defi_note.total_output_a_value << "\n";
        os << "    total_output_b_value: " << defi_note.total_output_b_value << "\n";
        os << "    interaction_result: " << defi_note.interaction_result << "\n";
    }

    return os;
}

} // namespace root_rollup
} // namespace proofs
} // namespace rollup
