#include "barretenberg/benchmark/honk_bench/benchmark_utilities.hpp"
#include "barretenberg/honk/flavor/goblin_ultra.hpp"
#include "barretenberg/honk/flavor/ultra.hpp"
#include "barretenberg/plonk/composer/standard_composer.hpp"
#include "barretenberg/plonk/composer/ultra_composer.hpp"
#include "barretenberg/plonk/proof_system/widgets/transition_widgets/plookup_auxiliary_widget.hpp"
#include <benchmark/benchmark.h>

namespace {
auto& engine = numeric::random::get_debug_engine();
}

namespace proof_system::plonk {

template <typename Flavor, typename Widget> void execute_widget(::benchmark::State& state)
{
    barretenberg::srs::init_crs_factory("../srs_db/ignition");
    auto inner_composer = plonk::UltraComposer();
    auto builder = typename plonk::UltraComposer::CircuitBuilder();
    bench_utils::generate_basic_arithmetic_circuit(builder, 80);
    auto inner_prover = inner_composer.create_prover(builder);
    auto inner_proof = inner_prover.construct_proof();
    for (auto _ : state) {
        Widget widget(inner_composer.circuit_proving_key.get());
        widget.compute_quotient_contribution(barretenberg::fr::random_element(), inner_prover.transcript);
    }
}

void plookup_auxiliary_widget(::benchmark::State& state) noexcept
{
    execute_widget<honk::flavor::Ultra, ProverPlookupAuxiliaryWidget<ultra_settings>>(state);
}
BENCHMARK(plookup_auxiliary_widget);

} // namespace proof_system::plonk
