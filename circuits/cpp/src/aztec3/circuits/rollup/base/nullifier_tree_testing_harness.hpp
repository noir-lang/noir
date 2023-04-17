
#pragma once
#include "barretenberg/stdlib/merkle_tree/hash.hpp"
#include "barretenberg/stdlib/merkle_tree/memory_tree.hpp"
#include "barretenberg/stdlib/merkle_tree/nullifier_tree/nullifier_leaf.hpp"
#include "barretenberg/stdlib/merkle_tree/nullifier_tree/nullifier_memory_tree.hpp"
#include <tuple>

/**
 * A version of the nullifier memory tree with extra methods specific to testing our rollup circuits.
 */
class NullifierMemoryTreeTestingHarness : public proof_system::plonk::stdlib::merkle_tree::NullifierMemoryTree {
    using nullifier_leaf = proof_system::plonk::stdlib::merkle_tree::nullifier_leaf;

  public:
    NullifierMemoryTreeTestingHarness(size_t depth);

    using MemoryTree::get_hash_path;
    using MemoryTree::root;
    using MemoryTree::update_element;

    using NullifierMemoryTree::update_element;

    using NullifierMemoryTree::get_hashes;
    using NullifierMemoryTree::get_leaf;
    using NullifierMemoryTree::get_leaves;

    // Get the value immediately lower than the given value
    std::pair<nullifier_leaf, size_t> find_lower(fr const& value);

    // Utilities to inspect tree
    fr size() const { return total_size_; }
    fr depth() const { return depth_; }

    // Get all of the sibling paths and low nullifier values required to craft an non membership / inclusion proofs
    std::tuple<std::vector<nullifier_leaf>, std::vector<std::vector<fr>>, std::vector<uint32_t>>
    circuit_prep_batch_insert(std::vector<fr> const& values, std::vector<fr> const& insertion_locations);

  protected:
    using MemoryTree::depth_;
    using MemoryTree::hashes_;
    using MemoryTree::root_;
    using MemoryTree::total_size_;
    using NullifierMemoryTree::leaves_;
};
