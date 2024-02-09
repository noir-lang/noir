#include "barretenberg/crypto/merkle_tree/indexed_tree/indexed_tree.hpp"
#include "barretenberg/crypto/merkle_tree/array_store.hpp"
#include "barretenberg/crypto/merkle_tree/hash.hpp"
#include "barretenberg/crypto/merkle_tree/indexed_tree/leaves_cache.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include <benchmark/benchmark.h>

using namespace benchmark;
using namespace bb::crypto::merkle_tree;

using Poseidon2 = IndexedTree<ArrayStore, LeavesCache, Poseidon2HashPolicy>;
using Pedersen = IndexedTree<ArrayStore, LeavesCache, PedersenHashPolicy>;

const size_t TREE_DEPTH = 32;
const size_t MAX_BATCH_SIZE = 128;

namespace {
auto& random_engine = bb::numeric::get_randomness();
} // namespace

template <typename TreeType>
void perform_batch_insert(TreeType& tree, const std::vector<fr>& values, bool single_threaded)
{
    tree.add_or_update_values(values, single_threaded);
}

template <typename TreeType> void multi_thread_indexed_tree_bench(State& state) noexcept
{
    const size_t batch_size = size_t(state.range(0));
    const size_t depth = TREE_DEPTH;

    ArrayStore store(depth, 1024 * 1024);
    TreeType tree = TreeType(store, depth, batch_size);

    for (auto _ : state) {
        state.PauseTiming();
        std::vector<fr> values(batch_size);
        for (size_t i = 0; i < batch_size; ++i) {
            values[i] = fr(random_engine.get_random_uint256());
        }
        state.ResumeTiming();
        perform_batch_insert(tree, values, false);
    }
}

template <typename TreeType> void single_thread_indexed_tree_bench(State& state) noexcept
{
    const size_t batch_size = size_t(state.range(0));
    const size_t depth = TREE_DEPTH;

    ArrayStore store(depth, 1024 * 1024);
    TreeType tree = TreeType(store, depth, batch_size);

    for (auto _ : state) {
        state.PauseTiming();
        std::vector<fr> values(batch_size);
        for (size_t i = 0; i < batch_size; ++i) {
            values[i] = fr(random_engine.get_random_uint256());
        }
        state.ResumeTiming();
        perform_batch_insert(tree, values, true);
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