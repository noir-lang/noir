#ifndef NO_MULTITHREADING
#include "log.hpp"
#include "thread.hpp"
#include "thread_pool.hpp"
#include <atomic>
#include <condition_variable>
#include <functional>
#include <mutex>
#include <queue>
#include <thread>
#include <vector>

namespace bb {
/**
 * A thread pooled strategey that assumed that thread pools would be more efficient than spawning threads.
 * Every iteration becomes a task in a queue. That's probably not very efficient.
 * A normal mutex and condition variables are used to distribute tasks and notify.
 */
void parallel_for_queued(size_t num_iterations, const std::function<void(size_t)>& func)
{
    static ThreadPool pool(get_num_cpus());

    // info("wait for pool enter");
    pool.wait();
    for (size_t i = 0; i < num_iterations; ++i) {
        // info("enqueing iteration ", i);
        pool.enqueue([=]() { func(i); });
    }
    // info("wait for pool exit");
    pool.wait();
    // info("pool finished work");
}
} // namespace bb
#endif