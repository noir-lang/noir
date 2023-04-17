#include "nullifier_tree_testing_harness.hpp"
#include <barretenberg/stdlib/merkle_tree/nullifier_tree/nullifier_memory_tree.hpp>
#include <barretenberg/stdlib/merkle_tree/nullifier_tree/nullifier_leaf.hpp>
#include <tuple>

using NullifierMemoryTree = proof_system::plonk::stdlib::merkle_tree::NullifierMemoryTree;
using nullifier_leaf = proof_system::plonk::stdlib::merkle_tree::nullifier_leaf;

NullifierMemoryTreeTestingHarness::NullifierMemoryTreeTestingHarness(size_t depth)
    : NullifierMemoryTree(depth)
{}

// handle synthetic membership assertions
std::tuple<std::vector<nullifier_leaf>, std::vector<std::vector<fr>>, std::vector<uint32_t>>
NullifierMemoryTreeTestingHarness::circuit_prep_batch_insert(std::vector<fr> const& values,
                                                             std::vector<fr> const& insertion_locations)
{
    // Low nullifiers
    std::vector<nullifier_leaf> low_nullifiers;

    // Low nullifier sibling paths
    std::vector<std::vector<fr>> sibling_paths;

    // Low nullifier indexes
    std::vector<uint32_t> low_nullifier_indexes;

    // Keep track of the currently touched nodes while updating
    std::set<size_t> touched_nodes;

    // Find the leaf with the value closest and less than `value` for each value
    for (size_t i = 0; i < values.size(); ++i) {
        auto value = values[i];
        auto insertion_index = uint256_t(insertion_locations[i]);

        size_t current;
        bool is_already_present;

        std::tie(current, is_already_present) = find_closest_leaf(leaves_, value);

        // If the low_nullifier node has been touched this sub tree insertion, we provide a dummy sibling path
        // It will be up to the circuit to check if the included node is valid vs the other nodes that have been
        // inserted before it If it has not been touched, we provide a sibling path then update the nodes pointers
        if (touched_nodes.contains(current)) {
            std::vector<fr> sp(depth_, 0);
            auto empty_leaf = nullifier_leaf{ 0, 0, 0 };

            // empty low nullifier
            sibling_paths.push_back(sp);
            low_nullifier_indexes.push_back(0);
            low_nullifiers.push_back(empty_leaf);
        } else {
            touched_nodes.insert(current);

            nullifier_leaf low_nullifier = leaves_[current];
            std::vector<fr> sibling_path = this->get_sibling_path(current);

            sibling_paths.push_back(sibling_path);
            low_nullifier_indexes.push_back(static_cast<uint32_t>(current));
            low_nullifiers.push_back(low_nullifier);

            // Update the current low nullifier
            nullifier_leaf new_leaf = { .value = low_nullifier.value,
                                        .nextIndex = insertion_index,
                                        .nextValue = value };

            // Update the old leaf in the tree
            update_element(current, new_leaf.hash());
        }
    }

    // Return tuple of low nullifiers and sibling paths
    return std::make_tuple(low_nullifiers, sibling_paths, low_nullifier_indexes);
}

std::pair<nullifier_leaf, size_t> NullifierMemoryTreeTestingHarness::find_lower(fr const& value)
{
    size_t current;
    bool is_already_present;
    std::tie(current, is_already_present) = find_closest_leaf(leaves_, value);

    // TODO: handle is already present case
    if (!is_already_present) {
        return std::make_pair(leaves_[current], current);
    } else {
        return std::make_pair(leaves_[current], current);
    }
}