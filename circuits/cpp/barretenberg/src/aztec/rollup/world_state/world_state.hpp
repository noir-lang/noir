#pragma once
#include <stdlib/merkle_tree/merkle_tree.hpp>
#include "../proofs/notes/native/defi_interaction/note.hpp"
#include "../proofs/notes/native/value/value_note.hpp"
#include "../proofs/notes/native/account/account_note.hpp"
#include "../proofs/notes/native/claim/claim_note.hpp"
#include "../constants.hpp"

namespace rollup {
namespace world_state {

using namespace plonk::stdlib::merkle_tree;
using namespace proofs::notes::native;

template <typename Store> class WorldState {
    using Tree = MerkleTree<Store>;

  public:
    WorldState()
        : data_tree(store, DATA_TREE_DEPTH, 0)
        , null_tree(store, NULL_TREE_DEPTH, 1)
        , root_tree(store, ROOT_TREE_DEPTH, 2)
        , defi_tree(store, DEFI_TREE_DEPTH, 3)
    {
        update_root_tree_with_data_root();
    }

    void update_root_tree_with_data_root()
    {
        auto data_root = data_tree.root();
        root_tree.update_element(root_tree.size(), data_root);
    }

    void insert_data_entry(uint256_t index, fr const& commitment, fr const& input_nullifier)
    {
        data_tree.update_element(index, commitment);
        input_nullifiers.resize(static_cast<size_t>(data_tree.size()));
        input_nullifiers[static_cast<size_t>(index)] = input_nullifier;
    }

    template <typename T> void append_data_note(T const& note)
    {
        insert_data_entry(data_tree.size(), note.commit(), note.input_nullifier);
    }

    void append_data_note(account::account_note const& note)
    {
        insert_data_entry(data_tree.size(), note.commit(), fr(0));
    }

    void add_defi_notes(std::vector<defi_interaction::note> const& din, uint32_t start_index)
    {
        for (uint32_t i = 0; i < din.size(); i++) {
            defi_tree.update_element(start_index + i, din[i].commit());
        }
    }

    void nullify(uint256_t index) { null_tree.update_element(index, { 1 }); }

    Store store;
    Tree data_tree;
    Tree null_tree;
    Tree root_tree;
    Tree defi_tree;
    std::vector<barretenberg::fr> input_nullifiers;
};

} // namespace world_state
} // namespace rollup