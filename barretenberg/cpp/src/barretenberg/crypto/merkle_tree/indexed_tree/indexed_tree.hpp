#pragma once
#include "../../../common/thread.hpp"
#include "../append_only_tree/append_only_tree.hpp"
#include "../hash.hpp"
#include "../hash_path.hpp"
#include "indexed_leaf.hpp"

namespace bb::crypto::merkle_tree {

using index_t = uint256_t;

/**
 * @brief Used in parallel insertions in the the IndexedTree. Workers signal to other following workes as they move up
 * the level of the tree.
 *
 */
class LevelSignal {
  public:
    LevelSignal(size_t initial_level)
        : signal_(initial_level){};
    ~LevelSignal() = default;
    LevelSignal(const LevelSignal& other)
        : signal_(other.signal_.load())
    {}
    LevelSignal(const LevelSignal&& other) noexcept
        : signal_(other.signal_.load())
    {}

    /**
     * @brief Causes the thread to wait until the required level has been signalled
     * @param level The required level
     *
     */
    void wait_for_level(size_t level)
    {
        size_t current_level = signal_.load();
        while (current_level > level) {
            signal_.wait(current_level);
            current_level = signal_.load();
        }
    }

    /**
     * @brief Signals that the given level has been passed
     * @param level The level to be signalled
     *
     */
    void signal_level(size_t level)
    {
        signal_.store(level);
        signal_.notify_all();
    }

  private:
    std::atomic<size_t> signal_;
};

/**
 * @brief Implements a parallelised batch insertion indexed tree
 * Accepts template argument of the type of store backing the tree, the type of store containing the leaves and the
 * hashing policy
 *
 */
template <typename Store, typename LeavesStore, typename HashingPolicy>
class IndexedTree : public AppendOnlyTree<Store, HashingPolicy> {
  public:
    IndexedTree(Store& store, size_t depth, size_t initial_size = 1, uint8_t tree_id = 0);
    IndexedTree(IndexedTree const& other) = delete;
    IndexedTree(IndexedTree&& other) = delete;
    ~IndexedTree();

    /**
     * @brief Adds or updates a single values in the tree (updates not currently supported)
     * @param value The value to be added or updated
     * @returns The 'previous' hash paths of all updated values
     */
    fr_hash_path add_or_update_value(const fr& value);

    /**
     * @brief Adds or updates the given set of values in the tree (updates not currently supported)
     * @param values The values to be added or updated
     * @param no_multithreading Performs single threaded insertion, just used whilst prototyping and benchmarking
     * @returns The 'previous' hash paths of all updated values
     */
    std::vector<fr_hash_path> add_or_update_values(const std::vector<fr>& values, bool no_multithreading = false);

    /**
     * @brief Adds or updates a single value without returning the previous hash path
     * @param value The value to be added or updated
     * @returns The new root of the tree
     */
    fr add_value(const fr& value) override;

    /**
     * @brief Adds or updates the given set of values without returning the previous hash paths
     * @param values The values to be added or updated
     * @returns The new root of the tree
     */
    fr add_values(const std::vector<fr>& values) override;

    indexed_leaf get_leaf(const index_t& index);

    using AppendOnlyTree<Store, HashingPolicy>::get_hash_path;
    using AppendOnlyTree<Store, HashingPolicy>::root;
    using AppendOnlyTree<Store, HashingPolicy>::depth;

  private:
    fr update_leaf_and_hash_to_root(const index_t& index, const indexed_leaf& leaf);
    fr update_leaf_and_hash_to_root(const index_t& index,
                                    const indexed_leaf& leaf,
                                    LevelSignal& leader,
                                    LevelSignal& follower,
                                    fr_hash_path& previous_hash_path);
    fr append_subtree(const index_t& start_index);

    using AppendOnlyTree<Store, HashingPolicy>::get_element_or_zero;
    using AppendOnlyTree<Store, HashingPolicy>::write_node;
    using AppendOnlyTree<Store, HashingPolicy>::read_node;

