#include "barretenberg/common/throw_or_abort.hpp"
#ifndef NO_MULTITHREADING
#include "log.hpp"
#include "thread.hpp"
#include <atomic>
#include <condition_variable>
#include <functional>
#include <mutex>
#include <queue>
#include <thread>
#include <vector>

#include "barretenberg/common/compiler_hints.hpp"

namespace {

class ThreadPool {
  public:
    ThreadPool(size_t num_threads);
    ThreadPool(const ThreadPool& other) = delete;
    ThreadPool(ThreadPool&& other) = delete;
    ~ThreadPool();

    ThreadPool& operator=(const ThreadPool& other) = delete;
    ThreadPool& operator=(ThreadPool&& other) = delete;

    void start_tasks(size_t num_iterations, const std::function<void(size_t)>& func)
    {
        {
            std::unique_lock<std::mutex> lock(tasks_mutex);
            task_ = func;
            num_iterations_ = num_iterations;
            iteration_ = 0;
            complete_ = 0;
        }
        condition.notify_all();

        do_iterations();

        {
            std::unique_lock<std::mutex> lock(tasks_mutex);
            complete_condition_.wait(lock, [this] { return complete_ == num_iterations_; });
        }
    }

  private:
    std::vector<std::thread> workers;
    std::mutex tasks_mutex;
    std::function<void(size_t)> task_;
    size_t num_iterations_ = 0;
    size_t iteration_ = 0;
    size_t complete_ = 0;
    std::condition_variable condition;
    std::condition_variable complete_condition_;
    bool stop = false;

    BB_NO_PROFILE void worker_loop(size_t thread_index);

    void do_iterations()
    {
        while (true) {
            size_t iteration = 0;
            {
                std::unique_lock<std::mutex> lock(tasks_mutex);
                if (iteration_ == num_iterations_) {
                    return;
                }
                iteration = iteration_++;
            }
            task_(iteration);
            {
                std::unique_lock<std::mutex> lock(tasks_mutex);
                if (++complete_ == num_iterations_) {
                    complete_condition_.notify_one();
                    return;
                }
            }
        }
    }
};

ThreadPool::ThreadPool(size_t num_threads)
{
    workers.reserve(num_threads);
    for (size_t i = 0; i < num_threads; ++i) {
        workers.emplace_back(&ThreadPool::worker_loop, this, i);
    }
}

ThreadPool::~ThreadPool()
{
    {
        std::unique_lock<std::mutex> lock(tasks_mutex);
        stop = true;
    }
    condition.notify_all();
    for (auto& worker : workers) {
        worker.join();
    }
}

void ThreadPool::worker_loop(size_t /*unused*/)
{
    // info("created worker ", worker_num);
    while (true) {
        {
            std::unique_lock<std::mutex> lock(tasks_mutex);
            condition.wait(lock, [this] { return (iteration_ < num_iterations_) || stop; });

            if (stop) {
                break;
            }
        }
        do_iterations();
    }
    // info("worker exit ", worker_num);
}
} // namespace

namespace bb {
/**
 * A thread pooled strategy that uses std::mutex for protection. Each worker increments the "iteration" and processes.
 * The main thread acts as a worker also, and when it completes, it spins until thread workers are done.
 */
void parallel_for_mutex_pool(size_t num_iterations, const std::function<void(size_t)>& func)
{
    static ThreadPool pool(get_num_cpus() - 1);
    // Note that if this is used safely, we don't need the std::atomic_bool (can use bool), but if we are catching the
    // mess up case of nesting parallel_for this should be atomic
    static std::atomic_bool nested = false;
    // Check if we are already in a nested parallel_for_mutex_pool call
    bool expected = false;
    if (!nested.compare_exchange_strong(expected, true)) {
        throw_or_abort("Error: Nested parallel_for_mutex_pool calls are not allowed.");
    }
    // info("starting job with iterations: ", num_iterations);
    pool.start_tasks(num_iterations, func);
    // info("done");
    nested = false;
}
} // namespace bb
#endif