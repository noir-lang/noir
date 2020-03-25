#include "pedersen.hpp"
#include <benchmark/benchmark.h>
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/bn254/fr.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <plonk/composer/turbo_composer.hpp>
#include <plonk/reference_string/file_reference_string.hpp>
#include <stdlib/primitives/field/field.hpp>

#define BARRETENBERG_SRS_PATH "../srs_db"

using namespace benchmark;

constexpr size_t NUM_CIRCUITS = 10;

constexpr size_t get_circuit_size(const size_t target_count_base)
{
    constexpr size_t base_gates = 2;
    constexpr size_t gates_per_hash = 262;
    return (target_count_base - base_gates) / gates_per_hash;
}

constexpr size_t num_hashes[10]{
    get_circuit_size(1 << 11), get_circuit_size(1 << 12), get_circuit_size(1 << 13), get_circuit_size(1 << 14),
    get_circuit_size(1 << 15), get_circuit_size(1 << 16), get_circuit_size(1 << 17), get_circuit_size(1 << 18),
    get_circuit_size(1 << 19), get_circuit_size(1 << 20),
};

constexpr size_t get_index(const size_t target_count_base)
{
    for (size_t i = 0; i < 10; ++i) {
        if (target_count_base == num_hashes[i]) {
            return i;
        }
    }
    return 0;
}
void generate_test_pedersen_circuit(waffle::TurboComposer& turbo_composer, size_t num_repetitions)
{
    plonk::stdlib::field_t<waffle::TurboComposer> left(
        plonk::stdlib::witness_t(&turbo_composer, barretenberg::fr::random_element()));
    plonk::stdlib::field_t<waffle::TurboComposer> out(
        plonk::stdlib::witness_t(&turbo_composer, barretenberg::fr::random_element()));

    for (size_t i = 0; i < num_repetitions; ++i) {
        out = plonk::stdlib::pedersen::compress(left, out);
    }
}

waffle::TurboProver pedersen_provers[NUM_CIRCUITS];
waffle::TurboVerifier pedersen_verifiers[NUM_CIRCUITS];
waffle::plonk_proof pedersen_proofs[NUM_CIRCUITS];

grumpkin::fq pedersen_function(const size_t count)
{
    grumpkin::fq left = grumpkin::fq::random_element();
    grumpkin::fq out = grumpkin::fq::random_element();
    for (size_t i = 0; i < count; ++i) {
        out = crypto::pedersen::compress_native(left, out);
    }
    return out;
}
void native_pedersen_hash_bench(State& state) noexcept
{
    for (auto _ : state) {
        const size_t count = (static_cast<size_t>(state.range(0)));
        (pedersen_function(count));
    }
}
BENCHMARK(native_pedersen_hash_bench)
    ->Arg(num_hashes[0])
    ->Arg(num_hashes[1])
    ->Arg(num_hashes[2])
    ->Arg(num_hashes[3])
    ->Arg(num_hashes[4])
    ->Arg(num_hashes[5])
    ->Arg(num_hashes[6])
    ->Arg(num_hashes[7])
    ->Arg(num_hashes[8])
    ->Arg(num_hashes[9]);

void native_pedersen_eight_hash_bench(State& state) noexcept
{
    std::array<grumpkin::fq, 8> elements;
    for (size_t i = 0; i < 8; ++i) {
        elements[i] = grumpkin::fq::random_element();
    }
    for (auto _ : state) {
        crypto::pedersen::compress_eight_native(elements);
    }
}
BENCHMARK(native_pedersen_eight_hash_bench)->MinTime(3);

void construct_pedersen_witnesses_bench(State& state) noexcept
{
    for (auto _ : state) {
        waffle::TurboComposer composer =
            waffle::TurboComposer(BARRETENBERG_SRS_PATH, static_cast<size_t>(state.range(0)));
        generate_test_pedersen_circuit(composer, static_cast<size_t>(state.range(0)));
        printf("compoesr gates = %lx \n", composer.n);
        composer.compute_witness();
    }
}
BENCHMARK(construct_pedersen_witnesses_bench)
    ->Arg(num_hashes[0])
    ->Arg(num_hashes[1])
    ->Arg(num_hashes[2])
    ->Arg(num_hashes[3])
    ->Arg(num_hashes[4])
    ->Arg(num_hashes[5])
    ->Arg(num_hashes[6])
    ->Arg(num_hashes[7])
    ->Arg(num_hashes[8])
    ->Arg(num_hashes[9]);

