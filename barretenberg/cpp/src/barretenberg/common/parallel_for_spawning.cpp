#ifndef NO_MULTITHREADING
#include "thread.hpp"
#include <thread>

namespace bb {
/**
 * A very simple strategy. Spawn a worker thread for every iteration (but no more than num cores).
 * Worker threads tight-loop incrementing an atomic variable from 0-num_iterations, until num_iterations reached.
 * Main thread waits on all worker threads by joining.
 */
void parallel_for_spawning(size_t num_iterations, const std::function<void(size_t)>& func)
{
    std::atomic<size_t> current_iteration(0);

    auto worker = [&](size_t) {
        // info("entered worker: ", thread_index);
        size_t index = 0;
        while ((index = current_iteration.fetch_add(1, std::memory_order_seq_cst)) < num_iterations) {
            func(index);
        }
        // info("exited worker: ", thread_index);
    };

    auto num_threads = std::min(num_iterations, get_num_cpus()) - 1;
    // if (num_threads == 1) {
    //     // info("Executing on main thread as only 1 cpu or iteration. iterations: ", num_iterations);
    //     worker(0);
    //     return;
    // }
    // info("Starting ", num_threads, " threads to handle ", num_iterations, " iterations.");

    std::vector<std::thread> threads(num_threads);

    for (size_t i = 0; i < num_threads; ++i) {
        threads[i] = std::thread(worker, i);
    }

    worker(num_threads);

    for (auto& thread : threads) {
        thread.join();
    }
    // info("joined!\n\n");
}
} // namespace bb
#endif