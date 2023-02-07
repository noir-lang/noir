#include "ecc/curves/bn254/fr.hpp"
#include "honk/proof_system/prover.hpp"
#include "honk/proof_system/prover.hpp"
#include "honk/proof_system/verifier.hpp"
#include <benchmark/benchmark.h>
#include <cstddef>
#include <honk/composer/standard_honk_composer.hpp>
#include <stdlib/primitives/field/field.hpp>

using namespace benchmark;

constexpr size_t MIN_LOG_NUM_GATES = 10; // num_gates = 1 << long_num_gates
constexpr size_t MAX_LOG_NUM_GATES = 16;

// IMPROVEMENT: Ideally we would save the data computed in each benchmark for use in benchmarks. For example, save the
// proofs constructed in the construction benchmark for use in the verification benchmark rather than recomputing them.

// constexpr size_t NUM_CIRCUITS = MAX_LOG_NUM_GATES - MIN_LOG_NUM_GATES + 1;
// honk::StandardUnrolledProver provers[NUM_CIRCUITS];
// honk::StandardUnrolledVerifier verifiers[NUM_CIRCUITS];
// waffle::plonk_proof proofs[NUM_CIRCUITS];

void generate_test_plonk_circuit(auto& composer, size_t log_num_gates)
{
    size_t num_gates = 1 << log_num_gates;
    plonk::stdlib::field_t a(plonk::stdlib::witness_t(&composer, barretenberg::fr::random_element()));
    plonk::stdlib::field_t b(plonk::stdlib::witness_t(&composer, barretenberg::fr::random_element()));
    plonk::stdlib::field_t c(&composer);
    for (size_t i = 0; i < (num_gates / 4) - 4; ++i) {
        c = a + b;
        c = a * c;
        a = b * b;
        b = c * c;
    }
}

/**
 * @brief Benchmark: Creation of an unrolled standard Honk prover
 */
void create_unrolled_prover_bench(State& state) noexcept
{
    for (auto _ : state) {
        auto log_num_gates = (size_t)state.range(0);
        auto composer = honk::StandardHonkComposer(static_cast<size_t>(log_num_gates));
        generate_test_plonk_circuit(composer, static_cast<size_t>(log_num_gates));
        // size_t idx = MAX_LOG_NUM_GATES - log_num_gates;
        composer.create_unrolled_prover();
    }
}
BENCHMARK(create_unrolled_prover_bench)->DenseRange(MIN_LOG_NUM_GATES, MAX_LOG_NUM_GATES, 1);

/**
 * @brief Benchmark: Creation of an unrolled standard Honk verifier
 */
void create_unrolled_verifier_bench(State& state) noexcept
{
    for (auto _ : state) {
        state.PauseTiming();
        auto log_num_gates = (size_t)state.range(0);
        auto composer = honk::StandardHonkComposer(static_cast<size_t>(log_num_gates));
        generate_test_plonk_circuit(composer, static_cast<size_t>(log_num_gates));
        state.ResumeTiming();

        composer.create_unrolled_verifier();
    }
}
BENCHMARK(create_unrolled_verifier_bench)->DenseRange(MIN_LOG_NUM_GATES, MAX_LOG_NUM_GATES, 1);

/**
 * @brief Benchmark: Construction of an unrolled standard Honk proof
 */
void construct_proof_bench(State& state) noexcept
{
    for (auto _ : state) {
        state.PauseTiming();
        auto log_num_gates = (size_t)state.range(0);
        auto composer = honk::StandardHonkComposer(static_cast<size_t>(log_num_gates));
        generate_test_plonk_circuit(composer, static_cast<size_t>(log_num_gates));
        auto ext_prover = composer.create_unrolled_prover();
        state.ResumeTiming();

        auto proof = ext_prover.construct_proof();
    }
    state.SetComplexityN(state.range(0)); // Set up for computation of constant C where prover ~ C*N
}
BENCHMARK(construct_proof_bench)->DenseRange(MIN_LOG_NUM_GATES, MAX_LOG_NUM_GATES, 1)->Complexity(benchmark::oN);

/**
 * @brief Benchmark: Verification of an unrolled standard Honk proof
 */
void verify_proof_bench(State& state) noexcept
{
    for (auto _ : state) {
        state.PauseTiming();
        auto log_num_gates = (size_t)state.range(0);
        auto composer = honk::StandardHonkComposer(static_cast<size_t>(log_num_gates));
        generate_test_plonk_circuit(composer, static_cast<size_t>(log_num_gates));
        auto prover = composer.create_unrolled_prover();
        auto proof = prover.construct_proof();
        auto verifier = composer.create_unrolled_verifier();
        state.ResumeTiming();

        verifier.verify_proof(proof);
    }
}
// Note: enforcing Iterations == 1 for now. Otherwise proof construction will occur many times and this bench will take
// a long time. (This is because the time limit for benchmarks does not include the time-excluded setup, and
// verification itself is pretty fast).
BENCHMARK(verify_proof_bench)->DenseRange(MIN_LOG_NUM_GATES, MAX_LOG_NUM_GATES, 1)->Iterations(1);

BENCHMARK_MAIN();