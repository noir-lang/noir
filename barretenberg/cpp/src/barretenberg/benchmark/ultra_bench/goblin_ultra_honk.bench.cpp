#include <benchmark/benchmark.h>

#include "barretenberg/benchmark/ultra_bench/mock_circuits.hpp"
#include "barretenberg/stdlib_circuit_builders/goblin_ultra_circuit_builder.hpp"

using namespace benchmark;
using namespace bb;

/**
 * @brief Benchmark: Construction of a Ultra Honk proof for a circuit determined by the provided circuit function
 */
static void construct_proof_goblinultrahonk(State& state,
                                            void (*test_circuit_function)(GoblinUltraCircuitBuilder&, size_t)) noexcept
{
    size_t num_iterations = 10; // 10x the circuit
    bb::mock_circuits::construct_proof_with_specified_num_iterations<GoblinUltraProver>(
        state, test_circuit_function, num_iterations);
}

/**
 * @brief Benchmark: Construction of a Ultra Plonk proof with 2**n gates
 */
static void construct_proof_goblinultrahonk_power_of_2(State& state) noexcept
{
    auto log2_of_gates = static_cast<size_t>(state.range(0));
    bb::mock_circuits::construct_proof_with_specified_num_iterations<GoblinUltraProver>(
        state, &bb::mock_circuits::generate_basic_arithmetic_circuit<GoblinUltraCircuitBuilder>, log2_of_gates);
}

// Define benchmarks
BENCHMARK_CAPTURE(construct_proof_goblinultrahonk,
                  sha256,
                  &stdlib::generate_sha256_test_circuit<GoblinUltraCircuitBuilder>)
    ->Unit(kMillisecond);
BENCHMARK_CAPTURE(construct_proof_goblinultrahonk,
                  keccak,
                  &stdlib::generate_keccak_test_circuit<GoblinUltraCircuitBuilder>)
    ->Unit(kMillisecond);
BENCHMARK_CAPTURE(construct_proof_goblinultrahonk,
                  ecdsa_verification,
                  &stdlib::generate_ecdsa_verification_test_circuit<GoblinUltraCircuitBuilder>)
    ->Unit(kMillisecond);
BENCHMARK_CAPTURE(construct_proof_goblinultrahonk,
                  merkle_membership,
                  &stdlib::generate_merkle_membership_test_circuit<GoblinUltraCircuitBuilder>)
    ->Unit(kMillisecond);

BENCHMARK(construct_proof_goblinultrahonk_power_of_2)
    // 2**15 gates to 2**20 gates
    ->DenseRange(15, 20)
    ->Unit(kMillisecond);

BENCHMARK_MAIN();
