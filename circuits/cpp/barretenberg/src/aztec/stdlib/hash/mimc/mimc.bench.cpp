#include <benchmark/benchmark.h>
#include <ecc/curves/bn254/fr.hpp>
#include <stdlib/primitives/field/field.hpp>
#include <plonk/composer/mimc_composer.hpp>
#include "mimc.hpp"

using namespace benchmark;

constexpr size_t MAX_REPETITIONS = 2400;
constexpr size_t NUM_CIRCUITS = 12;

void generate_test_mimc_circuit(waffle::MiMCComposer& mimc_composer, size_t num_repetitions)
{
    plonk::stdlib::field_t<waffle::MiMCComposer> mimc_input(
        plonk::stdlib::witness_t(&mimc_composer, barretenberg::fr::random_element()));
    plonk::stdlib::field_t<waffle::MiMCComposer> mimc_k(
        plonk::stdlib::witness_t(&mimc_composer, barretenberg::fr::zero()));
    plonk::stdlib::field_t<waffle::MiMCComposer> mimc_output(&mimc_composer);

    for (size_t i = 0; i < num_repetitions; ++i) {
        plonk::stdlib::mimc_block_cipher(mimc_input, mimc_k);
    }
}

waffle::Prover mimc_provers[NUM_CIRCUITS];
waffle::Verifier mimc_verifiers[NUM_CIRCUITS];
waffle::plonk_proof mimc_proofs[NUM_CIRCUITS];

void construct_mimc_witnesses_bench(State& state) noexcept
{
    for (auto _ : state) {
        waffle::MiMCComposer composer = waffle::MiMCComposer(static_cast<size_t>(state.range(0)));
        generate_test_mimc_circuit(composer, static_cast<size_t>(state.range(0)));
        size_t idx = ((static_cast<size_t>((state.range(0))) - static_cast<size_t>(200)) / 200);
        mimc_provers[idx] = composer.preprocess();
    }
}
BENCHMARK(construct_mimc_witnesses_bench)->DenseRange(200, MAX_REPETITIONS, 200);

void construct_mimc_instances_bench(State& state) noexcept
{
    for (auto _ : state) {
        waffle::MiMCComposer composer = waffle::MiMCComposer(static_cast<size_t>(state.range(0)));
        generate_test_mimc_circuit(composer, static_cast<size_t>(state.range(0)));
        size_t idx = ((static_cast<size_t>((state.range(0))) - static_cast<size_t>(200)) / 200);
        mimc_verifiers[idx] = composer.create_verifier();
    }
}
BENCHMARK(construct_mimc_instances_bench)->DenseRange(200, MAX_REPETITIONS, 200);

void construct_mimc_proofs_bench(State& state) noexcept
{
    for (auto _ : state) {
        size_t idx = ((static_cast<size_t>((state.range(0))) - static_cast<size_t>(200)) / 200);
        mimc_proofs[idx] = mimc_provers[idx].construct_proof();
        state.PauseTiming();
        mimc_provers[idx].reset();
        state.ResumeTiming();
    }
}
BENCHMARK(construct_mimc_proofs_bench)->DenseRange(200, MAX_REPETITIONS, 200);

void verify_mimc_proofs_bench(State& state) noexcept
{
    for (auto _ : state) {
        size_t idx = ((static_cast<size_t>((state.range(0))) - static_cast<size_t>(200)) / 200);
        bool result = mimc_verifiers[idx].verify_proof(mimc_proofs[idx]);
        state.PauseTiming();
        if (!result) {
            printf("hey! proof isn't valid!\n");
        }
        state.ResumeTiming();
    }
}
BENCHMARK(verify_mimc_proofs_bench)->DenseRange(200, MAX_REPETITIONS, 200);

BENCHMARK_MAIN();
