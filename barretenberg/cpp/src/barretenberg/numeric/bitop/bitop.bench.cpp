#include "count_leading_zeros.hpp"
#include <benchmark/benchmark.h>

using namespace benchmark;

void count_leading_zeros(State& state) noexcept
{
    uint256_t input = 7;
    for (auto _ : state) {
        auto r = count_leading_zeros(input);
        DoNotOptimize(r);
    }
}
BENCHMARK(count_leading_zeros);

// NOLINTNEXTLINE macro invokation triggers style errors from googletest code
BENCHMARK_MAIN();
