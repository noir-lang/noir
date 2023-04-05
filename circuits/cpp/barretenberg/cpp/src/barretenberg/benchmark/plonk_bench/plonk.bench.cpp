#include <benchmark/benchmark.h>
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"
#include "barretenberg/plonk/composer/standard_composer.hpp"
#include "barretenberg/plonk/proof_system/prover/prover.hpp"
#include "barretenberg/plonk/proof_system/verifier/verifier.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"

using namespace benchmark;

constexpr size_t MAX_GATES = 1 << 20;
constexpr size_t NUM_CIRCUITS = 10;
constexpr size_t START = (MAX_GATES) >> (NUM_CIRCUITS - 1);
// constexpr size_t NUM_HASH_CIRCUITS = 8;
// constexpr size_t MAX_HASH_ROUNDS = 8192;
// constexpr size_t START_HASH_ROUNDS = 64;

void generate_test_plonk_circuit(plonk::StandardComposer& composer, size_t num_gates)
{
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

plonk::Prover provers[NUM_CIRCUITS];
plonk::Verifier verifiers[NUM_CIRCUITS];
plonk::proof proofs[NUM_CIRCUITS];

void construct_witnesses_bench(State& state) noexcept
{
    for (auto _ : state) {
        state.PauseTiming();
        plonk::StandardComposer composer = proof_system::plonk::StandardComposer(static_cast<size_t>(state.range(0)));
        generate_test_plonk_circuit(composer, static_cast<size_t>(state.range(0)));
        composer.compute_proving_key();
        state.ResumeTiming();

        composer.compute_witness();
    }
}
BENCHMARK(construct_witnesses_bench)->RangeMultiplier(2)->Range(START, MAX_GATES);

void construct_proving_keys_bench(State& state) noexcept
{
    for (auto _ : state) {
        plonk::StandardComposer composer = proof_system::plonk::StandardComposer(static_cast<size_t>(state.range(0)));
        generate_test_plonk_circuit(composer, static_cast<size_t>(state.range(0)));
        size_t idx = static_cast<size_t>(numeric::get_msb((uint64_t)state.range(0))) -
                     static_cast<size_t>(numeric::get_msb(START));
        composer.compute_proving_key();
        state.PauseTiming();
        provers[idx] = composer.create_prover();
        state.ResumeTiming();
    }
}
BENCHMARK(construct_proving_keys_bench)->RangeMultiplier(2)->Range(START, MAX_GATES);

void construct_instances_bench(State& state) noexcept
{
    for (auto _ : state) {
        state.PauseTiming();
        plonk::StandardComposer composer = proof_system::plonk::StandardComposer(static_cast<size_t>(state.range(0)));
        generate_test_plonk_circuit(composer, static_cast<size_t>(state.range(0)));
        size_t idx = static_cast<size_t>(numeric::get_msb((uint64_t)state.range(0))) -
                     static_cast<size_t>(numeric::get_msb(START));
        composer.create_prover();
        state.ResumeTiming();
        verifiers[idx] = composer.create_verifier();
    }
}
BENCHMARK(construct_instances_bench)->RangeMultiplier(2)->Range(START, MAX_GATES);

void construct_proofs_bench(State& state) noexcept
{
    for (auto _ : state) {
        size_t idx = static_cast<size_t>(numeric::get_msb((uint64_t)state.range(0))) -
                     static_cast<size_t>(numeric::get_msb(START));
        // provers[idx].reset();
        proofs[idx] = provers[idx].construct_proof();
        state.PauseTiming();
        provers[idx].reset();
        state.ResumeTiming();
    }
}
BENCHMARK(construct_proofs_bench)->RangeMultiplier(2)->Range(START, MAX_GATES);

void verify_proofs_bench(State& state) noexcept
{
    for (auto _ : state) {
        size_t idx = static_cast<size_t>(numeric::get_msb((uint64_t)state.range(0))) -
                     static_cast<size_t>(numeric::get_msb(START));
        verifiers[idx].verify_proof(proofs[idx]);
        state.PauseTiming();
        // if (!result)
        // {
        //     printf("hey! proof isn't valid!\n");
        // }
        state.ResumeTiming();
    }
}
BENCHMARK(verify_proofs_bench)->RangeMultiplier(2)->Range(START, MAX_GATES);

BENCHMARK_MAIN();
