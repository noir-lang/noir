#include "barretenberg/crypto/merkle_tree/indexed_tree/indexed_tree.hpp"
#include "barretenberg/crypto/merkle_tree/fixtures.hpp"
#include "barretenberg/crypto/merkle_tree/hash.hpp"
#include "barretenberg/crypto/merkle_tree/indexed_tree/indexed_leaf.hpp"
#include "barretenberg/crypto/merkle_tree/lmdb_store/callbacks.hpp"
#include "barretenberg/crypto/merkle_tree/lmdb_store/lmdb_store.hpp"
#include "barretenberg/crypto/merkle_tree/node_store/cached_tree_store.hpp"
#include "barretenberg/crypto/merkle_tree/response.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include <benchmark/benchmark.h>
#include <filesystem>

using namespace benchmark;
using namespace bb::crypto::merkle_tree;

using StoreType = CachedTreeStore<LMDBStore, NullifierLeafValue>;

using Poseidon2 = IndexedTree<StoreType, Poseidon2HashPolicy>;
using Pedersen = IndexedTree<StoreType, PedersenHashPolicy>;

const size_t TREE_DEPTH = 40;
const size_t MAX_BATCH_SIZE = 128;

template <typename TreeType> void add_values(TreeType& tree, const std::vector<NullifierLeafValue>& values)
{
    Signal signal(1);
    auto completion = [&](const auto&) -> void { signal.signal_level(0); };

    tree.add_or_update_values(values, completion);
    signal.wait_for_level(0);
}

template <typename TreeType> void multi_thread_indexed_tree_bench(State& state) noexcept
{
    const size_t batch_size = size_t(state.range(0));
    const size_t depth = TREE_DEPTH;

    std::string directory = random_temp_directory();
    std::string name = random_string();
    std::filesystem::create_directories(directory);
    uint32_t num_threads = 16;
    LMDBEnvironment environment = LMDBEnvironment(directory, 1024 * 1024, 2, num_threads);

    LMDBStore db(environment, name, false, false, integer_key_cmp);
    StoreType store(name, depth, db);
    ThreadPool workers(num_threads);
    TreeType tree = TreeType(store, workers, batch_size);

    for (auto _ : state) {
        state.PauseTiming();
        std::vector<NullifierLeafValue> values(batch_size);
        for (size_t i = 0; i < batch_size; ++i) {
            values[i] = fr(random_engine.get_random_uint256());
        }
        state.ResumeTiming();
        add_values(tree, values);
    }
}

template <typename TreeType> void single_thread_indexed_tree_bench(State& state) noexcept
{
    const size_t batch_size = size_t(state.range(0));
    const size_t depth = TREE_DEPTH;

    std::string directory = random_temp_directory();
    std::string name = random_string();
    std::filesystem::create_directories(directory);
    uint32_t num_threads = 1;
    LMDBEnvironment environment = LMDBEnvironment(directory, 1024 * 1024, 2, num_threads);

    LMDBStore db(environment, name, false, false, integer_key_cmp);
    StoreType store(name, depth, db);
    ThreadPool workers(num_threads);
    TreeType tree = TreeType(store, workers, batch_size);

    for (auto _ : state) {
        state.PauseTiming();
        std::vector<NullifierLeafValue> values(batch_size);
        for (size_t i = 0; i < batch_size; ++i) {
            values[i] = fr(random_engine.get_random_uint256());
        }
        state.ResumeTiming();
        add_values(tree, values);
    }
}
BENCHMARK(single_thread_indexed_tree_bench<Pedersen>)
    ->Unit(benchmark::kMillisecond)
    ->RangeMultiplier(2)
    ->Range(2, MAX_BATCH_SIZE)
    ->Iterations(100);

BENCHMARK(multi_thread_indexed_tree_bench<Pedersen>)
    ->Unit(benchmark::kMillisecond)
    ->RangeMultiplier(2)
    ->Range(2, MAX_BATCH_SIZE)
    ->Iterations(100);

BENCHMARK(single_thread_indexed_tree_bench<Poseidon2>)
    ->Unit(benchmark::kMillisecond)
    ->RangeMultiplier(2)
    ->Range(2, MAX_BATCH_SIZE)
    ->Iterations(1000);
BENCHMARK(multi_thread_indexed_tree_bench<Poseidon2>)
    ->Unit(benchmark::kMillisecond)
    ->RangeMultiplier(2)
    ->Range(2, MAX_BATCH_SIZE)
    ->Iterations(1000);

BENCHMARK_MAIN();
