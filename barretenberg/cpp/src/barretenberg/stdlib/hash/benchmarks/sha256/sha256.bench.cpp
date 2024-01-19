#include "barretenberg/stdlib/hash/sha256/sha256.hpp"
#include "barretenberg/plonk/composer/ultra_composer.hpp"
#include <benchmark/benchmark.h>

using namespace benchmark;

using Builder = bb::UltraCircuitBuilder;
using Composer = bb::plonk::UltraComposer;
using Prover = bb::plonk::UltraProver;
using Verifier = bb::plonk::UltraVerifier;

constexpr size_t NUM_HASHES = 8;
constexpr size_t BYTES_PER_CHUNK = 512;
constexpr size_t START_BYTES = BYTES_PER_CHUNK - 9;
constexpr size_t MAX_BYTES = START_BYTES + (BYTES_PER_CHUNK * (NUM_HASHES - 1));

char get_random_char()
{
    return static_cast<char>(bb::fr::random_element().data[0] % 8);
}

void generate_test_plonk_circuit(Builder& builder, size_t num_bytes)
{
    std::string in;
    in.resize(num_bytes);
    for (size_t i = 0; i < num_bytes; ++i) {
        in[i] = get_random_char();
    }
    bb::stdlib::packed_byte_array<Builder> input(&builder, in);
    bb::stdlib::sha256<Builder>(input);
}

void* builders[NUM_HASHES];
void* composers[NUM_HASHES];
Prover provers[NUM_HASHES];
Verifier verifiers[NUM_HASHES];
plonk::proof proofs[NUM_HASHES];

void construct_witnesses_bench(State& state) noexcept
{
    for (auto _ : state) {
        size_t idx = (static_cast<size_t>((state.range(0))) - START_BYTES) / BYTES_PER_CHUNK;
        builders[idx] = (void*)new Builder();
        generate_test_plonk_circuit(*(Builder*)builders[idx], static_cast<size_t>(state.range(0)));
    }
}
BENCHMARK(construct_witnesses_bench)->DenseRange(START_BYTES, MAX_BYTES, BYTES_PER_CHUNK);

void preprocess_witnesses_bench(State& state) noexcept
{
    for (auto _ : state) {
        size_t idx = (static_cast<size_t>((state.range(0))) - START_BYTES) / BYTES_PER_CHUNK;
        composers[idx] = (void*)new Composer();
        provers[idx] = ((Composer*)composers[idx])->create_prover(*(Builder*)builders[idx]);
        std::cout << "prover subgroup size = " << provers[idx].key->small_domain.size << std::endl;
    }
}
BENCHMARK(preprocess_witnesses_bench)->DenseRange(START_BYTES, MAX_BYTES, BYTES_PER_CHUNK);

void construct_instances_bench(State& state) noexcept
{
    for (auto _ : state) {
        size_t idx = (static_cast<size_t>((state.range(0))) - START_BYTES) / BYTES_PER_CHUNK;
        verifiers[idx] = ((Composer*)composers[idx])->create_verifier(*(Builder*)builders[idx]);
    }
}
BENCHMARK(construct_instances_bench)->DenseRange(START_BYTES, MAX_BYTES, BYTES_PER_CHUNK);

void construct_proofs_bench(State& state) noexcept
{
    for (auto _ : state) {
        size_t idx = (static_cast<size_t>((state.range(0))) - START_BYTES) / BYTES_PER_CHUNK;
        proofs[idx] = provers[idx].construct_proof();
        state.PauseTiming();
        provers[idx].reset();
        state.ResumeTiming();
    }
}
BENCHMARK(construct_proofs_bench)->DenseRange(START_BYTES, MAX_BYTES, BYTES_PER_CHUNK);

void verify_proofs_bench(State& state) noexcept
{
    for (auto _ : state) {
        size_t idx = (static_cast<size_t>((state.range(0))) - START_BYTES) / BYTES_PER_CHUNK;
        verifiers[idx].verify_proof(proofs[idx]);
    }
}
BENCHMARK(verify_proofs_bench)->DenseRange(START_BYTES, MAX_BYTES, BYTES_PER_CHUNK);

BENCHMARK_MAIN();
