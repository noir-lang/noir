#include "barretenberg/benchmark/honk_bench/benchmark_utilities.hpp"
#include "barretenberg/plonk/composer/standard_composer.hpp"
#include "barretenberg/proof_system/circuit_builder/standard_circuit_builder.hpp"

using namespace benchmark;

namespace standard_plonk_bench {

using StandardBuilder = proof_system::StandardCircuitBuilder;
using StandardPlonk = proof_system::plonk::StandardComposer;

// Log number of gates for test circuit
constexpr size_t MIN_LOG_NUM_GATES = bench_utils::BenchParams::MIN_LOG_NUM_GATES;
constexpr size_t MAX_LOG_NUM_GATES = bench_utils::BenchParams::MAX_LOG_NUM_GATES;
// Number of times to repeat each benchmark
constexpr size_t NUM_REPETITIONS = bench_utils::BenchParams::NUM_REPETITIONS;

/**
 * @brief Benchmark: Construction of a Standard proof for a circuit determined by the provided circuit function
 */
void construct_proof_standard(State& state, void (*test_circuit_function)(StandardBuilder&, size_t)) noexcept
{
    bench_utils::construct_proof_with_specified_num_gates<StandardPlonk>(state, test_circuit_function);
}

BENCHMARK_CAPTURE(construct_proof_standard,
                  arithmetic,
                  &bench_utils::generate_basic_arithmetic_circuit<StandardBuilder>)
    ->DenseRange(MIN_LOG_NUM_GATES, MAX_LOG_NUM_GATES)
    ->Repetitions(NUM_REPETITIONS)
    ->Unit(::benchmark::kSecond);

} // namespace standard_plonk_bench