#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include <benchmark/benchmark.h>
#include <cstddef>
#include "barretenberg/honk/composer/standard_honk_composer.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "barretenberg/stdlib/primitives/witness/witness.hpp"

using namespace benchmark;
using namespace proof_system::plonk::stdlib;

namespace standard_honk_bench {

using Composer = proof_system::honk::StandardHonkComposer;

constexpr size_t MIN_LOG_NUM_GATES = 16;
constexpr size_t MAX_LOG_NUM_GATES = 16;
// To get good statistics, number of Repetitions must be sufficient. ~30 Repetitions gives good results.
constexpr size_t NUM_REPETITIONS = 5;

void generate_test_circuit(auto& composer, size_t num_gates)
{
    field_t a(witness_t(&composer, barretenberg::fr::random_element()));
    field_t b(witness_t(&composer, barretenberg::fr::random_element()));
    field_t c(&composer);
    for (size_t i = 0; i < (num_gates / 4) - 4; ++i) {
        c = a + b;
        c = a * c;
        a = b * b;
        b = c * c;
    }
}

/**
 * @brief Benchmark: Creation of a Standard Honk prover
 */
void create_prover_standard(State& state) noexcept
{
    for (auto _ : state) {
        state.PauseTiming();
        auto num_gates = 1 << (size_t)state.range(0);
        auto composer = Composer(static_cast<size_t>(num_gates));
        generate_test_circuit(composer, static_cast<size_t>(num_gates));
        state.ResumeTiming();

        composer.create_prover();
    }
}
BENCHMARK(create_prover_standard)->DenseRange(MIN_LOG_NUM_GATES, MAX_LOG_NUM_GATES, 1)->Repetitions(NUM_REPETITIONS);

/**
 * @brief Benchmark: Construction of a Standard Honk proof
 */
void construct_proof_standard(State& state) noexcept
{
    auto num_gates = 1 << (size_t)state.range(0);
    for (auto _ : state) {
        state.PauseTiming();
        auto composer = Composer(static_cast<size_t>(num_gates));
        generate_test_circuit(composer, static_cast<size_t>(num_gates));
        auto ext_prover = composer.create_prover();
        state.ResumeTiming();

        auto proof = ext_prover.construct_proof();
    }
    state.SetComplexityN(num_gates); // Set up for computation of constant C where prover ~ C*N
}
BENCHMARK(construct_proof_standard)
    ->DenseRange(MIN_LOG_NUM_GATES, MAX_LOG_NUM_GATES, 1)
    ->Repetitions(NUM_REPETITIONS)
    ->Complexity(oN);

/**
 * @brief Benchmark: Creation of a Standard Honk verifier
 */
void create_verifier_standard(State& state) noexcept
{
    for (auto _ : state) {
        state.PauseTiming();
        auto num_gates = 1 << (size_t)state.range(0);
        auto composer = Composer(static_cast<size_t>(num_gates));
        generate_test_circuit(composer, static_cast<size_t>(num_gates));
        state.ResumeTiming();

        composer.create_verifier();
    }
}
// BENCHMARK(create_verifier_standard)->DenseRange(MIN_LOG_NUM_GATES, MAX_LOG_NUM_GATES,
// 1)->Repetitions(NUM_REPETITIONS);

/**
 * @brief Benchmark: Verification of a Standard Honk proof
 */
void verify_proof_standard(State& state) noexcept
{
    for (auto _ : state) {
        state.PauseTiming();
        auto num_gates = (size_t)state.range(0);
        auto composer = Composer(static_cast<size_t>(num_gates));
        generate_test_circuit(composer, static_cast<size_t>(num_gates));
        auto prover = composer.create_prover();
        auto proof = prover.construct_proof();
        auto verifier = composer.create_verifier();
        state.ResumeTiming();

        verifier.verify_proof(proof);
    }
}
// BENCHMARK(verify_proof_standard)->DenseRange(MIN_LOG_NUM_GATES, MAX_LOG_NUM_GATES, 1)->Iterations(1);
} // namespace standard_honk_bench