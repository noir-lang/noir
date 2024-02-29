#include <benchmark/benchmark.h>

#include "barretenberg/benchmark/ultra_bench/mock_proofs.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"
#include "barretenberg/ultra_honk/ultra_composer.hpp"

using namespace benchmark;

namespace bb {

// Fold one instance into an accumulator.
template <typename Composer> void fold_one(State& state) noexcept
{
    using Flavor = typename Composer::Flavor;
    using Instance = ProverInstance_<Flavor>;
    using Instances = ProverInstances_<Flavor, 2>;
    using ProtoGalaxyProver = ProtoGalaxyProver_<Instances>;
    using Builder = typename Flavor::CircuitBuilder;

    bb::srs::init_crs_factory("../srs_db/ignition");

    auto log2_num_gates = static_cast<size_t>(state.range(0));
    Composer composer;

    const auto construct_instance = [&]() {
        Builder builder;
        if constexpr (std::same_as<Flavor, GoblinUltraFlavor>) {
            GoblinMockCircuits::construct_arithmetic_circuit(builder, log2_num_gates);
        } else {
            static_assert(std::same_as<Flavor, UltraFlavor>);
            bb::mock_proofs::generate_basic_arithmetic_circuit(builder, log2_num_gates);
        }
        return composer.create_prover_instance(builder);
    };

    std::shared_ptr<Instance> instance_1 = construct_instance();
    std::shared_ptr<Instance> instance_2 = construct_instance();

    ProtoGalaxyProver folding_prover = composer.create_folding_prover({ instance_1, instance_2 });

    for (auto _ : state) {
        auto proof = folding_prover.fold_instances();
    }
}

BENCHMARK(fold_one<UltraComposer>)->/* vary the circuit size */ DenseRange(14, 20)->Unit(kMillisecond);
BENCHMARK(fold_one<GoblinUltraComposer>)->/* vary the circuit size */ DenseRange(14, 20)->Unit(kMillisecond);
} // namespace bb

BENCHMARK_MAIN();
