#include "barretenberg/benchmark/honk_bench/benchmark_utilities.hpp"

using namespace benchmark;

namespace standard_honk_bench {

using StandardHonk = proof_system::honk::StandardHonkComposer;

// Log number of gates for test circuit
constexpr size_t MIN_LOG_NUM_GATES = bench_utils::BenchParams::MIN_LOG_NUM_GATES;
constexpr size_t MAX_LOG_NUM_GATES = bench_utils::BenchParams::MAX_LOG_NUM_GATES;
// Number of times to repeat each benchmark
constexpr size_t NUM_REPETITIONS = bench_utils::BenchParams::NUM_REPETITIONS;

/**
 * @brief Benchmark: Construction of a Standard proof for a circuit determined by the provided circuit function
 */
void construct_proof_standard(State& state, void (*test_circuit_function)(StandardHonk&, size_t)) noexcept
{
    bench_utils::construct_proof_with_specified_num_gates(state, test_circuit_function);
}

BENCHMARK_CAPTURE(construct_proof_standard, arithmetic, &bench_utils::generate_basic_arithmetic_circuit<StandardHonk>)
    ->DenseRange(MIN_LOG_NUM_GATES, MAX_LOG_NUM_GATES)
    ->Repetitions(NUM_REPETITIONS)
    ->Unit(::benchmark::kSecond);

} // namespace standard_honk_bench