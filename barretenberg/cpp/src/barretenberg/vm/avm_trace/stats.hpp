#pragma once

#include <cstdint>
#include <functional>
#include <mutex>
#include <string>
#include <unordered_map>

// To enable stats tracking, compile in RelWithAssert mode.
// cmake --preset $PRESET -DCMAKE_BUILD_TYPE=RelWithAssert
#ifndef NDEBUG
#define AVM_TRACK_STATS
#endif

#ifdef AVM_TRACK_STATS
// For tracking time spent in a block of code.
#define AVM_TRACK_TIME(key, body) ::bb::avm_trace::Stats::get().time(key, [&]() { body; });
// For tracking time spent in a block of code and returning a value.
#define AVM_TRACK_TIME_V(key, body) ::bb::avm_trace::Stats::get().template time_r(key, [&]() { return body; });
#else
#define AVM_TRACK_TIME(key, body) body
#define AVM_TRACK_TIME_V(key, body) body
#endif

namespace bb::avm_trace {

class Stats {
  public:
    static Stats& get();
    void reset();
    void increment(const std::string& key, uint64_t value);
    void time(const std::string& key, const std::function<void()>& f);

    template <typename F> auto time_r(const std::string& key, F&& f)
    {
        auto start = std::chrono::system_clock::now();
        auto result = f();
        auto elapsed = std::chrono::system_clock::now() - start;
        increment(key + "_ms",
                  static_cast<uint64_t>(std::chrono::duration_cast<std::chrono::milliseconds>(elapsed).count()));
        return result;
    }

    // Returns a string representation of the stats.
    // E.g., if depth = 2, it will show the top 2 levels of the stats.
    // That is, prove/logderiv_ms will be shown but
    // prove/logderiv/relation_ms will not be shown.
    std::string to_string(int depth = 2) const;

  private:
    Stats() = default;

    std::unordered_map<std::string, uint64_t> stats;
    mutable std::mutex stats_mutex;
};

} // namespace bb::avm_trace