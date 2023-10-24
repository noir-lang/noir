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

struct BasicPlonkKeyAndTranscript {
    std::shared_ptr<proving_key> key;
    transcript::StandardTranscript transcript;
};

BasicPlonkKeyAndTranscript get_plonk_key_and_transcript()
{
    barretenberg::srs::init_crs_factory("../srs_db/ignition");
    auto inner_composer = plonk::UltraComposer();
    auto builder = typename plonk::UltraComposer::CircuitBuilder();
    bench_utils::generate_basic_arithmetic_circuit(builder, 80);
    UltraProver inner_prover = inner_composer.create_prover(builder);
    inner_prover.construct_proof();
    return { inner_composer.circuit_proving_key, inner_prover.transcript };
}

template <typename Flavor, typename Widget> void execute_widget(::benchmark::State& state)
{
    BasicPlonkKeyAndTranscript data = get_plonk_key_and_transcript();
    Widget widget(data.key);
    for (auto _ : state) {
        widget.compute_quotient_contribution(barretenberg::fr::random_element(), data.transcript);
    }
}
void plookup_auxiliary_kernel(::benchmark::State& state) noexcept
{
    BasicPlonkKeyAndTranscript data = get_plonk_key_and_transcript();

    using FFTGetter = ProverPlookupAuxiliaryWidget<ultra_settings>::FFTGetter;
    using FFTKernel = ProverPlookupAuxiliaryWidget<ultra_settings>::FFTKernel;

    auto polynomials = FFTGetter::get_polynomials(data.key.get(), FFTKernel::get_required_polynomial_ids());
    auto challenges = FFTGetter::get_challenges(
        data.transcript, barretenberg::fr::random_element(), FFTKernel::quotient_required_challenges);

    for (auto _ : state) {
        // NOTE: this simply calls the following 3 functions it does NOT try to replicate ProverPlookupAuxiliaryWidget
        // logic exactly
        widget::containers::coefficient_array<barretenberg::fr> linear_terms;
        FFTKernel::compute_linear_terms(polynomials, challenges, linear_terms, 0);
        barretenberg::fr sum_of_linear_terms = FFTKernel::sum_linear_terms(polynomials, challenges, linear_terms, 0);
        FFTKernel::compute_non_linear_terms(polynomials, challenges, sum_of_linear_terms, 0);
    }
}
BENCHMARK(plookup_auxiliary_kernel);

void plookup_auxiliary_widget(::benchmark::State& state) noexcept
{
    BasicPlonkKeyAndTranscript data = get_plonk_key_and_transcript();
    ProverPlookupAuxiliaryWidget<ultra_settings> widget(data.key.get());
    for (auto _ : state) {
        widget.compute_quotient_contribution(barretenberg::fr::random_element(), data.transcript);
    }
}
BENCHMARK(plookup_auxiliary_widget);

} // namespace proof_system::plonk
