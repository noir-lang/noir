#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include <benchmark/benchmark.h>

using namespace benchmark;

namespace {
auto& engine = bb::numeric::get_debug_randomness();
}

using FF = bb::fr;
using bb::BarycentricData;
using bb::Univariate;

namespace bb::benchmark {

void extend_2_to_6(State& state) noexcept
{
    auto univariate = Univariate<FF, 2>::get_random();
    for (auto _ : state) {
        DoNotOptimize(univariate.extend_to<6>());
    }
}
BENCHMARK(extend_2_to_6);

} // namespace bb::benchmark