void construct_pedersen_proving_keys_bench(State& state) noexcept
{
    for (auto _ : state) {
        waffle::TurboComposer composer =
            waffle::TurboComposer(BARRETENBERG_SRS_PATH, static_cast<size_t>(state.range(0)));
        generate_test_pedersen_circuit(composer, static_cast<size_t>(state.range(0)));
        size_t idx = get_index(static_cast<size_t>(state.range(0)));
        composer.compute_proving_key();
        state.PauseTiming();
        pedersen_provers[idx] = composer.create_prover();
        state.ResumeTiming();
    }
}
BENCHMARK(construct_pedersen_proving_keys_bench)
    ->Arg(num_hashes[0])
    ->Arg(num_hashes[1])
    ->Arg(num_hashes[2])
    ->Arg(num_hashes[3])
    ->Arg(num_hashes[4])
    ->Arg(num_hashes[5])
    ->Arg(num_hashes[6])
    ->Arg(num_hashes[7])
    ->Arg(num_hashes[8])
    ->Arg(num_hashes[9]);

void construct_pedersen_instances_bench(State& state) noexcept
{
    for (auto _ : state) {
        state.PauseTiming();
        waffle::TurboComposer composer =
            waffle::TurboComposer(BARRETENBERG_SRS_PATH, static_cast<size_t>(state.range(0)));
        generate_test_pedersen_circuit(composer, static_cast<size_t>(state.range(0)));
        size_t idx = get_index(static_cast<size_t>(state.range(0)));
        composer.create_prover();
        state.ResumeTiming();
        pedersen_verifiers[idx] = composer.create_verifier();
    }
}
BENCHMARK(construct_pedersen_instances_bench)
    ->Arg(num_hashes[0])
    ->Arg(num_hashes[1])
    ->Arg(num_hashes[2])
    ->Arg(num_hashes[3])
    ->Arg(num_hashes[4])
    ->Arg(num_hashes[5])
    ->Arg(num_hashes[6])
    ->Arg(num_hashes[7])
    ->Arg(num_hashes[8])
    ->Arg(num_hashes[9]);

void construct_pedersen_proofs_bench(State& state) noexcept
{
    for (auto _ : state) {
        size_t idx = get_index(static_cast<size_t>(state.range(0)));
        pedersen_proofs[idx] = pedersen_provers[idx].construct_proof();
        state.PauseTiming();
        pedersen_provers[idx].reset();
        state.ResumeTiming();
    }
}
BENCHMARK(construct_pedersen_proofs_bench)
    ->Arg(num_hashes[0])
    ->Arg(num_hashes[1])
    ->Arg(num_hashes[2])
    ->Arg(num_hashes[3])
    ->Arg(num_hashes[4])
    ->Arg(num_hashes[5])
    ->Arg(num_hashes[6])
    ->Arg(num_hashes[7])
    ->Arg(num_hashes[8])
    ->Arg(num_hashes[9]);

void verify_pedersen_proofs_bench(State& state) noexcept
{
    for (auto _ : state) {
        size_t idx = get_index(static_cast<size_t>(state.range(0)));
        pedersen_verifiers[idx].verify_proof(pedersen_proofs[idx]);
    }
}
BENCHMARK(verify_pedersen_proofs_bench)
    ->Arg(num_hashes[0])
    ->Arg(num_hashes[1])
    ->Arg(num_hashes[2])
    ->Arg(num_hashes[3])
    ->Arg(num_hashes[4])
    ->Arg(num_hashes[5])
    ->Arg(num_hashes[6])
    ->Arg(num_hashes[7])
    ->Arg(num_hashes[8])
    ->Arg(num_hashes[9]);

BENCHMARK_MAIN();
