#include "barretenberg/crypto/merkle_tree/merkle_tree.hpp"
#include "barretenberg/crypto/merkle_tree/hash.hpp"
#include "barretenberg/crypto/merkle_tree/memory_store.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include <benchmark/benchmark.h>

using namespace benchmark;
using namespace bb;
using namespace bb::crypto::merkle_tree;

using TreeType = MerkleTree<MemoryStore, PedersenHashPolicy>;

namespace {
auto& engine = bb::numeric::get_debug_randomness();
} // namespace

constexpr size_t DEPTH = 256;
constexpr size_t MAX = 4096;

static std::vector<fr> VALUES = []() {
    std::vector<fr> values(MAX);
    for (size_t i = 0; i < MAX; ++i) {
        values[i] = fr(i);
    }
    return values;
}();

void hash(State& state) noexcept
{
    for (auto _ : state) {
        hash_pair_native({ 0, 0, 0, 0 }, { 1, 1, 1, 1 });
    }
}
BENCHMARK(hash)->MinTime(5);

void update_first_element(State& state) noexcept
{
    MemoryStore store;
    TreeType db(store, DEPTH);

    for (auto _ : state) {
        db.update_element(0, VALUES[1]);
    }
}
BENCHMARK(update_first_element)->Unit(benchmark::kMillisecond);

void update_elements(State& state) noexcept
{
    for (auto _ : state) {
        state.PauseTiming();
        MemoryStore store;
        TreeType db(store, DEPTH);
        state.ResumeTiming();
        for (size_t i = 0; i < (size_t)state.range(0); ++i) {
            db.update_element(i, VALUES[i]);
        }
    }
}
BENCHMARK(update_elements)->Unit(benchmark::kMillisecond)->RangeMultiplier(2)->Range(256, MAX);

void update_random_elements(State& state) noexcept
{
    for (auto _ : state) {
        state.PauseTiming();
        MemoryStore store;
        TreeType db(store, DEPTH);
        for (size_t i = 0; i < (size_t)state.range(0); i++) {
            state.PauseTiming();
            auto index = TreeType::index_t(engine.get_random_uint256());
            state.ResumeTiming();
            db.update_element(index, VALUES[i]);
        }
    }
}
BENCHMARK(update_random_elements)->Unit(benchmark::kMillisecond)->Range(100, 100)->Iterations(1);

BENCHMARK_MAIN();
