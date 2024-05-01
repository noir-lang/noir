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
            iterations_completed_ = 0;
        }
        condition.notify_all();

        do_iterations();

        while (iterations_completed_ != num_iterations) {
        }
    }

  private:
    std::vector<std::thread> workers;
    std::mutex tasks_mutex;
    std::function<void(size_t)> task_;
    size_t num_iterations_ = 0;
    std::atomic<size_t> iteration_;
    std::atomic<size_t> iterations_completed_;
    std::condition_variable condition;
    bool stop = false;

    void worker_loop(size_t thread_index);

    void do_iterations()
    {
        size_t iteration = 0;
        while ((iteration = iteration_.fetch_add(1, std::memory_order_seq_cst)) < num_iterations_) {
            // info("main thread processing iteration ", iteration);
            task_(iteration);
            iterations_completed_++;
        }
    }
};

ThreadPool::ThreadPool(size_t num_threads)
    : iteration_(0)
    , iterations_completed_(0)
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
 * A thread pooled strategy that uses atomics to prevent needing constantly lock on a queue.
 * The main thread acts as a worker also, and when it completes, it spins until thread workers are done.
 */
void parallel_for_atomic_pool(size_t num_iterations, const std::function<void(size_t)>& func)
{
    static ThreadPool pool(get_num_cpus() - 1);

    // info("starting job with iterations: ", num_iterations);
    pool.start_tasks(num_iterations, func);
    // info("done");
}
} // namespace bb
#endif