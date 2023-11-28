#include "./poseidon2.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include <benchmark/benchmark.h>

using namespace benchmark;

grumpkin::fq poseidon_function(const size_t count)
{
    std::vector<grumpkin::fq> inputs(count);
    for (size_t i = 0; i < count; ++i) {
        inputs[i] = grumpkin::fq::random_element();
    }
    std::span tmp(inputs);
    // hash count many field elements
    inputs[0] = crypto::Poseidon2<crypto::Poseidon2Bn254ScalarFieldParams>::hash(tmp);
    return inputs[0];
}

void native_poseidon2_commitment_bench(State& state) noexcept
{
    for (auto _ : state) {
        const size_t count = (static_cast<size_t>(state.range(0)));
        (poseidon_function(count));
    }
}
BENCHMARK(native_poseidon2_commitment_bench)->Arg(10)->Arg(1000)->Arg(10000);

BENCHMARK_MAIN();
// } // namespace crypto