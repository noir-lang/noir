
#pragma once

#ifndef BB_USE_OP_COUNT
// require a semicolon to appease formatters
// NOLINTNEXTLINE(cppcoreguidelines-macro-usage)
#define BB_OP_COUNT_TRACK() (void)0
// NOLINTNEXTLINE(cppcoreguidelines-macro-usage)
#define BB_OP_COUNT_TRACK_NAME(name) (void)0
#else
/**
 * Provides an abstraction that counts operations based on function names.
 * For efficiency, we spread out counts across threads.
 */

#include "barretenberg/common/compiler_hints.hpp"
#include <algorithm>
#include <atomic>
#include <cstdlib>
#include <map>
#include <mutex>
#include <string>
#include <vector>
namespace bb::detail {
// Compile-time string
// See e.g. https://www.reddit.com/r/cpp_questions/comments/pumi9r/does_c20_not_support_string_literals_as_template/
template <std::size_t N> struct OperationLabel {
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays)
    constexpr OperationLabel(const char (&str)[N])
    {
        for (std::size_t i = 0; i < N; ++i) {
            value[i] = str[i];
        }
    }

    // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays)
    char value[N];
};

// Contains all statically known op counts
struct GlobalOpCountContainer {
  public:
    struct Entry {
        std::string key;
        std::string thread_id;
        std::size_t* count;
    };
    std::mutex mutex;
    std::vector<Entry> counts;
    void print() const;
    // NOTE: Should be called when other threads aren't active
    void clear();
    void add_entry(const char* key, std::size_t* count);
    std::map<std::string, std::size_t> get_aggregate_counts() const;
};

// NOLINTNEXTLINE(cppcoreguidelines-avoid-non-const-global-variables)
extern GlobalOpCountContainer GLOBAL_OP_COUNTS;

template <OperationLabel Op> struct GlobalOpCount {
  public:
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-non-const-global-variables)
    static thread_local std::size_t* thread_local_count;

    static constexpr void increment_op_count()
    {
        if (std::is_constant_evaluated()) {
            // We do nothing if the compiler tries to run this
            return;
        }
        if (BB_UNLIKELY(thread_local_count == nullptr)) {
            thread_local_count = new std::size_t();
            GLOBAL_OP_COUNTS.add_entry(Op.value, thread_local_count);
        }
        (*thread_local_count)++;
    }
};
// NOLINTNEXTLINE(cppcoreguidelines-avoid-non-const-global-variables)
template <OperationLabel Op> thread_local std::size_t* GlobalOpCount<Op>::thread_local_count;

} // namespace bb::detail

// NOLINTNEXTLINE(cppcoreguidelines-macro-usage)
#define BB_OP_COUNT_TRACK() bb::detail::GlobalOpCount<__func__>::increment_op_count()
// NOLINTNEXTLINE(cppcoreguidelines-macro-usage)
#define BB_OP_COUNT_TRACK_NAME(name) bb::detail::GlobalOpCount<name>::increment_op_count()
#endif