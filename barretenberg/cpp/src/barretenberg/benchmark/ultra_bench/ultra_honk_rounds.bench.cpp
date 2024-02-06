#include <benchmark/benchmark.h>

#include "barretenberg/benchmark/ultra_bench/mock_proofs.hpp"
#include "barretenberg/common/op_count_google_bench.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"
#include "barretenberg/ultra_honk/ultra_composer.hpp"
#include "barretenberg/ultra_honk/ultra_prover.hpp"

using namespace benchmark;
using namespace bb;

// The rounds to measure
enum {
    PREAMBLE,
    WIRE_COMMITMENTS,
    SORTED_LIST_ACCUMULATOR,
    LOG_DERIVATIVE_INVERSE,
    GRAND_PRODUCT_COMPUTATION,
    RELATION_CHECK,
    ZEROMORPH
};

/**
 * @details Benchmark ultrahonk by performing all the rounds, but only measuring one.
 * Note: As a result the very short rounds take a long time for statistical significance, so recommended to set their
 * iterations to 1.
 * @param state - The google benchmark state.
 * @param prover - The ultrahonk prover.
 * @param index - The pass to measure.
 **/
BB_PROFILE static void test_round_inner(State& state, UltraProver& prover, size_t index) noexcept
{
    auto time_if_index = [&](size_t target_index, auto&& func) -> void {
        BB_REPORT_OP_COUNT_IN_BENCH(state);
        if (index == target_index) {
            state.ResumeTiming();
        }

        func();
        if (index == target_index) {
            state.PauseTiming();
        } else {
            // We don't actually want to write to user-defined counters
            BB_REPORT_OP_COUNT_BENCH_CANCEL();
        }
    };

    time_if_index(PREAMBLE, [&] { prover.execute_preamble_round(); });
    time_if_index(WIRE_COMMITMENTS, [&] { prover.execute_wire_commitments_round(); });
    time_if_index(SORTED_LIST_ACCUMULATOR, [&] { prover.execute_sorted_list_accumulator_round(); });
    time_if_index(LOG_DERIVATIVE_INVERSE, [&] { prover.execute_log_derivative_inverse_round(); });
    time_if_index(GRAND_PRODUCT_COMPUTATION, [&] { prover.execute_grand_product_computation_round(); });
    time_if_index(RELATION_CHECK, [&] { prover.execute_relation_check_rounds(); });
    time_if_index(ZEROMORPH, [&] { prover.execute_zeromorph_rounds(); });
}
BB_PROFILE static void test_round(State& state, size_t index) noexcept
{
    bb::srs::init_crs_factory("../srs_db/ignition");

    for (auto _ : state) {
        state.PauseTiming();
        UltraComposer composer;
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/761) benchmark both sparse and dense circuits
        UltraProver prover = bb::mock_proofs::get_prover(
            composer, &bb::stdlib::generate_ecdsa_verification_test_circuit<UltraCircuitBuilder>, 10);
        test_round_inner(state, prover, index);
        state.ResumeTiming();
        // NOTE: google bench is very finnicky, must end in ResumeTiming() for correctness
    }
}
#define ROUND_BENCHMARK(round)                                                                                         \
    static void ROUND_##round(State& state) noexcept                                                                   \
    {                                                                                                                  \
        test_round(state, round);                                                                                      \
    }                                                                                                                  \
    BENCHMARK(ROUND_##round)->Unit(kMillisecond)

// Fast rounds take a long time to benchmark because of how we compute statistical significance.
// Limit to one iteration so we don't spend a lot of time redoing full proofs just to measure this part.
ROUND_BENCHMARK(PREAMBLE)->Iterations(1);
ROUND_BENCHMARK(WIRE_COMMITMENTS)->Iterations(1);
ROUND_BENCHMARK(SORTED_LIST_ACCUMULATOR)->Iterations(1);
ROUND_BENCHMARK(LOG_DERIVATIVE_INVERSE)->Iterations(1);
ROUND_BENCHMARK(GRAND_PRODUCT_COMPUTATION)->Iterations(1);
ROUND_BENCHMARK(RELATION_CHECK);
ROUND_BENCHMARK(ZEROMORPH);

BENCHMARK_MAIN();
