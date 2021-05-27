#pragma once
#include <stdlib/merkle_tree/merkle_tree.hpp>
#include "../proofs/notes/native/defi_interaction/defi_interaction_note.hpp"
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
        auto data_root = to_buffer(data_tree.root());
        root_tree.update_element(root_tree.size(), data_root);
    }

    template <typename T> void append_data_note(T const& note) { append_note(note, data_tree); }

    void add_defi_notes(std::vector<defi_interaction::defi_interaction_note> const& din)
    {
        for (auto& interaction_note : din) {
            insert_note(interaction_note, interaction_note.interaction_nonce, defi_tree);
        }
    }

    void nullify(uint256_t index) { null_tree.update_element(index, { 1 }); }

    Store store;
    Tree data_tree;
    Tree null_tree;
    Tree root_tree;
    Tree defi_tree;

  private:
    template <typename T> void insert_note(T const& note, uint256_t index, Tree& tree)
    {
        auto enc_note = encrypt(note);
        tree.update_element(index, to_buffer(enc_note));
    }

    template <typename T> void append_note(T const& note, Tree& tree) { insert_note(note, tree.size(), tree); }
};

} // namespace world_state
} // namespace rollup