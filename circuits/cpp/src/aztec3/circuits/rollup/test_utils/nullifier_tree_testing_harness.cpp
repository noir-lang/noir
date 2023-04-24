#include "nullifier_tree_testing_harness.hpp"
#include <barretenberg/stdlib/merkle_tree/nullifier_tree/nullifier_memory_tree.hpp>
#include <barretenberg/stdlib/merkle_tree/nullifier_tree/nullifier_leaf.hpp>
#include <cstdint>
#include <tuple>

using NullifierMemoryTree = proof_system::plonk::stdlib::merkle_tree::NullifierMemoryTree;
using nullifier_leaf = proof_system::plonk::stdlib::merkle_tree::nullifier_leaf;

NullifierMemoryTreeTestingHarness::NullifierMemoryTreeTestingHarness(size_t depth)
    : NullifierMemoryTree(depth)
{}

// Check for a larger value in an array
bool check_has_less_than(std::vector<fr> const& values, fr const& value)
{

    // Must perform comparisons on integers
    uint256_t value_as_uint = uint256_t(value);
    for (auto const& v : values) {
        // info(v, " ", uint256_t(v) < value_as_uint);
        if (uint256_t(v) < value_as_uint) {
            return true;
        }
    }
    return false;
}

// handle synthetic membership assertions
std::tuple<std::vector<nullifier_leaf>, std::vector<std::vector<fr>>, std::vector<uint32_t>>
NullifierMemoryTreeTestingHarness::circuit_prep_batch_insert(std::vector<fr> const& values)
{
    // Start insertion index
    fr start_insertion_index = this->size();

    // Low nullifiers
    std::vector<nullifier_leaf> low_nullifiers;
    std::vector<nullifier_leaf> pending_insertion_tree;

    // Low nullifier sibling paths
    std::vector<std::vector<fr>> sibling_paths;

    // Low nullifier indexes
    std::vector<uint32_t> low_nullifier_indexes;

    // Keep track of the currently touched nodes while updating
    std::map<size_t, std::vector<fr>> touched_nodes;

    // Keep track of 0 values
    std::vector<fr> empty_sp(depth_, 0);
    nullifier_leaf empty_leaf = { 0, 0, 0 };
    uint32_t empty_index = 0;

    // Find the leaf with the value closest and less than `value` for each value
    for (size_t i = 0; i < values.size(); ++i) {
        auto new_value = values[i];
        auto insertion_index = start_insertion_index + i;

        size_t current;
        bool is_already_present;
        std::tie(current, is_already_present) = find_closest_leaf(leaves_, new_value);

        // If the inserted value is 0, then we ignore and provide a dummy low nullifier
        if (new_value == 0) {
            sibling_paths.push_back(empty_sp);
            low_nullifier_indexes.push_back(empty_index);
            low_nullifiers.push_back(empty_leaf);
            continue;
        }

        // If the low_nullifier node has been touched this sub tree insertion, we provide a dummy sibling path
        // It will be up to the circuit to check if the included node is valid vs the other nodes that have been
        // inserted before it If it has not been touched, we provide a sibling path then update the nodes pointers
        auto prev_nodes = touched_nodes.find(current);

        bool has_less_than = false;
        if (prev_nodes != touched_nodes.end()) {
            has_less_than = check_has_less_than(prev_nodes->second, new_value);
        }
        // If there is a lower value in the tree, we need to check the current low nullifiers for one that can be used
        if (has_less_than) {
            for (size_t j = 0; j < pending_insertion_tree.size(); ++j) {
                // Skip checking empty values
                if (pending_insertion_tree[j].value == 0) {
                    continue;
                }

                if (pending_insertion_tree[j].value < new_value &&
                    (pending_insertion_tree[j].nextValue > new_value || pending_insertion_tree[j].nextValue == 0)) {
                    // Add a new pending low nullifier for this value
                    nullifier_leaf new_leaf = { .value = new_value,
                                                .nextIndex = pending_insertion_tree[j].nextIndex,
                                                .nextValue = pending_insertion_tree[j].nextValue };
                    pending_insertion_tree.push_back(new_leaf);

                    // Update the pending low nullifier to point at the new value
                    pending_insertion_tree[j].nextIndex = insertion_index;
                    pending_insertion_tree[j].nextValue = new_value;

                    break;
                }
            }

            // add empty low nullifier
            sibling_paths.push_back(empty_sp);
            low_nullifier_indexes.push_back(empty_index);
            low_nullifiers.push_back(empty_leaf);
        } else {
            // Update the touched mapping
            if (prev_nodes == touched_nodes.end()) {
                std::vector<fr> new_touched_values = { new_value };
                touched_nodes[current] = new_touched_values;
            } else {
                prev_nodes->second.push_back(new_value);
            }

            nullifier_leaf low_nullifier = leaves_[current].unwrap();
            std::vector<fr> sibling_path = this->get_sibling_path(current);

            sibling_paths.push_back(sibling_path);
            low_nullifier_indexes.push_back(static_cast<uint32_t>(current));
            low_nullifiers.push_back(low_nullifier);

            // Update the current low nullifier
            nullifier_leaf new_leaf = { .value = low_nullifier.value,
                                        .nextIndex = insertion_index,
                                        .nextValue = new_value };

            // Update the old leaf in the tree
            // update old value in tree
            update_element_in_place(current, new_leaf);
        }
    }

    // Return tuple of low nullifiers and sibling paths
    return std::make_tuple(low_nullifiers, sibling_paths, low_nullifier_indexes);
}

void NullifierMemoryTreeTestingHarness::update_element_in_place(size_t index, nullifier_leaf leaf)
{
    // Find the leaf with the value closest and less than `value`
    this->leaves_[index].set(leaf);
    update_element(index, leaf.hash());
}

std::pair<nullifier_leaf, size_t> NullifierMemoryTreeTestingHarness::find_lower(fr const& value)
{
    size_t current;
    bool is_already_present;
    std::tie(current, is_already_present) = find_closest_leaf(leaves_, value);

    // TODO: handle is already present case
    if (!is_already_present) {
        return std::make_pair(leaves_[current].unwrap(), current);
    } else {
        return std::make_pair(leaves_[current].unwrap(), current);
    }
}