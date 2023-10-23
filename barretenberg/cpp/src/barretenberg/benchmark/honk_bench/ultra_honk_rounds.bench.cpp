#include <benchmark/benchmark.h>

#include "barretenberg/benchmark/honk_bench/benchmark_utilities.hpp"
#include "barretenberg/honk/composer/ultra_composer.hpp"
#include "barretenberg/honk/proof_system/ultra_prover.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"

using namespace benchmark;
using namespace proof_system;

// The rounds to measure
enum { PREAMBLE, WIRE_COMMITMENTS, SORTED_LIST_ACCUMULATOR, GRAND_PRODUCT_COMPUTATION, RELATION_CHECK, ZEROMORPH };

/**
 * @details Benchmark ultrahonk by performing all the rounds, but only measuring one.
 * Note: As a result the very short rounds take a long time for statistical significance, so recommended to set their
 * iterations to 1.
 * @param state - The google benchmark state.
 * @param prover - The ultrahonk prover.
 * @param index - The pass to measure.
 **/
BBERG_PROFILE static void test_round_inner(State& state, honk::UltraProver& prover, size_t index) noexcept
{
    auto time_if_index = [&](size_t target_index, auto&& func) -> void {
        if (index == target_index) {
            state.ResumeTiming();
            func();
            state.PauseTiming();
        } else {
            func();
        }
    };
    for (auto _ : state) {
        state.PauseTiming();
        time_if_index(PREAMBLE, [&] { prover.execute_preamble_round(); });
        time_if_index(WIRE_COMMITMENTS, [&] { prover.execute_wire_commitments_round(); });
        time_if_index(SORTED_LIST_ACCUMULATOR, [&] { prover.execute_sorted_list_accumulator_round(); });
        time_if_index(GRAND_PRODUCT_COMPUTATION, [&] { prover.execute_grand_product_computation_round(); });
        time_if_index(RELATION_CHECK, [&] { prover.execute_relation_check_rounds(); });
        time_if_index(ZEROMORPH, [&] { prover.execute_zeromorph_rounds(); });
        state.ResumeTiming();
    }
}
BBERG_PROFILE static void test_round(State& state, size_t index) noexcept
{
    barretenberg::srs::init_crs_factory("../srs_db/ignition");

    honk::UltraComposer composer;
    // TODO(AD) benchmark both sparse and dense circuits?
    honk::UltraProver prover =
        bench_utils::get_prover(composer, &bench_utils::generate_keccak_test_circuit<UltraCircuitBuilder>, 1);
    test_round_inner(state, prover, index);
}
#define ROUND_BENCHMARK(round)                                                                                         \
    static void ROUND_##round(State& state) noexcept                                                                   \
    {                                                                                                                  \
        test_round(state, round);                                                                                      \
    }                                                                                                                  \
    BENCHMARK(ROUND_##round)->Unit(::benchmark::kMillisecond)

// Fast rounds take a long time to benchmark because of how we compute statistical significance.
// Limit to one iteration so we don't spend a lot of time redoing full proofs just to measure this part.
ROUND_BENCHMARK(PREAMBLE)->Iterations(1);
ROUND_BENCHMARK(WIRE_COMMITMENTS)->Iterations(1);
ROUND_BENCHMARK(SORTED_LIST_ACCUMULATOR)->Iterations(1);
ROUND_BENCHMARK(GRAND_PRODUCT_COMPUTATION)->Iterations(1);
ROUND_BENCHMARK(RELATION_CHECK);
ROUND_BENCHMARK(ZEROMORPH);