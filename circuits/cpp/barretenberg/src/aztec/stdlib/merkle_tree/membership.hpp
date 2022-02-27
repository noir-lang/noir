#pragma once
#include "hash_path.hpp"
#include <stdlib/hash/pedersen/pedersen.hpp>
#include <stdlib/primitives/byte_array/byte_array.hpp>
#include <stdlib/primitives/field/field.hpp>

namespace plonk {
namespace stdlib {
namespace merkle_tree {

template <typename ComposerContext> using bit_vector = std::vector<bool_t<ComposerContext>>;
/**
 * Checks if the subtree is correctly inserted at a specified index in a Merkle tree.
 *
 * @param root: The root of the latest state of the merkle tree,
 * @param hashes: The hash path from any leaf in the subtree to the root, it doesn't matter if this hash path is
 * computed before or after updating the tree,
 * @param value: The value of the subtree root,
 * @param index: The index of any leaf in the subtree,
 * @param at_height: The height of the subtree,
 * @param is_updating_tree: set to true if we're updating the tree.
 * @tparam Composer: type of composer.
 *
 * @see Check full documentation: https://hackmd.io/2zyJc6QhRuugyH8D78Tbqg?view
 */
template <typename Composer>
bool_t<Composer> check_subtree_membership(field_t<Composer> const& root,
                                          hash_path<Composer> const& hashes,
                                          field_t<Composer> const& value,
                                          bit_vector<Composer> const& index,
                                          size_t at_height,
                                          bool const is_updating_tree = false)
{
    auto is_zero = value.is_zero();
    auto current = field_t<Composer>::conditional_assign(is_zero, (-field_t<Composer>(1)), value);

    for (size_t i = at_height; i < hashes.size(); ++i) {
        // get the parity bit at this level of the tree (get_bit returns bool so we know this is 0 or 1)
        bool_t<Composer> path_bit = index[i];

        // reconstruct the two inputs we need to hash
        // if `path_bit = false`, we know `current` is the left leaf and `hashes[i].second` is the right leaf
        // if `path_bit = true`, we know `current` is the right leaf and `hashes[i].first` is the left leaf
        // We don't need to explicitly check that hashes[i].first = current iff !path bit , or that hashes[i].second =
        // current iff path_bit If either of these does not hold, then the final computed merkle root will not match
        field_t<Composer> left = field_t<Composer>::conditional_assign(path_bit, hashes[i].first, current);
        field_t<Composer> right = field_t<Composer>::conditional_assign(path_bit, current, hashes[i].second);
        current = pedersen<Composer>::compress_unsafe(left, right, 0, false, is_updating_tree);
    }

    return (current == root);
}

/**
 * Asserts if the subtree is correctly inserted at a specified index in a Merkle tree.
 *
 * @param root: The root of the latest state of the merkle tree,
 * @param hashes: The hash path from any leaf in the subtree to the root, it doesn't matter if this hash path is
 * computed before or after updating the tree,
 * @param value: The value of the subtree root,
 * @param index: The index of any leaf in the subtree,
 * @param at_height: The height of the subtree,
 * @param is_updating_tree: set to true if we're updating the tree,
 * @param msg: error message.
 * @tparam Composer: type of composer.
 */
template <typename Composer>
void assert_check_subtree_membership(field_t<Composer> const& root,
                                     hash_path<Composer> const& hashes,
                                     field_t<Composer> const& value,
                                     bit_vector<Composer> const& index,
                                     size_t at_height,
                                     bool const is_updating_tree = false,
                                     std::string const& msg = "assert_check_subtree_membership")
{
    auto exists = check_subtree_membership(root, hashes, value, index, at_height, is_updating_tree);
    exists.assert_equal(true, msg);
}

/**
 * Checks if a value is correctly inserted at a specified leaf in a Merkle tree.
 *
 * @param root: The root of the updated merkle tree,
 * @param hashes: The hash path from the given index, it doesn't matter if this hash path is
 * computed before or after updating the tree,
 * @param value: The value of the leaf,
 * @param index: The index of the leaf in the tree,
 * @param is_updating_tree: set to true if we're updating the tree.
 * @tparam Composer: type of composer.
 */
template <typename Composer>
bool_t<Composer> check_membership(field_t<Composer> const& root,
                                  hash_path<Composer> const& hashes,
                                  field_t<Composer> const& value,
                                  bit_vector<Composer> const& index,
                                  bool const is_updating_tree = false)
{
    return check_subtree_membership(root, hashes, value, index, 0, is_updating_tree);
}

/**
 * Asserts if a value is correctly inserted at a specified leaf in a Merkle tree.
 *
 * @param root: The root of the updated merkle tree,
 * @param hashes: The hash path from the given index, it doesn't matter if this hash path is
 * computed before or after updating the tree,
 * @param value: The value of the leaf,
 * @param index: The index of the leaf in the tree,
 * @param is_updating_tree: set to true if we're updating the tree,
 * @param msg: error message.
 * @tparam Composer: type of composer.
 */
template <typename Composer>
void assert_check_membership(field_t<Composer> const& root,
                             hash_path<Composer> const& hashes,
                             field_t<Composer> const& value,
                             bit_vector<Composer> const& index,
                             bool const is_updating_tree = false,
                             std::string const& msg = "assert_check_membership")
{
    auto exists = stdlib::merkle_tree::check_membership(root, hashes, value, index, is_updating_tree);
    exists.assert_equal(true, msg);
}

/**
 * Asserts if old and new state of the tree is correct after updating a single leaf.
 *
 * @param new_root: The root of the updated merkle tree,
 * @param new_value: The new value of the leaf,
 * @param old_root: The root of the merkle tree before it was updated,
 * @param old_hashes: The hash path from the given index, it doesn't matter if this hash path is
 * computed before or after updating the tree,
 * @param old_value: The value of the leaf before it was updated with new_value,
 * @param index: The index of the leaf in the tree,
 * @param msg: error message.
 * @tparam Composer: type of composer.
 */
template <typename Composer>
void update_membership(field_t<Composer> const& new_root,
                       field_t<Composer> const& new_value,
                       field_t<Composer> const& old_root,
                       hash_path<Composer> const& old_hashes,
                       field_t<Composer> const& old_value,
                       bit_vector<Composer> const& index,
                       std::string const& msg = "update_membership")
{
    // Check that the old_value, is in the tree given by old_root, at index.
    assert_check_membership(old_root, old_hashes, old_value, index, false, msg + "_old_value");

    // Check that the new_value, is in the tree given by new_root, at index.
    assert_check_membership(new_root, old_hashes, new_value, index, true, msg + "_new_value");
}

/**
 * Asserts if old and new state of the tree is correct after a subtree-update.
 *
 * @param new_root: The root of the updated merkle tree,
 * @param new_subtree_root: The new value of the subtree root,
 * @param old_root: The root of the merkle tree before it was updated,
 * @param old_hashes: The hash path from any leaf in the subtree to the root, it doesn't matter if this hash path is
 * computed before or after updating the tree,
 * @param old_subtree_root: The value of the subtree root before it was updated,
 * @param index: The index of any leaf in the subtree,
 * @param at_height: The height of the subtree,
 * @param msg: error message.
 * @tparam Composer: type of composer.
 */
template <typename Composer>
void update_subtree_membership(field_t<Composer> const& new_root,
                               field_t<Composer> const& new_subtree_root,
                               field_t<Composer> const& old_root,
                               hash_path<Composer> const& old_hashes,
                               field_t<Composer> const& old_subtree_root,
                               bit_vector<Composer> const& index,
                               size_t at_height,
                               std::string const& msg = "update_subtree_membership")
{
    // Check that the old_subtree_root, is in the tree given by old_root, at index and at_height.
    assert_check_subtree_membership(
        old_root, old_hashes, old_subtree_root, index, at_height, false, msg + "_old_subtree");

    // Check that the new_subtree_root, is in the tree given by new_root, at index and at_height.
    // By extracting partner hashes from `old_hashes`, we also validate both membership proofs use
    // identical merkle trees (apart from the leaf that is being updated)
    assert_check_subtree_membership(
        new_root, old_hashes, new_subtree_root, index, at_height, true, msg + "_new_subtree");
}

/**
 * Computes the root of a tree with leaves given as the vector `input`.
 *
 * @param input: vector of leaf values.
 * @tparam Composer: type of composer.
 */
template <typename Composer> field_t<Composer> compute_tree_root(std::vector<field_t<Composer>> const& input)
{
    // Check if the input vector size is a power of 2.
    ASSERT(input.size() > 0);
    ASSERT(!(input.size() & (input.size() - 1)) == true);
    auto layer = input;
    for (auto& f : layer) {
        auto is_zero = f.is_zero();
        f = field_t<Composer>::conditional_assign(is_zero, (-field_t<Composer>(1)), f);
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

/**
 * Checks if a given root matches the root computed from a set of leaves.
 *
 * @param values: vector of leaf values.
 * @tparam Composer: type of composer.
 */
template <typename Composer>
bool_t<Composer> check_tree(field_t<Composer> const& root, std::vector<field_t<Composer>> const& values)
{
    return compute_tree_root(values) == root;
}

/**
 * Asserts if a given root matches the root computed from a set of leaves.
 *
 * @param values: vector of leaf values.
 * @tparam Composer: type of composer.
 */
template <typename Composer>
void assert_check_tree(field_t<Composer> const& root, std::vector<field_t<Composer>> const& values)
{
    auto valid = check_tree(root, values);
    valid.assert_equal(true, "assert_check_tree");
}

/**
 * Updates the tree with a vector of new values starting from the leaf at start_index.
 *
 * @param new_root: The root of the updated merkle tree,
 * @param old_root: The root of the merkle tree before it was updated,
 * @param old_hashes: The hash path from the leaf at start_index, computed before the tree was updated
 * @param new_values: The vector of values to be inserted from start_index,
 * @param start_index: The index of any leaf from which new values are inserted,
 * @param msg: error message.
 * @tparam Composer: type of composer.
 */
template <typename Composer>
void batch_update_membership(field_t<Composer> const& new_root,
                             field_t<Composer> const& old_root,
                             hash_path<Composer> const& old_path,
                             std::vector<field_t<Composer>> const& new_values,
                             field_t<Composer> const& start_index,
                             std::string const& msg = "batch_update_membership")
{
    size_t height = numeric::get_msb(new_values.size());
    auto zero_subtree_root = field_t<Composer>(zero_hash_at_height(height));

    auto rollup_root = compute_tree_root(new_values);

    update_subtree_membership(
        new_root, rollup_root, old_root, old_path, zero_subtree_root, start_index.decompose_into_bits(), height, msg);
}

} // namespace merkle_tree
} // namespace stdlib
} // namespace plonk
