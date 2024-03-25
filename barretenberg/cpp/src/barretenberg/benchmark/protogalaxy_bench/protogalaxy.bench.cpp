#include <benchmark/benchmark.h>

#include "barretenberg/protogalaxy/protogalaxy_prover.hpp"
#include "barretenberg/stdlib_circuit_builders/mock_circuits.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_circuit_builder.hpp"
#include "barretenberg/sumcheck/instance/instances.hpp"
#include "barretenberg/sumcheck/instance/prover_instance.hpp"

using namespace benchmark;

namespace bb {

// Fold one instance into an accumulator.
template <typename Flavor> void fold_one(State& state) noexcept
{
    using ProverInstance = ProverInstance_<Flavor>;
    using Instance = ProverInstance;
    using Instances = ProverInstances_<Flavor, 2>;
    using ProtoGalaxyProver = ProtoGalaxyProver_<Instances>;
    using Builder = typename Flavor::CircuitBuilder;

    bb::srs::init_crs_factory("../srs_db/ignition");

    auto log2_num_gates = static_cast<size_t>(state.range(0));

    const auto construct_instance = [&]() {
        Builder builder;
        MockCircuits::construct_arithmetic_circuit(builder, log2_num_gates);
        return std::make_shared<ProverInstance>(builder);
    };

    std::shared_ptr<Instance> instance_1 = construct_instance();
    std::shared_ptr<Instance> instance_2 = construct_instance();

    ProtoGalaxyProver folding_prover({ instance_1, instance_2 });

    for (auto _ : state) {
        auto proof = folding_prover.fold_instances();
    }
}

BENCHMARK(fold_one<UltraFlavor>)->/* vary the circuit size */ DenseRange(14, 20)->Unit(kMillisecond);
BENCHMARK(fold_one<GoblinUltraFlavor>)->/* vary the circuit size */ DenseRange(14, 20)->Unit(kMillisecond);
} // namespace bb

BENCHMARK_MAIN();
