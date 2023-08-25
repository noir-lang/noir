#include "barretenberg/common/thread.hpp"
#include "log.hpp"
#include "moody/blockingconcurrentqueue.h"
#include "timer.hpp"
#include <atomic>
#include <condition_variable>
#include <functional>
#include <mutex>
#include <queue>
#include <thread>
#include <vector>

class ThreadPool {
  public:
    ThreadPool(size_t num_threads)
        : tasks(1024)
        , complete_queue_(1)
    {
        workers.reserve(num_threads);
        for (size_t i = 0; i < num_threads; ++i) {
            workers.emplace_back(&ThreadPool::worker_loop, this, i);
        }
    }

    ~ThreadPool()
    {
        stop = true;
        for (size_t i = 0; i < workers.size(); ++i) {
            tasks.enqueue([]() {});
        }
        for (auto& worker : workers) {
            worker.join();
        }
    }

    ThreadPool(const ThreadPool& other) = delete;
    ThreadPool(ThreadPool&& other) = delete;
    ThreadPool& operator=(const ThreadPool& other) = delete;
    ThreadPool& operator=(ThreadPool&& other) = delete;

    void start_tasks(const std::function<void(size_t)>& task, size_t num_iterations)
    {
        std::atomic<size_t> complete_counter;
        //  3rd party library expects c-style array as input. Boo.
        // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays)
        std::function<void()> funcs[num_iterations];
        for (size_t i = 0; i < num_iterations; ++i) {
            funcs[i] = [&, i]() {
                // Timer t;
                task(i);
                // info("task took: ", t.nanoseconds());
                if (complete_counter.fetch_add(1, std::memory_order_relaxed) == num_iterations - 1) {
                    // info("iteration ", i, " was the last");
                    complete_queue_.enqueue(true);
                }
            };
        }
        tasks.enqueue_bulk(funcs, num_iterations);

        {
            std::function<void()> task;
            while (tasks.try_dequeue(task)) {
                task();
            }
        }

        bool complete = false;
        complete_queue_.wait_dequeue(complete);
        // info("all done!");
    }

  private:
    std::vector<std::thread> workers;
    moodycamel::BlockingConcurrentQueue<std::function<void()>> tasks;
    moodycamel::BlockingConcurrentQueue<bool> complete_queue_;
    std::atomic<bool> stop = false;

    void worker_loop(size_t /*unused*/)
    {
        // info("worker started");
        while (!stop) {
            std::function<void()> task;
            tasks.wait_dequeue(task);
            task();
        }
    }
};

/**
 * A Thread pooled strategy that uses a popular lock-free multiple-producer multiple-consume queue library by
 * "moodycamel" as the underlying mechanism to distribute work and join on completion.
 */
void parallel_for_moody(size_t num_iterations, const std::function<void(size_t)>& func)
{
    // -1 because main thread works.
    static ThreadPool pool(get_num_cpus() - 1);

    pool.start_tasks(func, num_iterations);
}
