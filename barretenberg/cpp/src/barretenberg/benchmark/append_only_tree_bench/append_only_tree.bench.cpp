#include "barretenberg/crypto/merkle_tree/append_only_tree/append_only_tree.hpp"
#include "barretenberg/common/thread_pool.hpp"
#include "barretenberg/crypto/merkle_tree/fixtures.hpp"
#include "barretenberg/crypto/merkle_tree/hash.hpp"
#include "barretenberg/crypto/merkle_tree/indexed_tree/indexed_leaf.hpp"
#include "barretenberg/crypto/merkle_tree/indexed_tree/indexed_tree.hpp"
#include "barretenberg/crypto/merkle_tree/lmdb_store/lmdb_store.hpp"
#include "barretenberg/crypto/merkle_tree/node_store/array_store.hpp"
#include "barretenberg/crypto/merkle_tree/node_store/cached_tree_store.hpp"
#include "barretenberg/crypto/merkle_tree/response.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include <benchmark/benchmark.h>
#include <cstdint>
#include <filesystem>

using namespace benchmark;
using namespace bb::crypto::merkle_tree;

namespace {
using StoreType = CachedTreeStore<LMDBStore, fr>;

using Pedersen = AppendOnlyTree<StoreType, PedersenHashPolicy>;
using Poseidon2 = AppendOnlyTree<StoreType, Poseidon2HashPolicy>;

const size_t TREE_DEPTH = 32;
const size_t MAX_BATCH_SIZE = 128;

template <typename TreeType> void perform_batch_insert(TreeType& tree, const std::vector<fr>& values)
{
    Signal signal(1);
    auto completion = [&](const TypedResponse<AddDataResponse>&) -> void { signal.signal_level(0); };

    tree.add_values(values, completion);
    signal.wait_for_level(0);
}

template <typename TreeType> void commit_tree(TreeType& tree)
{
    Signal signal(1);
    auto completion = [&]() -> void { signal.signal_level(0); };
    tree.commit(completion);
    signal.wait_for_level(0);
}

template <typename TreeType> void append_only_tree_bench(State& state) noexcept
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
    TreeType tree = TreeType(store, workers);

    for (auto _ : state) {
        state.PauseTiming();
        std::vector<fr> values(batch_size);
        for (size_t i = 0; i < batch_size; ++i) {
            values[i] = fr(random_engine.get_random_uint256());
        }
        state.ResumeTiming();
        perform_batch_insert(tree, values);
    }

    std::filesystem::remove_all(directory);
}
BENCHMARK(append_only_tree_bench<Pedersen>)
    ->Unit(benchmark::kMillisecond)
    ->RangeMultiplier(2)
    ->Range(2, MAX_BATCH_SIZE)
    ->Iterations(100);
BENCHMARK(append_only_tree_bench<Poseidon2>)
    ->Unit(benchmark::kMillisecond)
    ->RangeMultiplier(2)
    ->Range(2, MAX_BATCH_SIZE)
    ->Iterations(1000);

} // namespace

BENCHMARK_MAIN();