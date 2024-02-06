
#include <cstddef>
#ifdef BB_USE_OP_COUNT
#include "op_count.hpp"
#include <iostream>
#include <sstream>
#include <thread>

namespace bb::detail {
void GlobalOpCountContainer::add_entry(const char* key, std::size_t* count)
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
        if (*entry.count > 0) {
            std::cout << entry.key << "\t" << *entry.count << "\t[thread=" << entry.thread_id << "]" << std::endl;
        }
    }
    std::cout << "print_op_counts() END" << std::endl;
}

std::map<std::string, std::size_t> GlobalOpCountContainer::get_aggregate_counts() const
{
    std::map<std::string, std::size_t> aggregate_counts;
    for (const Entry& entry : counts) {
        if (*entry.count > 0) {
            aggregate_counts[entry.key] += *entry.count;
        }
    }
    return aggregate_counts;
}

void GlobalOpCountContainer::clear()
{
    std::unique_lock<std::mutex> lock(mutex);
    for (Entry& entry : counts) {
        *entry.count = 0;
    }
}

// NOLINTNEXTLINE(cppcoreguidelines-avoid-non-const-global-variables)
GlobalOpCountContainer GLOBAL_OP_COUNTS;
} // namespace bb::detail
#endif
