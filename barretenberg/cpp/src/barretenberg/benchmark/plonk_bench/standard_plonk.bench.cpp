#include "barretenberg/benchmark/ultra_bench/benchmark_utilities.hpp"
#include "barretenberg/plonk/composer/standard_composer.hpp"
#include "barretenberg/proof_system/circuit_builder/standard_circuit_builder.hpp"

using namespace benchmark;

using StandardBuilder = bb::StandardCircuitBuilder;
using StandardPlonk = bb::plonk::StandardComposer;

/**
 * @brief Benchmark: Construction of a Standard proof for a circuit determined by the provided circuit function
 */
static void construct_proof_standard_power_of_2(State& state) noexcept
{
    auto log2_of_gates = static_cast<size_t>(state.range(0));
    bench_utils::construct_proof_with_specified_num_iterations<bb::plonk::StandardComposer>(
        state, &bench_utils::generate_basic_arithmetic_circuit<bb::StandardCircuitBuilder>, log2_of_gates);
}

BENCHMARK(construct_proof_standard_power_of_2)
    // 2**15 gates to 2**20 gates
    ->DenseRange(15, 20)
    ->Unit(::benchmark::kMillisecond);