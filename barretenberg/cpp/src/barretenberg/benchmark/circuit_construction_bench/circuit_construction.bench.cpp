
#include <benchmark/benchmark.h>

#include "barretenberg/stdlib/primitives/biggroup/biggroup.hpp"
#include "barretenberg/stdlib/primitives/curves/bn254.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_circuit_builder.hpp"

using namespace benchmark;
using namespace bb;

namespace {

auto& engine = numeric::get_debug_randomness();
void biggroup_construction_bench(State& state)
{
    using Curve = stdlib::bn254<UltraCircuitBuilder>;
    using affine_element = Curve::AffineElementNative;
    using element_ct = Curve::Element;
    using scalar_ct = Curve::ScalarField;
    for (auto _ : state) {
        state.PauseTiming();

        UltraCircuitBuilder builder;
        size_t num_points = static_cast<size_t>(state.range(0));
        std::vector<affine_element> points;
        std::vector<fr> scalars;
        for (size_t i = 0; i < num_points; ++i) {
            points.push_back(affine_element(Curve::ElementNative::random_element()));
            scalars.push_back(fr::random_element());
        }

        std::vector<element_ct> circuit_points;
        std::vector<scalar_ct> circuit_scalars;
        for (size_t i = 0; i < num_points; ++i) {
            circuit_points.push_back(element_ct::from_witness(&builder, points[i]));
            circuit_scalars.push_back(scalar_ct::from_witness(&builder, scalars[i]));
        }
        state.ResumeTiming();
        element_ct::batch_mul(circuit_points, circuit_scalars);
        state.PauseTiming();
    }
}
} // namespace
BENCHMARK(biggroup_construction_bench)->Unit(kMicrosecond)->DenseRange(2, 20);

BENCHMARK_MAIN();