  private:
    using AppendOnlyTree<Store, HashingPolicy>::store_;
    using AppendOnlyTree<Store, HashingPolicy>::zero_hashes_;
    using AppendOnlyTree<Store, HashingPolicy>::depth_;
    using AppendOnlyTree<Store, HashingPolicy>::tree_id_;
    using AppendOnlyTree<Store, HashingPolicy>::root_;
    LeavesStore leaves_;
};

template <typename Store, typename LeavesStore, typename HashingPolicy>
IndexedTree<Store, LeavesStore, HashingPolicy>::IndexedTree(Store& store,
                                                            size_t depth,
                                                            size_t initial_size,
                                                            uint8_t tree_id)
    : AppendOnlyTree<Store, HashingPolicy>(store, depth, tree_id)
{
    ASSERT(initial_size > 0);
    zero_hashes_.resize(depth + 1);

    // Create the zero hashes for the tree
    indexed_leaf zero_leaf{ 0, 0, 0 };
    auto current = HashingPolicy::hash(zero_leaf.get_hash_inputs());
    for (size_t i = depth; i > 0; --i) {
        zero_hashes_[i] = current;
        current = HashingPolicy::hash_pair(current, current);
    }
    zero_hashes_[0] = current;
    // Inserts the initial set of leaves as a chain in incrementing value order
    for (size_t i = 0; i < initial_size; ++i) {
        // Insert the zero leaf to the `leaves` and also to the tree at index 0.
        indexed_leaf initial_leaf = indexed_leaf{ .value = i, .nextIndex = i + 1, .nextValue = i + 1 };
        leaves_.append_leaf(initial_leaf);
    }

    // Points the last leaf back to the first
    leaves_.set_at_index(
        initial_size - 1,
        indexed_leaf{ .value = leaves_.get_leaf(initial_size - 1).value, .nextIndex = 0, .nextValue = 0 },
        false);
    append_subtree(0);
}

template <typename Store, typename LeavesStore, typename HashingPolicy>
IndexedTree<Store, LeavesStore, HashingPolicy>::~IndexedTree()
{}

template <typename Store, typename LeavesStore, typename HashingPolicy>
indexed_leaf IndexedTree<Store, LeavesStore, HashingPolicy>::get_leaf(const index_t& index)
{
    return leaves_.get_leaf(index);
}

template <typename Store, typename LeavesStore, typename HashingPolicy>
fr IndexedTree<Store, LeavesStore, HashingPolicy>::add_value(const fr& value)
{
    return add_values(std::vector<fr>{ value });
}

template <typename Store, typename LeavesStore, typename HashingPolicy>
fr IndexedTree<Store, LeavesStore, HashingPolicy>::add_values(const std::vector<fr>& values)
{
    add_or_update_values(values);
    return root();
}

template <typename Store, typename LeavesStore, typename HashingPolicy>
fr_hash_path IndexedTree<Store, LeavesStore, HashingPolicy>::add_or_update_value(const fr& value)
{
    return add_or_update_values(std::vector<fr>{ value })[0];
}

template <typename Store, typename LeavesStore, typename HashingPolicy>
std::vector<fr_hash_path> IndexedTree<Store, LeavesStore, HashingPolicy>::add_or_update_values(
    const std::vector<fr>& values, bool no_multithreading)
{
    // The first thing we do is sort the values into descending order but maintain knowledge of their orignal order
    struct {
        bool operator()(const std::pair<fr, size_t>& a, const std::pair<fr, size_t>& b) const
        {
            return uint256_t(a.first) > uint256_t(b.first);
        }
    } comp;
    std::vector<std::pair<fr, size_t>> values_sorted(values.size());
    for (size_t i = 0; i < values.size(); ++i) {
        values_sorted[i] = std::make_pair(values[i], i);
    }
    std::sort(values_sorted.begin(), values_sorted.end(), comp);

    // Now that we have the sorted values we need to identify the leaves that need updating.
    // This is performed sequentially and is stored in this 'leaf_insertion' struct
    struct leaf_insertion {
        index_t low_leaf_index;
        indexed_leaf low_leaf;
    };

    std::vector<leaf_insertion> insertions(values.size());
    index_t old_size = leaves_.get_size();

    for (size_t i = 0; i < values_sorted.size(); ++i) {
        fr value = values_sorted[i].first;
        index_t index_of_new_leaf = index_t(values_sorted[i].second) + old_size;

        // This gives us the leaf that need updating
        index_t current;
        bool is_already_present;
        std::tie(is_already_present, current) = leaves_.find_low_value(values_sorted[i].first);
        indexed_leaf current_leaf = leaves_.get_leaf(current);

        indexed_leaf new_leaf =
            indexed_leaf{ .value = value, .nextIndex = current_leaf.nextIndex, .nextValue = current_leaf.nextValue };

        // We only handle new values being added. We don't yet handle values being updated
        if (!is_already_present) {
            // Update the current leaf to point it to the new leaf
            current_leaf.nextIndex = index_of_new_leaf;
            current_leaf.nextValue = value;

            leaves_.set_at_index(current, current_leaf, false);
            leaves_.set_at_index(index_of_new_leaf, new_leaf, true);
        }

        // Capture the index and value of the updated 'low' leaf
        leaf_insertion& insertion = insertions[i];
        insertion.low_leaf_index = current;
        insertion.low_leaf = indexed_leaf{ .value = current_leaf.value,
                                           .nextIndex = current_leaf.nextIndex,
                                           .nextValue = current_leaf.nextValue };
    }

    // We now kick off multiple workers to perform the low leaf updates
    // We create set of signals to coordinate the workers as the move up the tree
    std::vector<fr_hash_path> paths(insertions.size());
    std::vector<LevelSignal> signals;
    // The first signal is set to 0. This ensure the first worker up the tree is not impeded
    signals.emplace_back(size_t(0));
    // Workers will follow their leaders up the tree, being trigger by the signal in front of them
    for (size_t i = 0; i < insertions.size(); ++i) {
        signals.emplace_back(size_t(1 + depth_));
    }

    if (no_multithreading) {
        // Execute the jobs in series
        for (size_t i = 0; i < insertions.size(); ++i) {
            leaf_insertion& insertion = insertions[i];
            update_leaf_and_hash_to_root(
                insertion.low_leaf_index, insertion.low_leaf, signals[i], signals[i + 1], paths[i]);
        }
    } else {
        // Execute the jobs in parallel
        parallel_for(insertions.size(), [&](size_t i) {
            leaf_insertion& insertion = insertions[i];
            update_leaf_and_hash_to_root(
                insertion.low_leaf_index, insertion.low_leaf, signals[i], signals[i + 1], paths[i]);
        });
    }

    // Now that we have updated all of the low leaves, we insert the new leaves as a subtree at the end
    root_ = append_subtree(old_size);

    return paths;
}

template <typename Store, typename LeavesStore, typename HashingPolicy>
fr IndexedTree<Store, LeavesStore, HashingPolicy>::update_leaf_and_hash_to_root(const index_t& leaf_index,
                                                                                const indexed_leaf& leaf)
{
    LevelSignal leader(0);
    LevelSignal follower(0);
    fr_hash_path hash_path;
    return update_leaf_and_hash_to_root(leaf_index, leaf, leader, follower, hash_path);
}

template <typename Store, typename LeavesStore, typename HashingPolicy>
fr IndexedTree<Store, LeavesStore, HashingPolicy>::update_leaf_and_hash_to_root(const index_t& leaf_index,
                                                                                const indexed_leaf& leaf,
                                                                                LevelSignal& leader,
                                                                                LevelSignal& follower,
                                                                                fr_hash_path& previous_hash_path)
{
    // We are a worker at a specific leaf index.
    // We are going to move up the tree and at each node/level:
    // 1. Wait for the level above to become 'signalled' as clear for us to write into
    // 2. Read the node and it's sibling
    // 3. Write the new node value
    index_t index = leaf_index;
    size_t level = depth_;
    fr new_hash = HashingPolicy::hash(leaf.get_hash_inputs());

    // Wait until we see that our leader has cleared 'depth_ - 1' (i.e. the level above the leaves that we are about to
    // write into) this ensures that our leader is not still reading the leaves
    size_t leader_level = depth_ - 1;
    leader.wait_for_level(leader_level);

    // Extract the value of the leaf node and it's sibling
    bool is_right = bool(index & 0x01);
    // extract the current leaf hash values for the previous hash path
    fr current_right_value = get_element_or_zero(level, index + (is_right ? 0 : 1));
    fr current_left_value = get_element_or_zero(level, is_right ? (index - 1) : index);
    previous_hash_path.push_back(std::make_pair(current_left_value, current_right_value));

    // Write the new leaf hash in place
    write_node(level, index, new_hash);
    // Signal that this level has been written
    follower.signal_level(level);

    while (level > 0) {
        if (level > 1) {
            // Level is > 1. Therefore we need to wait for our leader to have written to the level above meaning we can
            // read from it
            size_t level_to_read = level - 1;
            leader_level = level_to_read;

            leader.wait_for_level(leader_level);

            // Now read the node and it's sibling
            index_t index_of_node_above = index >> 1;
            bool node_above_is_right = bool(index_of_node_above & 0x01);
            fr above_right_value =
                get_element_or_zero(level_to_read, index_of_node_above + (node_above_is_right ? 0 : 1));
            fr above_left_value = get_element_or_zero(
                level_to_read, node_above_is_right ? (index_of_node_above - 1) : index_of_node_above);
            previous_hash_path.push_back(std::make_pair(above_left_value, above_right_value));
        }

        // Now that we have extracted the hash path from the row above
        // we can compute the new hash at that level and write it
        is_right = bool(index & 0x01);
        fr new_right_value = is_right ? new_hash : get_element_or_zero(level, index + 1);
        fr new_left_value = is_right ? get_element_or_zero(level, index - 1) : new_hash;
        new_hash = HashingPolicy::hash_pair(new_left_value, new_right_value);
        index >>= 1;
        --level;
        if (level > 0) {
            // Before we write we need to ensure that our leader has already written to the row above it
            // otherwise it could still be reading from this level
            leader_level = level - 1;
            leader.wait_for_level(leader_level);
        }

        // Write this node and signal that it is done
        write_node(level, index, new_hash);
        follower.signal_level(level);
    }
    return new_hash;
}

template <typename Store, typename LeavesStore, typename HashingPolicy>
fr IndexedTree<Store, LeavesStore, HashingPolicy>::append_subtree(const index_t& start_index)
{
    index_t index = start_index;
    size_t number_to_insert = size_t(index_t(leaves_.get_size()) - index);
    std::vector<fr> hashes_to_append = std::vector<fr>(number_to_insert);

    for (size_t i = 0; i < number_to_insert; ++i) {
        index_t index_to_insert = index + i;
        hashes_to_append[i] = HashingPolicy::hash(leaves_.get_leaf(size_t(index_to_insert)).get_hash_inputs());
    }

    return AppendOnlyTree<Store, HashingPolicy>::add_values(hashes_to_append);
}

} // namespace bb::crypto::merkle_tree