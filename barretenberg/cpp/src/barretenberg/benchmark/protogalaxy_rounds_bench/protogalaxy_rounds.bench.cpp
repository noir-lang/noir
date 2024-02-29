#include <benchmark/benchmark.h>

#include "barretenberg/benchmark/ultra_bench/mock_proofs.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"
#include "barretenberg/ultra_honk/ultra_composer.hpp"

using namespace benchmark;

namespace bb {

template <typename Composer>
void _bench_round(::benchmark::State& state,
                  void (*F)(ProtoGalaxyProver_<ProverInstances_<typename Composer::Flavor, 2>>&))
{
    using Flavor = typename Composer::Flavor;
    using Builder = typename Flavor::CircuitBuilder;

    bb::srs::init_crs_factory("../srs_db/ignition");
    auto log2_num_gates = static_cast<size_t>(state.range(0));
    auto composer = Composer();

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

    auto prover_instance_1 = construct_instance();
    auto prover_instance_2 = construct_instance();

    auto folding_prover = composer.create_folding_prover({ prover_instance_1, prover_instance_2 });

    // prepare the prover state
    folding_prover.state.accumulator = prover_instance_1;
    folding_prover.state.deltas.resize(log2_num_gates);
    std::fill_n(folding_prover.state.deltas.begin(), log2_num_gates, 0);
    folding_prover.state.perturbator = Flavor::Polynomial::random(1 << log2_num_gates);
    folding_prover.transcript = Flavor::Transcript::prover_init_empty();
    folding_prover.preparation_round();

    for (auto _ : state) {
        F(folding_prover);
    }
}

void bench_round_ultra(::benchmark::State& state, void (*F)(ProtoGalaxyProver_<ProverInstances_<UltraFlavor, 2>>&))
{
    _bench_round<UltraComposer>(state, F);
}

void bench_round_goblin_ultra(::benchmark::State& state,
                              void (*F)(ProtoGalaxyProver_<ProverInstances_<GoblinUltraFlavor, 2>>&))
{
    _bench_round<GoblinUltraComposer>(state, F);
}

BENCHMARK_CAPTURE(bench_round_ultra, preparation, [](auto& prover) { prover.preparation_round(); })
    -> DenseRange(14, 20) -> Unit(kMillisecond);
BENCHMARK_CAPTURE(bench_round_ultra, perturbator, [](auto& prover) { prover.perturbator_round(); })
    -> DenseRange(14, 20) -> Unit(kMillisecond);
BENCHMARK_CAPTURE(bench_round_ultra, combiner_quotient, [](auto& prover) { prover.combiner_quotient_round(); })
    -> DenseRange(14, 20) -> Unit(kMillisecond);
BENCHMARK_CAPTURE(bench_round_ultra, accumulator_update, [](auto& prover) { prover.accumulator_update_round(); })
    -> DenseRange(14, 20) -> Unit(kMillisecond);

BENCHMARK_CAPTURE(bench_round_goblin_ultra, preparation, [](auto& prover) { prover.preparation_round(); })
    -> DenseRange(14, 20) -> Unit(kMillisecond);
BENCHMARK_CAPTURE(bench_round_goblin_ultra, perturbator, [](auto& prover) { prover.perturbator_round(); })
    -> DenseRange(14, 20) -> Unit(kMillisecond);
BENCHMARK_CAPTURE(bench_round_goblin_ultra, combiner_quotient, [](auto& prover) { prover.combiner_quotient_round(); })
    -> DenseRange(14, 20) -> Unit(kMillisecond);
BENCHMARK_CAPTURE(bench_round_goblin_ultra, accumulator_update, [](auto& prover) { prover.accumulator_update_round(); })
    -> DenseRange(14, 20) -> Unit(kMillisecond);

} // namespace bb

BENCHMARK_MAIN();
