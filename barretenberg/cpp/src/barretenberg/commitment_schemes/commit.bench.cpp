
#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/srs/factories/mem_bn254_crs_factory.hpp"
#include <benchmark/benchmark.h>

namespace bb {

template <typename Curve> std::shared_ptr<CommitmentKey<Curve>> create_commitment_key(const size_t num_points)
{
    std::string srs_path;
    if constexpr (std::same_as<Curve, curve::BN254>) {
        srs_path = "../srs_db/ignition";
    } else {
        static_assert(std::same_as<Curve, curve::Grumpkin>);
        srs_path = "../srs_db/grumpkin";
    }
    auto crs_factory = std::make_shared<bb::srs::factories::FileCrsFactory<Curve>>(srs_path, num_points);
    return std::make_shared<CommitmentKey<Curve>>(num_points, crs_factory);
}

constexpr size_t MAX_LOG_NUM_POINTS = 24;
constexpr size_t MAX_NUM_POINTS = 1 << MAX_LOG_NUM_POINTS;

auto key = create_commitment_key<curve::BN254>(MAX_NUM_POINTS);

template <typename Curve> void bench_commit(::benchmark::State& state)
{
    const size_t num_points = 1 << state.range(0);
    const auto polynomial = Polynomial<typename Curve::ScalarField>(num_points);
    for (auto _ : state) {
        benchmark::DoNotOptimize(key->commit(polynomial));
    }
}

BENCHMARK(bench_commit<curve::BN254>)->DenseRange(10, MAX_LOG_NUM_POINTS)->Unit(benchmark::kMillisecond);

} // namespace bb

BENCHMARK_MAIN();
