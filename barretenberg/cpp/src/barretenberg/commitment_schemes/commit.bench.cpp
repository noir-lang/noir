
#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/srs/factories/mem_bn254_crs_factory.hpp"
#include <benchmark/benchmark.h>

namespace bb {

template <typename Curve> std::shared_ptr<CommitmentKey<Curve>> create_commitment_key(const size_t num_points)
{
    bb::srs::init_crs_factory("../srs_db/ignition");
    std::string srs_path;
    return std::make_shared<CommitmentKey<Curve>>(num_points);
}

// Generate a polynomial with a specified number of nonzero random coefficients
template <typename FF> Polynomial<FF> sparse_random_poly(const size_t size, const size_t num_nonzero)
{
    auto& engine = numeric::get_debug_randomness();
    auto polynomial = Polynomial<FF>(size);

    for (size_t i = 0; i < num_nonzero; i++) {
        size_t idx = engine.get_random_uint32() % size;
        polynomial[idx] = FF::random_element();
    }

    return polynomial;
}

constexpr size_t MIN_LOG_NUM_POINTS = 16;
constexpr size_t MAX_LOG_NUM_POINTS = 20;
constexpr size_t MAX_NUM_POINTS = 1 << MAX_LOG_NUM_POINTS;
constexpr size_t SPARSE_NUM_NONZERO = 100;

// Commit to a zero polynomial
template <typename Curve> void bench_commit_zero(::benchmark::State& state)
{
    auto key = create_commitment_key<Curve>(MAX_NUM_POINTS);

    const size_t num_points = 1 << state.range(0);
    const auto polynomial = Polynomial<typename Curve::ScalarField>(num_points);
    for (auto _ : state) {
        key->commit(polynomial);
    }
}

// Commit to a polynomial with sparse nonzero entries equal to 1
template <typename Curve> void bench_commit_sparse(::benchmark::State& state)
{
    using Fr = typename Curve::ScalarField;
    auto key = create_commitment_key<Curve>(MAX_NUM_POINTS);

    const size_t num_points = 1 << state.range(0);
    const size_t num_nonzero = SPARSE_NUM_NONZERO;

    auto polynomial = Polynomial<Fr>(num_points);
    for (size_t i = 0; i < num_nonzero; i++) {
        polynomial[i] = 1;
    }

    for (auto _ : state) {
        key->commit(polynomial);
    }
}

// Commit to a polynomial with sparse nonzero entries equal to 1 using the commit_sparse method to preprocess the input
template <typename Curve> void bench_commit_sparse_preprocessed(::benchmark::State& state)
{
    using Fr = typename Curve::ScalarField;
    auto key = create_commitment_key<Curve>(MAX_NUM_POINTS);

    const size_t num_points = 1 << state.range(0);
    const size_t num_nonzero = SPARSE_NUM_NONZERO;

    auto polynomial = Polynomial<Fr>(num_points);
    for (size_t i = 0; i < num_nonzero; i++) {
        polynomial[i] = 1;
    }

    for (auto _ : state) {
        key->commit_sparse(polynomial);
    }
}

// Commit to a polynomial with sparse random nonzero entries
template <typename Curve> void bench_commit_sparse_random(::benchmark::State& state)
{
    using Fr = typename Curve::ScalarField;
    auto key = create_commitment_key<Curve>(MAX_NUM_POINTS);

    const size_t num_points = 1 << state.range(0);
    const size_t num_nonzero = SPARSE_NUM_NONZERO;

    auto polynomial = sparse_random_poly<Fr>(num_points, num_nonzero);

    for (auto _ : state) {
        key->commit(polynomial);
    }
}

// Commit to a polynomial with sparse random nonzero entries using the commit_sparse method to preprocess the input
template <typename Curve> void bench_commit_sparse_random_preprocessed(::benchmark::State& state)
{
    using Fr = typename Curve::ScalarField;
    auto key = create_commitment_key<Curve>(MAX_NUM_POINTS);

    const size_t num_points = 1 << state.range(0);
    const size_t num_nonzero = SPARSE_NUM_NONZERO;

    auto polynomial = sparse_random_poly<Fr>(num_points, num_nonzero);

    for (auto _ : state) {
        key->commit_sparse(polynomial);
    }
}

// Commit to a polynomial with dense random nonzero entries
template <typename Curve> void bench_commit_random(::benchmark::State& state)
{
    using Fr = typename Curve::ScalarField;
    auto key = create_commitment_key<Curve>(MAX_NUM_POINTS);

    const size_t num_points = 1 << state.range(0);
    auto polynomial = Polynomial<Fr>(num_points);
    for (auto& coeff : polynomial) {
        coeff = Fr::random_element();
    }
    for (auto _ : state) {
        key->commit(polynomial);
    }
}

BENCHMARK(bench_commit_zero<curve::BN254>)
    ->DenseRange(MIN_LOG_NUM_POINTS, MAX_LOG_NUM_POINTS)
    ->Unit(benchmark::kMillisecond);
BENCHMARK(bench_commit_sparse<curve::BN254>)
    ->DenseRange(MIN_LOG_NUM_POINTS, MAX_LOG_NUM_POINTS)
    ->Unit(benchmark::kMillisecond);
BENCHMARK(bench_commit_sparse_preprocessed<curve::BN254>)
    ->DenseRange(MIN_LOG_NUM_POINTS, MAX_LOG_NUM_POINTS)
    ->Unit(benchmark::kMillisecond);
BENCHMARK(bench_commit_sparse_random<curve::BN254>)
    ->DenseRange(MIN_LOG_NUM_POINTS, MAX_LOG_NUM_POINTS)
    ->Unit(benchmark::kMillisecond);
BENCHMARK(bench_commit_sparse_random_preprocessed<curve::BN254>)
    ->DenseRange(MIN_LOG_NUM_POINTS, MAX_LOG_NUM_POINTS)
    ->Unit(benchmark::kMillisecond);
BENCHMARK(bench_commit_random<curve::BN254>)
    ->DenseRange(MIN_LOG_NUM_POINTS, MAX_LOG_NUM_POINTS)
    ->Unit(benchmark::kMillisecond);

} // namespace bb

BENCHMARK_MAIN();
