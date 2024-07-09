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
#define AVM_TRACK_TIME(key, body) ::bb::avm_trace::Stats::get().time(key, [&]() { body; });
#else
#define AVM_TRACK_TIME(key, body) body
#endif

namespace bb::avm_trace {

class Stats {
  public:
    static Stats& get();
    void reset();
    void increment(const std::string& key, uint64_t value);
    void time(const std::string& key, std::function<void()> f);
    std::string to_string() const;
    std::string aggregate_to_string(const std::string& key_prefix) const;

  private:
    Stats() = default;

    std::unordered_map<std::string, uint64_t> stats;
    mutable std::mutex stats_mutex;
};

} // namespace bb::avm_trace