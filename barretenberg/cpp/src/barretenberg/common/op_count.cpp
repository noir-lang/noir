
#include <cstddef>
#ifdef BB_USE_OP_COUNT
#include "op_count.hpp"
#include <iostream>
#include <sstream>
#include <thread>

namespace bb::detail {

GlobalOpCountContainer::~GlobalOpCountContainer()
{
    // This is useful for printing counts at the end of non-benchmarks.
    // See op_count_google_bench.hpp for benchmarks.
    // print();
}

void GlobalOpCountContainer::add_entry(const char* key, const std::shared_ptr<OpStats>& count)
{
    std::unique_lock<std::mutex> lock(mutex);
    std::stringstream ss;
    ss << std::this_thread::get_id();
    counts.push_back({ key, ss.str(), count });
}

void GlobalOpCountContainer::print() const
{
    std::cout << "print_op_counts() START" << std::endl;
    for (const Entry& entry : counts) {
        if (entry.count->count > 0) {
            std::cout << entry.key << "\t" << entry.count->count << "\t[thread=" << entry.thread_id << "]" << std::endl;
        }
        if (entry.count->time > 0) {
            std::cout << entry.key << "(t)\t" << static_cast<double>(entry.count->time) / 1000000.0
                      << "ms\t[thread=" << entry.thread_id << "]" << std::endl;
        }
        if (entry.count->cycles > 0) {
            std::cout << entry.key << "(c)\t" << entry.count->cycles << "\t[thread=" << entry.thread_id << "]"
                      << std::endl;
        }
    }
    std::cout << "print_op_counts() END" << std::endl;
}

std::map<std::string, std::size_t> GlobalOpCountContainer::get_aggregate_counts() const
{
    std::map<std::string, std::size_t> aggregate_counts;
    for (const Entry& entry : counts) {
        if (entry.count->count > 0) {
            aggregate_counts[entry.key] += entry.count->count;
        }
        if (entry.count->time > 0) {
            aggregate_counts[entry.key + "(t)"] += entry.count->time;
        }
        if (entry.count->cycles > 0) {
            aggregate_counts[entry.key + "(c)"] += entry.count->cycles;
        }
    }
    return aggregate_counts;
}

void GlobalOpCountContainer::clear()
{
    std::unique_lock<std::mutex> lock(mutex);
    for (Entry& entry : counts) {
        *entry.count = OpStats();
    }
}

// NOLINTNEXTLINE(cppcoreguidelines-avoid-non-const-global-variables)
GlobalOpCountContainer GLOBAL_OP_COUNTS;

OpCountCycleReporter::OpCountCycleReporter(OpStats* stats)
    : stats(stats)
{
#if __clang__ && (defined(__x86_64__) || defined(_M_X64) || defined(__i386) || defined(_M_IX86))
    // Don't support any other targets but x86 clang for now, this is a bit lazy but more than fits our needs
    cycles = __builtin_ia32_rdtsc();
#endif
}
OpCountCycleReporter::~OpCountCycleReporter()
{
#if __clang__ && (defined(__x86_64__) || defined(_M_X64) || defined(__i386) || defined(_M_IX86))
    // Don't support any other targets but x86 clang for now, this is a bit lazy but more than fits our needs
    stats->count += 1;
    stats->cycles += __builtin_ia32_rdtsc() - cycles;
#endif
}
OpCountTimeReporter::OpCountTimeReporter(OpStats* stats)
    : stats(stats)
{
    auto now = std::chrono::high_resolution_clock::now();
    auto now_ns = std::chrono::time_point_cast<std::chrono::nanoseconds>(now);
    time = static_cast<std::size_t>(now_ns.time_since_epoch().count());
}
OpCountTimeReporter::~OpCountTimeReporter()
{
    auto now = std::chrono::high_resolution_clock::now();
    auto now_ns = std::chrono::time_point_cast<std::chrono::nanoseconds>(now);
    stats->count += 1;
    stats->time += static_cast<std::size_t>(now_ns.time_since_epoch().count()) - time;
}
} // namespace bb::detail
#endif
