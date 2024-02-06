
#pragma once
#include <benchmark/benchmark.h>

#ifndef BB_USE_OP_COUNT
namespace bb {
struct GoogleBenchOpCountReporter {
    GoogleBenchOpCountReporter(::benchmark::State& state)
    {
        // unused, we don't have op counts on
        (void)state;
    }
};
}; // namespace bb
// require a semicolon to appease formatters
#define BB_REPORT_OP_COUNT_IN_BENCH(state) (void)0
#define BB_REPORT_OP_COUNT_BENCH_CANCEL() (void)0
#else
#include "op_count.hpp"
namespace bb {
// NOLINTNEXTLINE(cppcoreguidelines-special-member-functions)
struct GoogleBenchOpCountReporter {
    // We allow having a ref member as this only lives inside a function frame
    ::benchmark::State& state;
    bool cancelled = false;
    GoogleBenchOpCountReporter(::benchmark::State& state)
        : state(state)
    {
        // Intent: Clear when we enter the state loop
        bb::detail::GLOBAL_OP_COUNTS.clear();
    }
    ~GoogleBenchOpCountReporter()
    {
        // Allow for conditional reporting
        if (cancelled) {
            return;
        }
        // Intent: Collect results when we exit the state loop
        for (auto& entry : bb::detail::GLOBAL_OP_COUNTS.get_aggregate_counts()) {
            state.counters[entry.first] = static_cast<double>(entry.second);
        }
    }
};
// Allow for integration with google benchmark user-defined counters
// NOLINTNEXTLINE(cppcoreguidelines-macro-usage)
#define BB_REPORT_OP_COUNT_IN_BENCH(state) GoogleBenchOpCountReporter __bb_report_op_count_in_bench{ state };
// NOLINTNEXTLINE(cppcoreguidelines-macro-usage)
#define BB_REPORT_OP_COUNT_BENCH_CANCEL() __bb_report_op_count_in_bench.cancelled = true;
}; // namespace bb
#endif