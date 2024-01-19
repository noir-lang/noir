#include <benchmark/benchmark.h>

#include "barretenberg/benchmark/ultra_bench/benchmark_utilities.hpp"
#include "barretenberg/eccvm/eccvm_composer.hpp"
#include "barretenberg/proof_system/circuit_builder/eccvm/eccvm_circuit_builder.hpp"

using namespace benchmark;
using namespace bb;

using Flavor = honk::flavor::ECCVM;
using Builder = ECCVMCircuitBuilder<Flavor>;
using Composer = honk::ECCVMComposer;

namespace {

Builder generate_trace(size_t target_num_gates)
{
    Builder builder;
    using G1 = typename Flavor::CycleGroup;
    using Fr = typename G1::Fr;

    auto generators = G1::derive_generators("test generators", 2);

    typename G1::element a = generators[0];
    typename G1::element b = generators[1];
    Fr x = Fr::random_element();
    Fr y = Fr::random_element();

    typename G1::element expected_1 = (a * x) + a + a + (b * y) + (b * x) + (b * x);

    // Each loop adds 163 gates. Note: builder.get_num_gates() is very expensive here (bug?) and it's actually painful
    // to use a `while` loop
    size_t num_iterations = target_num_gates / 163;
    for (size_t _ = 0; _ < num_iterations; _++) {
        builder.add_accumulate(a);
        builder.mul_accumulate(a, x);
        builder.mul_accumulate(b, x);
        builder.mul_accumulate(b, y);
        builder.add_accumulate(a);
        builder.mul_accumulate(b, x);
        builder.eq_and_reset(expected_1);
    }
    return builder;
}

void eccvm_generate_prover(State& state) noexcept
{
    bb::srs::init_grumpkin_crs_factory("../srs_db/grumpkin");

    size_t target_num_gates = 1 << static_cast<size_t>(state.range(0));
    for (auto _ : state) {
        Builder builder = generate_trace(target_num_gates);
        Composer composer;
        auto prover = composer.create_prover(builder);
    };
}

void eccvm_prove(State& state) noexcept
{
    bb::srs::init_grumpkin_crs_factory("../srs_db/grumpkin");

    size_t target_num_gates = 1 << static_cast<size_t>(state.range(0));
    Builder builder = generate_trace(target_num_gates);
    Composer composer;
    auto prover = composer.create_prover(builder);
    for (auto _ : state) {
        auto proof = prover.construct_proof();
    };
}

BENCHMARK(eccvm_generate_prover)->Unit(kMillisecond)->DenseRange(10, 20);
BENCHMARK(eccvm_prove)->Unit(kMillisecond)->DenseRange(10, 20);
} // namespace