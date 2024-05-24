#include "barretenberg/benchmark/ultra_bench/mock_circuits.hpp"
#include "barretenberg/plonk/composer/standard_composer.hpp"
#include "barretenberg/plonk/composer/ultra_composer.hpp"
#include "barretenberg/plonk/proof_system/widgets/random_widgets/permutation_widget.hpp"
#include "barretenberg/plonk/proof_system/widgets/random_widgets/plookup_widget.hpp"
#include "barretenberg/plonk/proof_system/widgets/transition_widgets/elliptic_widget.hpp"
#include "barretenberg/plonk/proof_system/widgets/transition_widgets/genperm_sort_widget.hpp"
#include "barretenberg/plonk/proof_system/widgets/transition_widgets/plookup_arithmetic_widget.hpp"
#include "barretenberg/plonk/proof_system/widgets/transition_widgets/plookup_auxiliary_widget.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_flavor.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_flavor.hpp"

// The widgets are implemented in a non-uniform way where the transition widgets provide a per-row execution function
// `accumulate_contribution` while the random widgets do not. Defining this preprocessor variable allows to derive a
// per-row exeuction cost that is suitable for comparing against the cost of executing the Honk relations. For
// validation, we also directly benchmark the available `accumulate_contribution` functions.
//
// NOTE: this code is to be run singly threaded via taskset, e.g. taskset -c 0
// #define GET_PER_ROW_TIME

namespace {
auto& engine = bb::numeric::get_debug_randomness();
}

namespace bb::plonk {

#ifdef GET_PER_ROW_TIME
constexpr size_t LARGE_DOMAIN_SIZE = 4;
constexpr size_t WIDGET_BENCH_TEST_CIRCUIT_SIZE = 1 << 16;
#endif

struct BasicPlonkKeyAndTranscript {
    std::shared_ptr<proving_key> key;
    transcript::StandardTranscript transcript;
};

BasicPlonkKeyAndTranscript get_plonk_key_and_transcript()
{
    bb::srs::init_crs_factory("../srs_db/ignition");
    auto inner_composer = plonk::UltraComposer();
    auto builder = typename plonk::UltraComposer::CircuitBuilder();
    bb::mock_circuits::generate_basic_arithmetic_circuit(builder, 16);
    UltraProver inner_prover = inner_composer.create_prover(builder);
#ifdef GET_PER_ROW_TIME
    if (!(inner_prover.key->circuit_size == WIDGET_BENCH_TEST_CIRCUIT_SIZE)) {
        throw_or_abort("Circit size changed; update value for accurate benchmarks");
    }
#endif
    inner_prover.construct_proof();
    return { inner_composer.circuit_proving_key, inner_prover.transcript };
}

template <typename Flavor, typename Widget> void execute_widget(::benchmark::State& state)
{
    BasicPlonkKeyAndTranscript data = get_plonk_key_and_transcript();
    Widget widget(data.key);
    for (auto _ : state) {
        widget.compute_quotient_contribution(bb::fr::random_element(), data.transcript);
    }
}

template <typename Widget> void quotient_contribution(::benchmark::State& state) noexcept
{
    BasicPlonkKeyAndTranscript data = get_plonk_key_and_transcript();
    Widget widget(data.key.get());
    for (auto _ : state) {
#ifdef GET_PER_ROW_TIME
        auto start = std::chrono::high_resolution_clock::now();
#endif
        widget.compute_quotient_contribution(bb::fr::random_element(), data.transcript);
#ifdef GET_PER_ROW_TIME
        auto end = std::chrono::high_resolution_clock::now();
        auto elapsed_seconds = std::chrono::duration_cast<std::chrono::duration<double>>(end - start);
        state.SetIterationTime(elapsed_seconds.count() / (LARGE_DOMAIN_SIZE * WIDGET_BENCH_TEST_CIRCUIT_SIZE));
#endif
    }
}

#ifdef GET_PER_ROW_TIME
BENCHMARK(quotient_contribution<ProverPlookupArithmeticWidget<ultra_settings>>)->Iterations(1)->UseManualTime();
BENCHMARK(quotient_contribution<ProverGenPermSortWidget<ultra_settings>>)->Iterations(1)->UseManualTime();
BENCHMARK(quotient_contribution<ProverEllipticWidget<ultra_settings>>)->Iterations(1)->UseManualTime();
BENCHMARK(quotient_contribution<ProverPlookupAuxiliaryWidget<ultra_settings>>)->Iterations(1)->UseManualTime();
BENCHMARK(quotient_contribution<ProverPlookupWidget<4>>)->Iterations(1)->UseManualTime();
BENCHMARK(quotient_contribution<ProverPermutationWidget<4, true>>)->Iterations(1)->UseManualTime();
#else
BENCHMARK(quotient_contribution<ProverPlookupArithmeticWidget<ultra_settings>>)->Iterations(1);
BENCHMARK(quotient_contribution<ProverGenPermSortWidget<ultra_settings>>)->Iterations(1);
BENCHMARK(quotient_contribution<ProverEllipticWidget<ultra_settings>>)->Iterations(1);
BENCHMARK(quotient_contribution<ProverPlookupAuxiliaryWidget<ultra_settings>>)->Iterations(1);
BENCHMARK(quotient_contribution<ProverPlookupWidget<4>>)->Iterations(1);
BENCHMARK(quotient_contribution<ProverPermutationWidget<4, true>>)->Iterations(1);
#endif

template <typename Widget> void accumulate_contribution(::benchmark::State& state) noexcept
{
    BasicPlonkKeyAndTranscript data = get_plonk_key_and_transcript();

    using FFTGetter = typename Widget::FFTGetter;
    using FFTKernel = typename Widget::FFTKernel;

    auto polynomials = FFTGetter::get_polynomials(data.key.get(), FFTKernel::get_required_polynomial_ids());
    auto challenges =
        FFTGetter::get_challenges(data.transcript, bb::fr::random_element(), FFTKernel::quotient_required_challenges);

    for (auto _ : state) {
        bb::fr result{ 0 };
        FFTKernel::accumulate_contribution(polynomials, challenges, result, 0);
    }
}
BENCHMARK(accumulate_contribution<ProverPlookupArithmeticWidget<ultra_settings>>);
BENCHMARK(accumulate_contribution<ProverGenPermSortWidget<ultra_settings>>);
BENCHMARK(accumulate_contribution<ProverEllipticWidget<ultra_settings>>);
BENCHMARK(accumulate_contribution<ProverPlookupAuxiliaryWidget<ultra_settings>>);

} // namespace bb::plonk

BENCHMARK_MAIN();
