#include "barretenberg/benchmark/honk_bench/benchmark_utilities.hpp"
#include "barretenberg/plonk/composer/standard_composer.hpp"
#include "barretenberg/proof_system/circuit_builder/standard_circuit_builder.hpp"

using namespace benchmark;

using StandardBuilder = proof_system::StandardCircuitBuilder;
using StandardPlonk = proof_system::plonk::StandardComposer;

/**
 * @brief Benchmark: Construction of a Standard proof for a circuit determined by the provided circuit function
 */
static void construct_proof_standard_power_of_2(State& state) noexcept
{
    auto log2_of_gates = static_cast<size_t>(state.range(0));
    bench_utils::construct_proof_with_specified_num_iterations<proof_system::plonk::StandardComposer>(
        state, &bench_utils::generate_basic_arithmetic_circuit<proof_system::StandardCircuitBuilder>, log2_of_gates);
}

BENCHMARK(construct_proof_standard_power_of_2)
    // 2**13 gates to 2**18 gates
    ->DenseRange(13, 18)
    ->Unit(::benchmark::kMillisecond);