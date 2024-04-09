#include <benchmark/benchmark.h>

#include "barretenberg/common/op_count_google_bench.hpp"
#include "barretenberg/protogalaxy/protogalaxy_prover.hpp"
#include "barretenberg/stdlib_circuit_builders/mock_circuits.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_circuit_builder.hpp"
#include "barretenberg/sumcheck/instance/instances.hpp"
#include "barretenberg/sumcheck/instance/prover_instance.hpp"

using namespace benchmark;

namespace bb {

// Fold one instance into an accumulator.
template <typename Flavor, size_t k> void fold_k(State& state) noexcept
{
    using ProverInstance = ProverInstance_<Flavor>;
    using Instance = ProverInstance;
    using Instances = ProverInstances_<Flavor, k + 1>;
    using ProtoGalaxyProver = ProtoGalaxyProver_<Instances>;
    using Builder = typename Flavor::CircuitBuilder;

    bb::srs::init_crs_factory("../srs_db/ignition");

    auto log2_num_gates = static_cast<size_t>(state.range(0));

    const auto construct_instance = [&]() {
        Builder builder;
        MockCircuits::construct_arithmetic_circuit(builder, log2_num_gates);
        return std::make_shared<ProverInstance>(builder);
    };
    std::vector<std::shared_ptr<Instance>> instances;
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/938): Parallelize this loop
    for (size_t i = 0; i < k + 1; ++i) {
        instances.emplace_back(construct_instance());
    }

    ProtoGalaxyProver folding_prover(instances);

    for (auto _ : state) {
        BB_REPORT_OP_COUNT_IN_BENCH(state);
        auto proof = folding_prover.fold_instances();
    }
}

BENCHMARK(fold_k<UltraFlavor, 1>)->/* vary the circuit size */ DenseRange(14, 20)->Unit(kMillisecond);
BENCHMARK(fold_k<GoblinUltraFlavor, 1>)->/* vary the circuit size */ DenseRange(14, 20)->Unit(kMillisecond);

BENCHMARK(fold_k<UltraFlavor, 2>)->/* vary the circuit size */ DenseRange(14, 20)->Unit(kMillisecond);
BENCHMARK(fold_k<GoblinUltraFlavor, 2>)->/* vary the circuit size */ DenseRange(14, 20)->Unit(kMillisecond);

BENCHMARK(fold_k<UltraFlavor, 3>)->/* vary the circuit size */ DenseRange(14, 20)->Unit(kMillisecond);
BENCHMARK(fold_k<GoblinUltraFlavor, 3>)->/* vary the circuit size */ DenseRange(14, 20)->Unit(kMillisecond);

} // namespace bb

BENCHMARK_MAIN();
