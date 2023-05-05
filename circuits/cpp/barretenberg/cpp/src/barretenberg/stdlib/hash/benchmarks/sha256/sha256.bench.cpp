#include "../../sha256/sha256.hpp"
#include <benchmark/benchmark.h>
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/plonk/composer/ultra_composer.hpp"
#include "barretenberg/plonk/proof_system/prover/prover.hpp"
#include "barretenberg/stdlib/primitives/packed_byte_array/packed_byte_array.hpp"

using namespace benchmark;

using Composer = proof_system::plonk::UltraComposer;
using Prover = proof_system::plonk::UltraProver;
using Verifier = proof_system::plonk::UltraVerifier;

constexpr size_t NUM_HASHES = 8;
constexpr size_t BYTES_PER_CHUNK = 512;
constexpr size_t START_BYTES = BYTES_PER_CHUNK - 9;
constexpr size_t MAX_BYTES = START_BYTES + (BYTES_PER_CHUNK * (NUM_HASHES - 1));

char get_random_char()
{
    return static_cast<char>(barretenberg::fr::random_element().data[0] % 8);
}

void generate_test_plonk_circuit(Composer& composer, size_t num_bytes)
{
    std::string in;
    in.resize(num_bytes);
    for (size_t i = 0; i < num_bytes; ++i) {
        in[i] = get_random_char();
    }
    proof_system::plonk::stdlib::packed_byte_array<Composer> input(&composer, in);
    proof_system::plonk::stdlib::sha256<Composer>(input);
}

Composer composers[NUM_HASHES];
Prover provers[NUM_HASHES];
Verifier verifiers[NUM_HASHES];
plonk::proof proofs[NUM_HASHES];

void construct_witnesses_bench(State& state) noexcept
{
    for (auto _ : state) {
        size_t idx = (static_cast<size_t>((state.range(0))) - START_BYTES) / BYTES_PER_CHUNK;
        composers[idx] = Composer();
        generate_test_plonk_circuit(composers[idx], static_cast<size_t>(state.range(0)));
    }
}
BENCHMARK(construct_witnesses_bench)->DenseRange(START_BYTES, MAX_BYTES, BYTES_PER_CHUNK);

void preprocess_witnesses_bench(State& state) noexcept
{
    for (auto _ : state) {
        size_t idx = (static_cast<size_t>((state.range(0))) - START_BYTES) / BYTES_PER_CHUNK;
        provers[idx] = composers[idx].create_prover();
        std::cout << "prover subgroup size = " << provers[idx].key->small_domain.size << std::endl;
        // printf("num bytes = %" PRIx64 ", num gates = %zu\n", state.range(0), composers[idx].get_num_gates());
    }
}
BENCHMARK(preprocess_witnesses_bench)->DenseRange(START_BYTES, MAX_BYTES, BYTES_PER_CHUNK);

void construct_instances_bench(State& state) noexcept
{
    for (auto _ : state) {
        size_t idx = (static_cast<size_t>((state.range(0))) - START_BYTES) / BYTES_PER_CHUNK;
        verifiers[idx] = composers[idx].create_verifier();
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
