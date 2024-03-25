#include <benchmark/benchmark.h>

#include "barretenberg/benchmark/ultra_bench/mock_circuits.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_circuit_builder.hpp"

using namespace benchmark;
using namespace bb;

// The rounds to measure
enum {
    PREAMBLE,
    FIRST_WIRE_COMMITMENTS,
    SECOND_FIAT_SHAMIR_ETA,
    THIRD_FIAT_SHAMIR_BETA_GAMMA,
    FOURTH_FIAT_SHAMIR_ALPHA_AND_COMMIT,
    FIFTH_COMPUTE_QUOTIENT_EVALUTION,
    SIXTH_BATCH_OPEN
};

BB_PROFILE static void plonk_round(
    State& state, plonk::UltraProver& prover, size_t target_index, size_t index, auto&& func) noexcept
{
    if (index == target_index) {
        state.ResumeTiming();
    }
    func();
    prover.queue.process_queue();
    if (index == target_index) {
        state.PauseTiming();
    }
}
/**
 * @details Benchmark ultraplonk by performing all the rounds, but only measuring one.
 * Note: As a result the very short rounds take a long time for statistical significance, so recommended to set
 *their iterations to 1.
 * @param state - The google benchmark state.
 * @param prover - The ultraplonk prover.
 * @param index - The pass to measure.
 **/
BB_PROFILE static void test_round_inner(State& state, plonk::UltraProver& prover, size_t index) noexcept
{
    plonk_round(state, prover, PREAMBLE, index, [&] { prover.execute_preamble_round(); });
    plonk_round(state, prover, FIRST_WIRE_COMMITMENTS, index, [&] { prover.execute_first_round(); });
    plonk_round(state, prover, SECOND_FIAT_SHAMIR_ETA, index, [&] { prover.execute_second_round(); });
    plonk_round(state, prover, THIRD_FIAT_SHAMIR_BETA_GAMMA, index, [&] { prover.execute_third_round(); });
    plonk_round(state, prover, FOURTH_FIAT_SHAMIR_ALPHA_AND_COMMIT, index, [&] { prover.execute_fourth_round(); });
    plonk_round(state, prover, FIFTH_COMPUTE_QUOTIENT_EVALUTION, index, [&] { prover.execute_fifth_round(); });
    plonk_round(state, prover, SIXTH_BATCH_OPEN, index, [&] { prover.execute_sixth_round(); });
}
BB_PROFILE static void test_round(State& state, size_t index) noexcept
{
    bb::srs::init_crs_factory("../srs_db/ignition");
    for (auto _ : state) {
        state.PauseTiming();
        // TODO: https://github.com/AztecProtocol/barretenberg/issues/761 benchmark both sparse and dense circuits
        auto prover = bb::mock_circuits::get_prover<plonk::UltraProver>(
            &bb::stdlib::generate_ecdsa_verification_test_circuit<UltraCircuitBuilder>, 10);
        test_round_inner(state, prover, index);
        // NOTE: google bench is very finnicky, must end in ResumeTiming() for correctness
        state.ResumeTiming();
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
ROUND_BENCHMARK(FIRST_WIRE_COMMITMENTS)->Iterations(1);
ROUND_BENCHMARK(SECOND_FIAT_SHAMIR_ETA)->Iterations(1);
ROUND_BENCHMARK(THIRD_FIAT_SHAMIR_BETA_GAMMA)->Iterations(1);
ROUND_BENCHMARK(FOURTH_FIAT_SHAMIR_ALPHA_AND_COMMIT)->Iterations(1);
ROUND_BENCHMARK(FIFTH_COMPUTE_QUOTIENT_EVALUTION)->Iterations(1);
ROUND_BENCHMARK(SIXTH_BATCH_OPEN)->Iterations(1);

BENCHMARK_MAIN();
