#pragma once
#include "barretenberg/env/logstr.hpp"
#include "barretenberg/stdlib/primitives/circuit_builders/circuit_builders_fwd.hpp"
#include <algorithm>
#include <sstream>
#include <string>
#include <vector>

#define BENCHMARK_INFO_PREFIX "##BENCHMARK_INFO_PREFIX##"
#define BENCHMARK_INFO_SEPARATOR "#"
#define BENCHMARK_INFO_SUFFIX "##BENCHMARK_INFO_SUFFIX##"

template <typename... Args> std::string format(Args... args)
{
    std::ostringstream os;
    ((os << args), ...);
    return os.str();
}

template <typename T> void benchmark_format_chain(std::ostream& os, T const& first)
{
    // We will be saving these values to a CSV file, so we can't tolerate commas
    std::stringstream current_argument;
    current_argument << first;
    std::string current_argument_string = current_argument.str();
    std::replace(current_argument_string.begin(), current_argument_string.end(), ',', ';');
    os << current_argument_string << BENCHMARK_INFO_SUFFIX;
}

template <typename T, typename... Args>
void benchmark_format_chain(std::ostream& os, T const& first, Args const&... args)
{
    // We will be saving these values to a CSV file, so we can't tolerate commas
    std::stringstream current_argument;
    current_argument << first;
    std::string current_argument_string = current_argument.str();
    std::replace(current_argument_string.begin(), current_argument_string.end(), ',', ';');
    os << current_argument_string << BENCHMARK_INFO_SEPARATOR;
    benchmark_format_chain(os, args...);
}

template <typename... Args> std::string benchmark_format(Args... args)
{
    std::ostringstream os;
    os << BENCHMARK_INFO_PREFIX;
    benchmark_format_chain(os, args...);
    return os.str();
}

#ifndef NDEBUG
template <typename... Args> inline void debug(Args... args)
{
    logstr(format(args...).c_str());
}
#else
template <typename... Args> inline void debug(Args... /*unused*/) {}
#endif

template <typename... Args> inline void info(Args... args)
{
    logstr(format(args...).c_str());
}

extern bool verbose_logging;
template <typename... Args> inline void vinfo(Args... args)
{
    if (verbose_logging) {
        info(args...);
    }
}

template <typename... Args> inline void important(Args... args)
{
    logstr(format("important: ", args...).c_str());
}

/**
 * @brief Info used to store circuit statistics during CI/CD with concrete structure. Writes straight to log
 *
 * @details Automatically appends the necessary prefix and suffix,  as well as separators.
 *
 * @tparam Args
 * @param args
 */
#ifdef CI
template <typename Arg1, typename Arg2, typename Arg3, typename Arg4, typename Arg5>
inline void benchmark_info(Arg1 composer, Arg2 class_name, Arg3 operation, Arg4 metric, Arg5 value)
{
    logstr(benchmark_format(composer, class_name, operation, metric, value).c_str());
}
#else
template <typename... Args> inline void benchmark_info(Args... /*unused*/) {}
#endif

/**
 * @brief A class for saving benchmarks and printing them all at once in the end of the function.
 *
 */
class BenchmarkInfoCollator {

    std::vector<std::string> saved_benchmarks;

  public:
    BenchmarkInfoCollator() = default;
    BenchmarkInfoCollator(const BenchmarkInfoCollator& other) = default;
    BenchmarkInfoCollator(BenchmarkInfoCollator&& other) = default;
    BenchmarkInfoCollator& operator=(const BenchmarkInfoCollator& other) = default;
    BenchmarkInfoCollator& operator=(BenchmarkInfoCollator&& other) = default;

/**
 * @brief Info used to store circuit statistics during CI/CD with concrete structure. Stores string in vector for now
 * (used to flush all benchmarks at the end of test).
 *
 * @details Automatically appends the necessary prefix and suffix,  as well as separators.
 *
 * @tparam Args
 * @param args
 */
#ifdef CI
    template <typename Arg1, typename Arg2, typename Arg3, typename Arg4, typename Arg5>
    inline void benchmark_info_deferred(Arg1 composer, Arg2 class_name, Arg3 operation, Arg4 metric, Arg5 value)
    {
        saved_benchmarks.push_back(benchmark_format(composer, class_name, operation, metric, value).c_str());
    }
#else
    explicit BenchmarkInfoCollator(std::vector<std::string> saved_benchmarks)
        : saved_benchmarks(std::move(saved_benchmarks))
    {}
    template <typename... Args> inline void benchmark_info_deferred(Args... /*unused*/) {}
#endif
    ~BenchmarkInfoCollator()
    {
        for (auto& x : saved_benchmarks) {
            logstr(x.c_str());
        }
    }
};
