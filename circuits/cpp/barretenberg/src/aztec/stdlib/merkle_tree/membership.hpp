#pragma once
#include "hash_path.hpp"
#include <stdlib/hash/pedersen/pedersen.hpp>
#include <stdlib/primitives/byte_array/byte_array.hpp>
#include <stdlib/primitives/field/field.hpp>

namespace plonk {
namespace stdlib {
namespace merkle_tree {

template <typename Composer>
bool_t<Composer> check_subtree_membership(Composer& composer,
                                          field_t<Composer> const& root,
                                          hash_path<Composer> const& hashes,
                                          field_t<Composer> const& value,
                                          byte_array<Composer> const& index,
                                          size_t at_height)
{
    auto current = value;
    bool_t is_member = witness_t(&composer, true);

    for (size_t i = at_height; i < hashes.size(); ++i) {
        bool_t path_bit = index.get_bit(i);

        bool_t is_left = (current == hashes[i].first) & !path_bit;
        bool_t is_right = (current == hashes[i].second) & path_bit;
        is_member &= is_left ^ is_right;
        // if (!is_member.get_value()) {
        //     std::cout << "failed at height " << i << std::endl;
        //     std::cout << "is_left " << is_left.get_value() << std::endl;
        //     std::cout << "is_right " << is_right.get_value() << std::endl;
        // }
        current = pedersen<Composer>::compress(hashes[i].first, hashes[i].second);
    }

    // std::cout << "current " << current << " root " << root << std::endl;

    is_member &= current == root;
    return is_member;
}

template <typename Composer>
void assert_check_subtree_membership(Composer& composer,
                                     field_t<Composer> const& root,
                                     hash_path<Composer> const& hashes,
                                     field_t<Composer> const& value,
                                     byte_array<Composer> const& index,
                                     size_t at_height)
{
    auto exists = check_subtree_membership(composer, root, hashes, value, index, at_height);
    composer.assert_equal_constant(exists.witness_index, fr::one());
}

template <typename Composer>
bool_t<Composer> check_membership(Composer& composer,
                                  field_t<Composer> const& root,
                                  hash_path<Composer> const& hashes,
                                  byte_array<Composer> const& value,
                                  byte_array<Composer> const& index)
{
    return check_subtree_membership(composer, root, hashes, hash_value(value), index, 0);
}

template <typename Composer>
void assert_check_membership(Composer& composer,
                             field_t<Composer> const& root,
                             hash_path<Composer> const& hashes,
                             byte_array<Composer> const& value,
                             byte_array<Composer> const& index)
{
    auto exists = stdlib::merkle_tree::check_membership(composer, root, hashes, value, index);
    composer.assert_equal_constant(exists.witness_index, fr::one());
}

template <typename Composer>
void update_membership(Composer& composer,
                       field_t<Composer> const& new_root,
                       hash_path<Composer> const& new_hashes,
                       byte_array<Composer> const& new_value,
                       field_t<Composer> const& old_root,
                       hash_path<Composer> const& old_hashes,
                       byte_array<Composer> const& old_value,
                       byte_array<Composer> const& index)
{
    // Check that the old_value, is in the tree given by old_root, at index.
    assert_check_membership(composer, old_root, old_hashes, old_value, index);

    // Check that the new_value, is in the tree given by new_root, at index.
    assert_check_membership(composer, new_root, new_hashes, new_value, index);

    // Check that the old and new values, are actually in the same tree.
    for (size_t i = 0; i < new_hashes.size(); ++i) {
        bool_t path_bit = index.get_bit(i);
        bool_t share_left = (old_hashes[i].first == new_hashes[i].first) & path_bit;
        bool_t share_right = (old_hashes[i].second == new_hashes[i].second) & !path_bit;
        composer.assert_equal_constant((share_left ^ share_right).witness_index, barretenberg::fr::one());
    }
}

template <typename Composer>
void update_subtree_membership(Composer& composer,
                               field_t<Composer> const& new_root,
                               hash_path<Composer> const& new_hashes,
                               field_t<Composer> const& new_subtree_root,
                               field_t<Composer> const& old_root,
                               hash_path<Composer> const& old_hashes,
                               field_t<Composer> const& old_subtree_root,
                               byte_array<Composer> const& index,
                               size_t at_height)
{
    // Check check that the old_subtree_root, is in the tree given by old_root, at index and at_height.
    assert_check_subtree_membership(composer, old_root, old_hashes, old_subtree_root, index, at_height);

    // Check check that the new_subtree_root, is in the tree given by new_root, at index and at_height.
    assert_check_subtree_membership(composer, new_root, new_hashes, new_subtree_root, index, at_height);

    // Check that the old and new values, are actually in the same tree.
    for (size_t i = at_height; i < new_hashes.size(); ++i) {
        bool_t path_bit = index.get_bit(i);
        bool_t share_left = (old_hashes[i].first == new_hashes[i].first) & path_bit;
        bool_t share_right = (old_hashes[i].second == new_hashes[i].second) & !path_bit;
        composer.assert_equal_constant((share_left ^ share_right).witness_index, barretenberg::fr::one());
    }
}

template <typename Composer> field_t<Composer> compute_tree_root(std::vector<byte_array<Composer>> const& values)
{
    std::vector<field_t<Composer>> layer(values.size());
    for (size_t i = 0; i < values.size(); ++i) {
        layer[i] = hash_value(values[i]);
    }

    while (layer.size() > 1) {
        std::vector<field_t<Composer>> next_layer(layer.size() / 2);
        for (size_t i = 0; i < next_layer.size(); ++i) {
            next_layer[i] = pedersen<Composer>::compress(layer[i * 2], layer[i * 2 + 1]);
        }
        layer = std::move(next_layer);
    }

    return layer[0];
}

template <typename Composer>
bool_t<Composer> check_tree(field_t<Composer> const& root, std::vector<byte_array<Composer>> const& values)
{
    return compute_tree_root(values) == root;
}

template <typename Composer>
void assert_check_tree(Composer& composer,
                       field_t<Composer> const& root,
                       std::vector<byte_array<Composer>> const& values)
{
    auto valid = check_tree(root, values);
    composer.assert_equal_constant(valid.witness_index, fr::one());
}

} // namespace merkle_tree
} // namespace stdlib
} // namespace plonk
