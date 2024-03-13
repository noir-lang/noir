/**
 * @file external.bench.cpp
 * @author Kesha (Rumata888)
 * @brief Benchmarks for external benchmarking projects (e.g. delendum-xyz)
 *
 */
#include <benchmark/benchmark.h>

#include "barretenberg/plonk/composer/ultra_composer.hpp"
#include "barretenberg/stdlib/hash/blake3s/blake3s.hpp"
#include "barretenberg/stdlib/hash/sha256/sha256.hpp"

using namespace benchmark;

using Builder = bb::UltraCircuitBuilder;
using Composer = bb::plonk::UltraComposer;

using Prover = bb::plonk::UltraProver;
using Verifier = bb::plonk::UltraVerifier;

constexpr size_t PROOF_COUNT_LOG = 10;
constexpr size_t NUM_PROOFS = 3;

/**
 * @brief Main function generating a circuit with num_iterations sequential sha256 hashes, where the output of a
 * previous iteration is fed into the next one
 *
 * @param builder
 * @param num_iterations
 */
void generate_test_sha256_plonk_circuit(Builder& builder, size_t num_iterations)
{
    std::string in;
    in.resize(32);
    for (size_t i = 0; i < 32; ++i) {
        in[i] = 0;
    }

    bb::stdlib::packed_byte_array<Builder> input(&builder, in);
    for (size_t i = 0; i < num_iterations; i++) {
        input = bb::stdlib::sha256<Builder>(input);
    }
}

void* external_builders[NUM_PROOFS];
void* external_composers[NUM_PROOFS];
Prover external_provers[NUM_PROOFS];
Verifier external_verifiers[NUM_PROOFS];
plonk::proof external_proofs[NUM_PROOFS];

/**
 * @brief Construct the circuit for sequential sha256 proofs and compute the proof for each case
 *
 * @param state
 */
void generate_sha256_proof_bench(State& state) noexcept
{
    for (auto _ : state) {
        size_t idx = static_cast<size_t>(state.range(0));
        size_t num_iterations = 1;
        for (size_t i = 0; i < idx; i++) {
            num_iterations *= PROOF_COUNT_LOG;
        }
        external_composers[idx] = (void*)new Composer();
        external_builders[idx] = (void*)new Builder();
        generate_test_sha256_plonk_circuit(*(Builder*)external_builders[idx], num_iterations);
        external_provers[idx] = ((Composer*)external_composers[idx])->create_prover(*(Builder*)external_builders[idx]);
        external_proofs[idx] = external_provers[idx].construct_proof();
        // info("Proof Size for SHA256 hash count ", num_iterations, ": ", external_proofs[idx].proof_data.size());
    }
}

/**
 * @brief We have to warm up the benchmarking function first, otherwise we spend 50% more time than expected
 *
 */
BENCHMARK(generate_sha256_proof_bench)->DenseRange(0, 2)->MinWarmUpTime(10)->MinTime(2)->Unit(benchmark::kMillisecond);
/**
 * @brief Create sha256 verifier
 *
 * @details We don't want to benchmark this
 *
 * @param state
 */
static void generate_sha256_verifier(const State& state)
{

    size_t idx = static_cast<size_t>(state.range(0));
    external_verifiers[idx] = ((Composer*)external_composers[idx])->create_verifier(*(Builder*)external_builders[idx]);
}
/**
 * @brief Benchmark sha256 verification
 *
 * @param state
 */
void verify_sha256_proof_bench(State& state) noexcept
{
    for (auto _ : state) {

        size_t idx = static_cast<size_t>(state.range(0));
        external_verifiers[idx].verify_proof(external_proofs[idx]);
    }
}

BENCHMARK(verify_sha256_proof_bench)->DenseRange(0, 2)->Setup(generate_sha256_verifier)->Unit(benchmark::kMillisecond);

/**
 * @brief Main function for generating Blake 3 circuits
 *
 * @param builder
 * @param num_iterations
 */
void generate_test_blake3s_plonk_circuit(Builder& builder, size_t num_iterations)
{
    std::string in;
    in.resize(32);
    for (size_t i = 0; i < 32; ++i) {
        in[i] = 0;
    }
    bb::stdlib::packed_byte_array<Builder> input(&builder, in);
    for (size_t i = 0; i < num_iterations; i++) {
        input = bb::stdlib::blake3s<Builder>(input);
    }
}

/**
 * @brief Blake3 circuit construction and proof creation benchmark function
 *
 * @param state
 */
void generate_blake3s_proof_bench(State& state) noexcept
{
    for (auto _ : state) {

        size_t idx = static_cast<size_t>(state.range(0));
        size_t num_iterations = 1;
        for (size_t i = 0; i < idx; i++) {
            num_iterations *= PROOF_COUNT_LOG;
        }
        external_composers[idx] = new Composer();
        generate_test_blake3s_plonk_circuit(*(Builder*)external_builders[idx], num_iterations);
        external_provers[idx] = ((Composer*)external_composers[idx])->create_prover(*(Builder*)external_builders[idx]);
        external_proofs[idx] = external_provers[idx].construct_proof();
        // Proof size with no public inputs is always 2144
        // info("Proof Size for Blake3s hash count ", num_iterations, ": ", external_proofs[idx].proof_data.size());
    }
}

BENCHMARK(generate_blake3s_proof_bench)->DenseRange(0, 2)->MinWarmUpTime(10)->MinTime(2)->Unit(benchmark::kMillisecond);

/**
 * @brief Create blake 3 verifier
 *
 * @details We don't benchmark verifier creation
 *
 * @param state
 */
static void generate_blake3s_verifier(const State& state)
{

    size_t idx = static_cast<size_t>(state.range(0));
    external_verifiers[idx] = ((Composer*)external_composers[idx])->create_verifier(*(Builder*)external_builders[idx]);
}

/**
 * @brief Benchmark blake3 proof verification
 *
 * @param state
 */
void verify_blake3s_proof_bench(State& state) noexcept
{
    for (auto _ : state) {

        size_t idx = static_cast<size_t>(state.range(0));
        external_verifiers[idx].verify_proof(external_proofs[idx]);
    }
}

BENCHMARK(verify_blake3s_proof_bench)
    ->DenseRange(0, 2)
    ->Setup(generate_blake3s_verifier)
    ->Unit(benchmark::kMillisecond);

BENCHMARK_MAIN();