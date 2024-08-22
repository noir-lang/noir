#pragma once
#include "../append_only_tree/append_only_tree.hpp"
#include "../hash.hpp"
#include "../hash_path.hpp"
#include "../signal.hpp"
#include "barretenberg/common/assert.hpp"
#include "barretenberg/common/thread_pool.hpp"
#include "barretenberg/crypto/merkle_tree/response.hpp"
#include "barretenberg/crypto/merkle_tree/types.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "indexed_leaf.hpp"
#include <algorithm>
#include <atomic>
#include <cstdint>
#include <exception>
#include <functional>
#include <iostream>
#include <memory>
#include <optional>
#include <stdexcept>
#include <unordered_set>
#include <vector>

namespace bb::crypto::merkle_tree {

/**
 * @brief Implements a parallelised batch insertion indexed tree
 * Accepts template argument of the type of store backing the tree, the type of store containing the leaves and the
 * hashing policy
 * All public methods are asynchronous unless marked otherwise
 */
template <typename Store, typename HashingPolicy> class IndexedTree : public AppendOnlyTree<Store, HashingPolicy> {
  public:
    using StoreType = Store;

    // The public methods accept these function types as asynchronous callbacks
    using LeafValueType = typename Store::LeafType;
    using IndexedLeafValueType = typename Store::IndexedLeafValueType;
    using AddCompletionCallback = std::function<void(const TypedResponse<AddIndexedDataResponse<LeafValueType>>&)>;
    using LeafCallback = std::function<void(const TypedResponse<GetIndexedLeafResponse<LeafValueType>>&)>;
    using FindLowLeafCallback = std::function<void(const TypedResponse<std::pair<bool, index_t>>&)>;

    IndexedTree(Store& store, ThreadPool& workers, index_t initial_size);
    IndexedTree(IndexedTree const& other) = delete;
    IndexedTree(IndexedTree&& other) = delete;
    ~IndexedTree() = default;
    IndexedTree& operator=(const IndexedTree& other) = delete;
    IndexedTree& operator=(IndexedTree&& other) = delete;

    /**
     * @brief Adds or updates a single values in the tree (updates not currently supported)
     */
    void add_or_update_value(const LeafValueType& value, const AddCompletionCallback& completion);

    /**
     * @brief Adds or updates the given set of values in the tree (updates not currently supported)
     * @param values The values to be added or updated
     * @param completion The callback to be triggered once the values have been added
     */
    void add_or_update_values(const std::vector<LeafValueType>& values, const AddCompletionCallback& completion);

    /**
     * @brief Adds or updates the given set of values in the tree (updates not currently supported)
     * @param values The values to be added or updated
     * @param subtree_depth The height of the subtree to be inserted.
     * @param completion The callback to be triggered once the values have been added
     */
    void add_or_update_values(const std::vector<LeafValueType>& values,
                              uint32_t subtree_depth,
                              const AddCompletionCallback& completion);

    void get_leaf(const index_t& index, bool includeUncommitted, const LeafCallback& completion) const;

    /**
     * @brief Find the index of the provided leaf value if it exists
     */
    void find_leaf_index(const LeafValueType& leaf,
                         bool includeUncommitted,
                         const AppendOnlyTree<Store, HashingPolicy>::FindLeafCallback& on_completion) const;

    /**
     * @brief Find the index of the provided leaf value if it exists, only considers indexed beyond the value provided
     */
    void find_leaf_index_from(const LeafValueType& leaf,
                              index_t start_index,
                              bool includeUncommitted,
                              const AppendOnlyTree<Store, HashingPolicy>::FindLeafCallback& on_completion) const;

    /**
     * @brief Find the leaf with the value immediately lower then the value provided
     */
    void find_low_leaf(const fr& leaf_key, bool includeUncommitted, const FindLowLeafCallback& on_completion) const;

    using AppendOnlyTree<Store, HashingPolicy>::get_sibling_path;

  private:
    using typename AppendOnlyTree<Store, HashingPolicy>::AppendCompletionCallback;
    using ReadTransaction = typename Store::ReadTransaction;
    using ReadTransactionPtr = typename Store::ReadTransactionPtr;

    struct Status {
        std::atomic_bool success{ true };
        std::string message;

        void set_failure(const std::string& msg)
        {
            if (success.exchange(false)) {
                message = msg;
            }
        }
    };

    struct LeafInsertion {
        index_t low_leaf_index;
        IndexedLeafValueType low_leaf, original_low_leaf;
    };

    void update_leaf_and_hash_to_root(const index_t& index,
                                      const IndexedLeafValueType& leaf,
                                      Signal& leader,
                                      Signal& follower,
                                      fr_sibling_path& previous_sibling_path,
                                      ReadTransaction& tx);

    struct InsertionGenerationResponse {
        std::shared_ptr<std::vector<LeafInsertion>> insertions;
        std::shared_ptr<std::vector<IndexedLeafValueType>> indexed_leaves;
    };

    using InsertionGenerationCallback = std::function<void(const TypedResponse<InsertionGenerationResponse>&)>;
    void generate_insertions(const std::shared_ptr<std::vector<std::pair<LeafValueType, index_t>>>& values_to_be_sorted,
                             const InsertionGenerationCallback& completion);

    struct InsertionCompletionResponse {
        std::shared_ptr<std::vector<LowLeafWitnessData<LeafValueType>>> low_leaf_witness_data;
    };

    using InsertionCompletionCallback = std::function<void(const TypedResponse<InsertionCompletionResponse>&)>;
    void perform_insertions(size_t total_leaves,
                            std::shared_ptr<std::vector<LeafInsertion>> insertions,
                            const InsertionCompletionCallback& completion);

    struct HashGenerationResponse {
        std::shared_ptr<std::vector<fr>> hashes;
    };

    using HashGenerationCallback = std::function<void(const TypedResponse<HashGenerationResponse>&)>;
    void generate_hashes_for_appending(std::shared_ptr<std::vector<IndexedLeafValueType>> leaves_to_hash,
                                       const HashGenerationCallback& completion);

    using AppendOnlyTree<Store, HashingPolicy>::get_element_or_zero;
    using AppendOnlyTree<Store, HashingPolicy>::write_node;
    using AppendOnlyTree<Store, HashingPolicy>::read_node;

    using AppendOnlyTree<Store, HashingPolicy>::add_value;
    using AppendOnlyTree<Store, HashingPolicy>::add_values;
    using AppendOnlyTree<Store, HashingPolicy>::add_values_internal;

    using AppendOnlyTree<Store, HashingPolicy>::store_;
    using AppendOnlyTree<Store, HashingPolicy>::zero_hashes_;
    using AppendOnlyTree<Store, HashingPolicy>::depth_;
    using AppendOnlyTree<Store, HashingPolicy>::name_;
    using AppendOnlyTree<Store, HashingPolicy>::workers_;
    using AppendOnlyTree<Store, HashingPolicy>::max_size_;
};

template <typename Store, typename HashingPolicy>
IndexedTree<Store, HashingPolicy>::IndexedTree(Store& store, ThreadPool& workers, index_t initial_size)
    : AppendOnlyTree<Store, HashingPolicy>(store, workers)
{
    if (initial_size < 2) {
        throw std::runtime_error("Indexed trees must have initial size > 1");
    }
    zero_hashes_.resize(depth_ + 1);

    // Create the zero hashes for the tree
    // Indexed_LeafType zero_leaf{ 0, 0, 0 };
    auto current = fr::zero();
    for (uint32_t i = depth_; i > 0; --i) {
        zero_hashes_[i] = current;
        current = HashingPolicy::hash_pair(current, current);
    }
    zero_hashes_[0] = current;

    index_t stored_size = 0;
    bb::fr stored_root = fr::zero();
    {
        ReadTransactionPtr tx = store_.create_read_transaction();
        std::string name;
        uint32_t depth = 0;
        store_.get_full_meta(stored_size, stored_root, name, depth, *tx, false);
    }

    if (stored_size > 0) {
        return;
    }

    std::vector<IndexedLeafValueType> appended_leaves;
    std::vector<bb::fr> appended_hashes;
    // Inserts the initial set of leaves as a chain in incrementing value order
    for (uint32_t i = 0; i < initial_size; ++i) {
        // Insert the zero leaf to the `leaves` and also to the tree at index 0.
        bool last = i == (initial_size - 1);
        IndexedLeafValueType initial_leaf =
            IndexedLeafValueType(LeafValueType::padding(i), last ? 0 : i + 1, last ? 0 : i + 1);
        appended_leaves.push_back(initial_leaf);
        appended_hashes.push_back(HashingPolicy::hash(initial_leaf.get_hash_inputs()));
        store_.set_at_index(i, initial_leaf, true);
    }

    TypedResponse<AddDataResponse> result;
    Signal signal(1);
    AppendCompletionCallback completion = [&](const TypedResponse<AddDataResponse>& _result) -> void {
        result = _result;
        signal.signal_level(0);
    };
    AppendOnlyTree<Store, HashingPolicy>::add_values_internal(appended_hashes, completion, false);
    signal.wait_for_level(0);
    if (!result.success) {
        throw std::runtime_error("Failed to initialise tree: " + result.message);
    }
    store_.commit();
}

template <typename Store, typename HashingPolicy>
void IndexedTree<Store, HashingPolicy>::get_leaf(const index_t& index,
                                                 bool includeUncommitted,
                                                 const LeafCallback& completion) const
{
    auto job = [=, this]() {
        execute_and_report<GetIndexedLeafResponse<LeafValueType>>(
            [=, this](TypedResponse<GetIndexedLeafResponse<LeafValueType>>& response) {
                ReadTransactionPtr tx = store_.create_read_transaction();
                response.inner.indexed_leaf = store_.get_leaf(index, *tx, includeUncommitted);
            },
            completion);
    };
    workers_.enqueue(job);
}

template <typename Store, typename HashingPolicy>
void IndexedTree<Store, HashingPolicy>::find_leaf_index(
    const LeafValueType& leaf,
    bool includeUncommitted,
    const AppendOnlyTree<Store, HashingPolicy>::FindLeafCallback& on_completion) const
{
    find_leaf_index_from(leaf, 0, includeUncommitted, on_completion);
}

template <typename Store, typename HashingPolicy>
void IndexedTree<Store, HashingPolicy>::find_leaf_index_from(
    const LeafValueType& leaf,
    index_t start_index,
    bool includeUncommitted,
    const AppendOnlyTree<Store, HashingPolicy>::FindLeafCallback& on_completion) const
{
    auto job = [=, this]() -> void {
        execute_and_report<FindLeafIndexResponse>(
            [=, this](TypedResponse<FindLeafIndexResponse>& response) {
                typename Store::ReadTransactionPtr tx = store_.create_read_transaction();
                std::optional<index_t> leaf_index =
                    store_.find_leaf_index_from(leaf, start_index, *tx, includeUncommitted);
                response.success = leaf_index.has_value();
                if (response.success) {
                    response.inner.leaf_index = leaf_index.value();
                }
            },
            on_completion);
    };
    workers_.enqueue(job);
}

template <typename Store, typename HashingPolicy>
void IndexedTree<Store, HashingPolicy>::find_low_leaf(const fr& leaf_key,
                                                      bool includeUncommitted,
                                                      const FindLowLeafCallback& on_completion) const
{
    auto job = [=, this]() {
        execute_and_report<std::pair<bool, index_t>>(
            [=, this](TypedResponse<std::pair<bool, index_t>>& response) {
                typename Store::ReadTransactionPtr tx = store_.create_read_transaction();
                response.inner = store_.find_low_value(leaf_key, includeUncommitted, *tx);
            },
            on_completion);
    };

    workers_.enqueue(job);
}

template <typename Store, typename HashingPolicy>
void IndexedTree<Store, HashingPolicy>::add_or_update_value(const LeafValueType& value,
                                                            const AddCompletionCallback& completion)
{
    add_or_update_values(std::vector<LeafValueType>{ value }, 1, completion);
}

template <typename Store, typename HashingPolicy>
void IndexedTree<Store, HashingPolicy>::add_or_update_values(const std::vector<LeafValueType>& values,
                                                             const AddCompletionCallback& completion)
{
    add_or_update_values(values, 0, completion);
}

template <typename Store, typename HashingPolicy>
void IndexedTree<Store, HashingPolicy>::add_or_update_values(const std::vector<LeafValueType>& values,
                                                             const uint32_t subtree_depth,
                                                             const AddCompletionCallback& completion)
{
    // We first take a copy of the leaf values and their locations within the set given to us
    std::shared_ptr<std::vector<std::pair<LeafValueType, size_t>>> values_to_be_sorted =
        std::make_shared<std::vector<std::pair<LeafValueType, size_t>>>(values.size());
    for (size_t i = 0; i < values.size(); ++i) {
        (*values_to_be_sorted)[i] = std::make_pair(values[i], i);
    }

    // This is to collect some state from the asynchronous operations we are about to perform
    struct IntermediateResults {
        // new hashes that will be appended to the tree
        std::shared_ptr<std::vector<fr>> hashes_to_append;
        // info about the low leaves that have been updated
        std::shared_ptr<std::vector<LowLeafWitnessData<LeafValueType>>> low_leaf_witness_data;
        fr_sibling_path subtree_path;
        std::atomic<uint32_t> count;
        Status status;

        // We set to 2 here as we will kick off the 2 main async operations concurrently and we need to trakc thri
        // completion
        IntermediateResults()
            : count(2)
        {
            // Default to success, set to false on error
            status.success = true;
        };
    };
    std::shared_ptr<IntermediateResults> results = std::make_shared<IntermediateResults>();

    auto on_error = [=](const std::string& message) {
        try {
            TypedResponse<AddIndexedDataResponse<LeafValueType>> response;
            response.success = false;
            response.message = message;
            completion(response);
        } catch (std::exception&) {
        }
    };

    // Thsi is the final callback triggered once the leaves have been appended to the tree
    auto final_completion = [=](const TypedResponse<AddDataResponse>& add_data_response) {
        TypedResponse<AddIndexedDataResponse<LeafValueType>> response;
        response.success = add_data_response.success;
        response.message = add_data_response.message;
        if (add_data_response.success) {
            response.inner.subtree_path = results->subtree_path;
            response.inner.sorted_leaves = values_to_be_sorted;
            response.inner.low_leaf_witness_data = results->low_leaf_witness_data;
            response.inner.add_data_result = add_data_response.inner;
        }
        // Trigger the client's provided callback
        completion(response);
    };

    auto sibling_path_completion = [=, this](const TypedResponse<GetSiblingPathResponse>& response) {
        if (!response.success) {
            results->status.set_failure(response.message);
        } else {
            results->subtree_path = response.inner.path;
            AppendOnlyTree<Store, HashingPolicy>::add_values_internal(
                (*results->hashes_to_append), final_completion, false);
        }
    };

    // This signals the completion of the appended hash generation
    // If the low leaf updates are also completed then we will append the leaves
    HashGenerationCallback hash_completion = [=, this](const TypedResponse<HashGenerationResponse>& hashes_response) {
        if (!hashes_response.success) {
            results->status.set_failure(hashes_response.message);
        } else {
            results->hashes_to_append = hashes_response.inner.hashes;
        }

        if (results->count.fetch_sub(1) == 1) {
            if (!results->status.success) {
                on_error(results->status.message);
                return;
            }
            AppendOnlyTree<Store, HashingPolicy>::get_subtree_sibling_path(
                subtree_depth, sibling_path_completion, true);
        }
    };

    // This signals the completion of the low leaf updates
    // If the append hash generation has also copleted then the hashes can be appended
    InsertionCompletionCallback insertion_completion =
        [=, this](const TypedResponse<InsertionCompletionResponse>& insertion_response) {
            if (!insertion_response.success) {
                results->status.set_failure(insertion_response.message);
            } else {
                results->low_leaf_witness_data = insertion_response.inner.low_leaf_witness_data;
            }

            if (results->count.fetch_sub(1) == 1) {
                if (!results->status.success) {
                    on_error(results->status.message);
                    return;
                }
                AppendOnlyTree<Store, HashingPolicy>::get_subtree_sibling_path(
                    subtree_depth, sibling_path_completion, true);
            }
        };

    // This signals the completion of the insertion data generation
    // Here we will enqueue both the generation of the appended hashes and the low leaf updates (insertions)
    InsertionGenerationCallback insertion_generation_completed =
        [=, this](const TypedResponse<InsertionGenerationResponse>& insertion_response) {
            if (!insertion_response.success) {
                on_error(insertion_response.message);
                return;
            }
            workers_.enqueue([=, this]() {
                generate_hashes_for_appending(insertion_response.inner.indexed_leaves, hash_completion);
            });
            perform_insertions(values.size(), insertion_response.inner.insertions, insertion_completion);
        };

    // We start by enqueueing the insertion data generation
    workers_.enqueue([=, this]() { generate_insertions(values_to_be_sorted, insertion_generation_completed); });
}

template <typename Store, typename HashingPolicy>
void IndexedTree<Store, HashingPolicy>::perform_insertions(size_t total_leaves,
                                                           std::shared_ptr<std::vector<LeafInsertion>> insertions,
                                                           const InsertionCompletionCallback& completion)
{
    auto low_leaf_witness_data = std::make_shared<std::vector<LowLeafWitnessData<LeafValueType>>>(
        total_leaves,
        LowLeafWitnessData<LeafValueType>{ IndexedLeafValueType::empty(), 0, fr_sibling_path(depth_, fr::zero()) });

    // early return, no insertions to perform
    if (insertions->size() == 0) {
        TypedResponse<InsertionCompletionResponse> response;
        response.success = true;
        response.inner.low_leaf_witness_data = low_leaf_witness_data;
        completion(response);
        return;
    }

    // We now kick off multiple workers to perform the low leaf updates
    // We create set of signals to coordinate the workers as the move up the tree
    std::shared_ptr<std::vector<Signal>> signals = std::make_shared<std::vector<Signal>>();
    std::shared_ptr<Status> status = std::make_shared<Status>();
    // The first signal is set to 0. This ensures the first worker up the tree is not impeded
    signals->emplace_back(0);
    // Workers will follow their leaders up the tree, being triggered by the signal in front of them
    for (size_t i = 0; i < insertions->size(); ++i) {
        signals->emplace_back(uint32_t(1 + depth_));
    }

    for (uint32_t i = 0; i < insertions->size(); ++i) {
        auto op = [=, this]() {
            LeafInsertion& insertion = (*insertions)[i];
            Signal& leaderSignal = (*signals)[i];
            Signal& followerSignal = (*signals)[i + 1];
            try {
                ReadTransactionPtr tx = store_.create_read_transaction();
                auto& current_witness_data = low_leaf_witness_data->at(i);
                current_witness_data.leaf = insertion.original_low_leaf;
                current_witness_data.index = insertion.low_leaf_index;
                current_witness_data.path.clear();
                update_leaf_and_hash_to_root(insertion.low_leaf_index,
                                             insertion.low_leaf,
                                             leaderSignal,
                                             followerSignal,
                                             current_witness_data.path,
                                             *tx);
            } catch (std::exception& e) {
                status->set_failure(e.what());
                // ensure that any followers are not blocked by our failure
                followerSignal.signal_level(0);
            }
            if (i == insertions->size() - 1) {
                TypedResponse<InsertionCompletionResponse> response;
                response.success = status->success;
                response.message = status->message;
                if (response.success) {
                    response.inner.low_leaf_witness_data = low_leaf_witness_data;
                }
                completion(response);
            }
        };
        workers_.enqueue(op);
    }
}

template <typename Store, typename HashingPolicy>
void IndexedTree<Store, HashingPolicy>::generate_hashes_for_appending(
    std::shared_ptr<std::vector<IndexedLeafValueType>> leaves_to_hash, const HashGenerationCallback& completion)
{
    execute_and_report<HashGenerationResponse>(
        [=](TypedResponse<HashGenerationResponse>& response) {
            response.inner.hashes = std::make_shared<std::vector<fr>>(leaves_to_hash->size(), 0);
            std::vector<IndexedLeafValueType>& leaves = *leaves_to_hash;
            for (uint32_t i = 0; i < leaves.size(); ++i) {
                if (!leaves[i].is_empty()) {
                    (*response.inner.hashes)[i] = (HashingPolicy::hash(leaves[i].get_hash_inputs()));
                }
            }
        },
        completion);
}

template <typename Store, typename HashingPolicy>
void IndexedTree<Store, HashingPolicy>::generate_insertions(
    const std::shared_ptr<std::vector<std::pair<LeafValueType, index_t>>>& values_to_be_sorted,
    const InsertionGenerationCallback& on_completion)
{
    execute_and_report<InsertionGenerationResponse>(
        [=, this](TypedResponse<InsertionGenerationResponse>& response) {
            // The first thing we do is sort the values into descending order but maintain knowledge of their
            // orignal order
            struct {
                bool operator()(std::pair<LeafValueType, index_t>& a, std::pair<LeafValueType, index_t>& b) const
                {
                    uint256_t aValue = a.first.get_key();
                    uint256_t bValue = b.first.get_key();
                    return aValue == bValue ? a.second < b.second : aValue > bValue;
                }
            } comp;
            std::sort(values_to_be_sorted->begin(), values_to_be_sorted->end(), comp);

            std::vector<std::pair<LeafValueType, index_t>>& values = *values_to_be_sorted;

            // Now that we have the sorted values we need to identify the leaves that need updating.
            // This is performed sequentially and is stored in this 'leaf_insertion' struct
            response.inner.insertions = std::make_shared<std::vector<LeafInsertion>>();
            response.inner.insertions->reserve(values.size());
            response.inner.indexed_leaves =
                std::make_shared<std::vector<IndexedLeafValueType>>(values.size(), IndexedLeafValueType::empty());
            index_t old_size = 0;
            index_t num_leaves_to_be_inserted = values.size();
            std::set<uint256_t> unique_values;
            {
                ReadTransactionPtr tx = store_.create_read_transaction();
                bb::fr old_root = fr::zero();
                store_.get_meta(old_size, old_root, *tx, true);
                // Ensure that the tree is not going to be overfilled
                index_t new_total_size = num_leaves_to_be_inserted + old_size;
                if (new_total_size > max_size_) {
                    throw std::runtime_error("Tree is full");
                }
                for (size_t i = 0; i < values.size(); ++i) {
                    std::pair<LeafValueType, size_t>& value_pair = values[i];
                    size_t index_into_appended_leaves = value_pair.second;
                    index_t index_of_new_leaf = static_cast<index_t>(index_into_appended_leaves) + old_size;
                    if (value_pair.first.is_empty()) {
                        continue;
                    }
                    fr value = value_pair.first.get_key();
                    auto it = unique_values.insert(value);
                    if (!it.second) {
                        throw std::runtime_error("Duplicate key not allowed in same batch");
                    }

                    // This gives us the leaf that need updating
                    index_t current = 0;
                    bool is_already_present = false;
                    std::tie(is_already_present, current) =
                        store_.find_low_value(value_pair.first.get_key(), true, *tx);
                    // .value() throws if the low leaf does not exist
                    IndexedLeafValueType current_leaf = store_.get_leaf(current, *tx, true).value();

                    LeafInsertion insertion = {
                        .low_leaf_index = current,
                        .low_leaf = IndexedLeafValueType::empty(),
                        .original_low_leaf = current_leaf,
                    };

                    // Capture the index and original value of the 'low' leaf

                    if (!is_already_present) {
                        // Update the current leaf to point it to the new leaf
                        IndexedLeafValueType new_leaf =
                            IndexedLeafValueType(value_pair.first, current_leaf.nextIndex, current_leaf.nextValue);

                        current_leaf.nextIndex = index_of_new_leaf;
                        current_leaf.nextValue = value;
                        store_.set_at_index(current, current_leaf, false);
                        store_.set_at_index(index_of_new_leaf, new_leaf, true);

                        // Update the set of leaves to append
                        (*response.inner.indexed_leaves)[index_into_appended_leaves] = new_leaf;
                    } else if (IndexedLeafValueType::is_updateable()) {
                        // Update the current leaf's value, don't change it's link
                        IndexedLeafValueType replacement_leaf =
                            IndexedLeafValueType(value_pair.first, current_leaf.nextIndex, current_leaf.nextValue);
                        store_.set_at_index(current, replacement_leaf, false);
                        IndexedLeafValueType empty_leaf = IndexedLeafValueType::empty();
                        // don't update the index for this empty leaf
                        store_.set_at_index(index_of_new_leaf, empty_leaf, false);
                        // The set of appended leaves already has an empty leaf in the slot at index
                        // 'index_into_appended_leaves'
                    } else {
                        throw std::runtime_error("IndexedLeafValue is not updateable");
                    }

                    // capture new low leaf
                    insertion.low_leaf =
                        IndexedLeafValueType(current_leaf.value, current_leaf.nextIndex, current_leaf.nextValue);

                    response.inner.insertions->push_back(insertion);
                }
            }
        },
        on_completion);
}

template <typename Store, typename HashingPolicy>
void IndexedTree<Store, HashingPolicy>::update_leaf_and_hash_to_root(const index_t& leaf_index,
                                                                     const IndexedLeafValueType& leaf,
                                                                     Signal& leader,
                                                                     Signal& follower,
                                                                     fr_sibling_path& previous_sibling_path,
                                                                     ReadTransaction& tx)
{
    auto get_node = [&](uint32_t level, index_t index) -> fr { return get_element_or_zero(level, index, tx, true); };
    // We are a worker at a specific leaf index.
    // We are going to move up the tree and at each node/level:
    // 1. Wait for the level above to become 'signalled' as clear for us to write into
    // 2. Read the node and it's sibling
    // 3. Write the new node value
    index_t index = leaf_index;
    uint32_t level = depth_;
    fr new_hash = HashingPolicy::hash(leaf.get_hash_inputs());

    // Wait until we see that our leader has cleared 'depth_ - 1' (i.e. the level above the leaves that we are about
    // to write into) this ensures that our leader is not still reading the leaves
    uint32_t leader_level = depth_ - 1;
    leader.wait_for_level(leader_level);

    // Extract the value of the leaf node and it's sibling
    bool is_right = static_cast<bool>(index & 0x01);

    // Write the new leaf hash in place
    write_node(level, index, new_hash);
    // Signal that this level has been written
    follower.signal_level(level);

    while (level > 0) {
        if (level > 1) {
            // Level is > 1. Therefore we need to wait for our leader to have written to the level above meaning we
            // can read from it
            leader_level = level - 1;
            leader.wait_for_level(leader_level);

            // Now read the node and it's sibling
            index_t index_of_node_above = index >> 1;
            bool node_above_is_right = static_cast<bool>(index_of_node_above & 0x01);
            fr above_sibling = get_node(level, node_above_is_right ? index_of_node_above - 1 : index_of_node_above + 1);
        }

        // Now that we have extracted the hash path from the row above
        // we can compute the new hash at that level and write it
        is_right = static_cast<bool>(index & 0x01);
        fr new_right_value = is_right ? new_hash : get_node(level, index + 1);
        fr new_left_value = is_right ? get_node(level, index - 1) : new_hash;
        previous_sibling_path.emplace_back(is_right ? new_left_value : new_right_value);
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
}

} // namespace bb::crypto::merkle_tree
