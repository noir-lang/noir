/**
 * @file sha256.bench.cpp
 * @author Rumata888
 * @brief This file contains benchmarks for an external benchmark project https://github.com/celer-network/zk-benchmark
 * @version 0.1
 * @date 2023-08-02
 *
 */
#include "barretenberg/stdlib/hash/sha256/sha256.hpp"
#include "barretenberg/plonk/composer/ultra_composer.hpp"
#include <benchmark/benchmark.h>
#include <bit>

using namespace benchmark;

using Builder = bb::UltraCircuitBuilder;
using Composer = bb::plonk::UltraComposer;
using Prover = bb::plonk::UltraProver;
using Verifier = bb::plonk::UltraVerifier;

constexpr size_t NUM_HASHES = 20;
constexpr size_t CHUNK_SIZE = 64;
constexpr size_t MINIMUM_CHUNKS = 1;
constexpr size_t MAXIMUM_CHUNKS = 1024;

/**
 * @brief Generate a circuit computing sha256 hash of a 0-filled array of num_bytes
 *
 * @param builder circuit builder
 * @param num_bytes Length of the array
 */
void generate_test_plonk_circuit(Builder& builder, size_t num_bytes)
{
    std::string in;
    in.resize(num_bytes);
    bb::stdlib::packed_byte_array<Builder> input(&builder, in);

    bb::stdlib::sha256<Builder>(input);
}

// Because of the way we do internal allocations in some of our more complex structures, we can't just globally allocate
// them
Builder* builders[NUM_HASHES];
Composer* composers[NUM_HASHES];
Prover provers[NUM_HASHES];
Verifier verifiers[NUM_HASHES];
plonk::proof proofs[NUM_HASHES];

/**
 * @brief Benchmark for constructing the circuit, witness polynomials, proving and verification key
 *
 * @param state
 */
void preprocess_and_construct_witnesses_bench(State& state) noexcept
{
    for (auto _ : state) {
        size_t num_chunks = static_cast<size_t>(state.range(0));
        size_t idx = static_cast<size_t>(std::countr_zero(num_chunks));
        builders[idx] = new Builder();
        generate_test_plonk_circuit(*builders[idx], num_chunks * CHUNK_SIZE);
        composers[idx] = new Composer();
        provers[idx] = (composers[idx])->create_prover(*builders[idx]);
        std::cout << "prover subgroup size = " << provers[idx].key->small_domain.size << std::endl;

        verifiers[idx] = (composers[idx])->create_verifier(*builders[idx]);
    }
}
BENCHMARK(preprocess_and_construct_witnesses_bench)
    ->RangeMultiplier(2)
    ->Range(MINIMUM_CHUNKS, MAXIMUM_CHUNKS)
    ->Unit(benchmark::kMillisecond);

/**
 * @brief Benchmark for creating the proof with preprocessed data
 *
 * @param state
 */
void construct_proofs_bench(State& state) noexcept
{
    for (auto _ : state) {
        size_t num_chunks = static_cast<size_t>(state.range(0));
        size_t idx = static_cast<size_t>(std::countr_zero(num_chunks));

        proofs[idx] = provers[idx].construct_proof();
        std::cout << "Plonk proof size: " << proofs[idx].proof_data.size() << std::endl;
        state.PauseTiming();
        provers[idx].reset();
        state.ResumeTiming();
    }
}
BENCHMARK(construct_proofs_bench)
    ->RangeMultiplier(2)
    ->Range(MINIMUM_CHUNKS, MAXIMUM_CHUNKS)
    ->Unit(benchmark::kMillisecond);

/**
 * @brief Benchmark for proof verification
 *
 * @param state
 */
void verify_proofs_bench(State& state) noexcept
{
    for (auto _ : state) {
        size_t num_chunks = static_cast<size_t>(state.range(0));
        size_t idx = static_cast<size_t>(std::countr_zero(num_chunks));
        verifiers[idx].verify_proof(proofs[idx]);
    }
}
BENCHMARK(verify_proofs_bench)
    ->RangeMultiplier(2)
    ->Range(MINIMUM_CHUNKS, MAXIMUM_CHUNKS)
    ->Unit(benchmark::kMillisecond);

BENCHMARK_MAIN();
