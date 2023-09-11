#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/polynomials/barycentric.hpp"
#include <benchmark/benchmark.h>

using namespace benchmark;

namespace {
auto& engine = numeric::random::get_debug_engine();
}

using FF = barretenberg::fr;
using barretenberg::BarycentricData;
using barretenberg::Univariate;

namespace proof_system::benchmark {

void extend_2_to_6(State& state) noexcept
{
    auto univariate = Univariate<FF, 2>::get_random();
    BarycentricData<FF, 2, 6> barycentric_2_to_6;
    for (auto _ : state) {
        DoNotOptimize(barycentric_2_to_6.extend(univariate));
    }
}
BENCHMARK(extend_2_to_6);

} // namespace proof_system::benchmark