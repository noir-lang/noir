#include "barretenberg/commitment_schemes/ipa/ipa.hpp"
#include <benchmark/benchmark.h>

using namespace benchmark;
using namespace bb;

namespace {
using Curve = curve::Grumpkin;
using Fr = Curve::ScalarField;

constexpr size_t MIN_POLYNOMIAL_DEGREE_LOG2 = 10;
constexpr size_t MAX_POLYNOMIAL_DEGREE_LOG2 = 16;
std::shared_ptr<bb::srs::factories::CrsFactory<curve::Grumpkin>> crs_factory(
    new bb::srs::factories::FileCrsFactory<curve::Grumpkin>("../srs_db/grumpkin", 1 << 16));

auto ck = std::make_shared<CommitmentKey<Curve>>(1 << MAX_POLYNOMIAL_DEGREE_LOG2);
auto vk = std::make_shared<VerifierCommitmentKey<Curve>>(1 << MAX_POLYNOMIAL_DEGREE_LOG2, crs_factory);

std::vector<std::shared_ptr<NativeTranscript>> prover_transcripts(MAX_POLYNOMIAL_DEGREE_LOG2 -
                                                                  MIN_POLYNOMIAL_DEGREE_LOG2 + 1);
std::vector<OpeningClaim<Curve>> opening_claims(MAX_POLYNOMIAL_DEGREE_LOG2 - MIN_POLYNOMIAL_DEGREE_LOG2 + 1);

void ipa_open(State& state) noexcept
{
    numeric::RNG& engine = numeric::get_debug_randomness();
    for (auto _ : state) {
        state.PauseTiming();
        size_t n = 1 << static_cast<size_t>(state.range(0));
        // Construct the polynomial
        Polynomial<Fr> poly(n);
        for (size_t i = 0; i < n; ++i) {
            poly[i] = Fr::random_element(&engine);
        }
        auto x = Fr::random_element(&engine);
        auto eval = poly.evaluate(x);
        const OpeningPair<Curve> opening_pair = { x, eval };
        const OpeningClaim<Curve> opening_claim{ opening_pair, ck->commit(poly) };
        // initialize empty prover transcript
        auto prover_transcript = std::make_shared<NativeTranscript>();
        state.ResumeTiming();
        // Compute proof
        IPA<Curve>::compute_opening_proof(ck, opening_pair, poly, prover_transcript);
        // Store info for verifier
        prover_transcripts[static_cast<size_t>(state.range(0)) - MIN_POLYNOMIAL_DEGREE_LOG2] = prover_transcript;
        opening_claims[static_cast<size_t>(state.range(0)) - MIN_POLYNOMIAL_DEGREE_LOG2] = opening_claim;
    }
}
void ipa_verify(State& state) noexcept
{
    for (auto _ : state) {
        state.PauseTiming();
        // Retrieve proofs
        auto prover_transcript = prover_transcripts[static_cast<size_t>(state.range(0)) - MIN_POLYNOMIAL_DEGREE_LOG2];
        auto opening_claim = opening_claims[static_cast<size_t>(state.range(0)) - MIN_POLYNOMIAL_DEGREE_LOG2];
        // initialize verifier transcript from proof data
        auto verifier_transcript = std::make_shared<NativeTranscript>(prover_transcript->proof_data);

        state.ResumeTiming();
        auto result = IPA<Curve>::verify(vk, opening_claim, verifier_transcript);
        ASSERT(result);
    }
}
} // namespace
BENCHMARK(ipa_open)->Unit(kMillisecond)->DenseRange(MIN_POLYNOMIAL_DEGREE_LOG2, MAX_POLYNOMIAL_DEGREE_LOG2);
BENCHMARK(ipa_verify)->Unit(kMillisecond)->DenseRange(MIN_POLYNOMIAL_DEGREE_LOG2, MAX_POLYNOMIAL_DEGREE_LOG2);
BENCHMARK_MAIN();