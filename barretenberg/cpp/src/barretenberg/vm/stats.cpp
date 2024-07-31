#include "barretenberg/vm/stats.hpp"

#include <chrono>
#include <cstdint>
#include <string>
#include <vector>

namespace bb::avm_trace {

Stats& Stats::get()
{
    static Stats stats;
    return stats;
}

void Stats::reset()
{
    stats.clear();
}

void Stats::increment(const std::string& key, uint64_t value)
{
    std::lock_guard lock(stats_mutex);
    stats[key] += value;
}

void Stats::time(const std::string& key, const std::function<void()>& f)
{
    auto start = std::chrono::system_clock::now();
    f();
    auto elapsed = std::chrono::system_clock::now() - start;
    increment(key + "_ms",
              static_cast<uint64_t>(std::chrono::duration_cast<std::chrono::milliseconds>(elapsed).count()));
}

std::string Stats::to_string(int depth) const
{
    std::lock_guard lock(stats_mutex);

    std::vector<std::string> result;
    result.reserve(stats.size());
    for (const auto& [key, value] : stats) {
        if (std::count(key.begin(), key.end(), '/') < depth) {
            result.push_back(key + ": " + std::to_string(value));
        }
    }
    std::sort(result.begin(), result.end());
    std::string joined;
    for (auto& s : result) {
        joined += std::move(s) + "\n";
    }
    return joined;
}

} // namespace bb::avm_trace