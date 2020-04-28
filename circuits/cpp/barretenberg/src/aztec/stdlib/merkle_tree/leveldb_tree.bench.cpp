#include "hash.hpp"
#include "leveldb_store.hpp"
#include "leveldb_tree.hpp"
#include <benchmark/benchmark.h>
#include <leveldb/db.h>
#include <numeric/random/engine.hpp>

using namespace benchmark;
using namespace plonk::stdlib::merkle_tree;

namespace {
auto& engine = numeric::random::get_debug_engine();
}

constexpr size_t DEPTH = 128;
constexpr size_t MAX = 4096;
const std::string DB_PATH = "/tmp/leveldb_test";

static std::vector<LevelDbTree::value_t> VALUES = []() {
    std::vector<LevelDbTree::value_t> values(MAX);
    for (size_t i = 0; i < MAX; ++i) {
        LevelDbTree::value_t v(64, 0);
        *(size_t*)v.data() = i;
        values[i] = v;
    }
    return values;
}();

void hash(State& state) noexcept
{
    for (auto _ : state) {
        compress_native({ { 0, 0, 0, 0 }, { 1, 1, 1, 1 } });
    }
}
BENCHMARK(hash)->MinTime(5);

void update_first_element(State& state) noexcept
{
    LevelDbStore::destroy(DB_PATH);
    LevelDbStore store(DB_PATH);
    LevelDbTree db(store, DEPTH);

    for (auto _ : state) {
        db.update_element(0, VALUES[1]);
    }
}
BENCHMARK(update_first_element)->Unit(benchmark::kMillisecond);

void update_elements(State& state) noexcept
{
    for (auto _ : state) {
        state.PauseTiming();
        LevelDbStore::destroy(DB_PATH);
        LevelDbStore store(DB_PATH);
        LevelDbTree db(store, DEPTH);
        state.ResumeTiming();
        for (size_t i = 0; i < (size_t)state.range(0); ++i) {
            db.update_element(i, VALUES[i]);
        }
    }
}
BENCHMARK(update_elements)->Unit(benchmark::kMillisecond)->RangeMultiplier(2)->Range(256, MAX);

void update_1024_random_elements(State& state) noexcept
{
    for (auto _ : state) {
        state.PauseTiming();
        LevelDbStore::destroy(DB_PATH);
        LevelDbStore store(DB_PATH);
        LevelDbTree db(store, DEPTH);
        for (size_t i = 0; i < 1024; i++) {
            state.PauseTiming();
            LevelDbTree::index_t index = engine.get_random_uint128();
            state.ResumeTiming();
            db.update_element(index, VALUES[i]);
        }
    }
}
BENCHMARK(update_1024_random_elements)->Unit(benchmark::kMillisecond);

BENCHMARK_MAIN();
