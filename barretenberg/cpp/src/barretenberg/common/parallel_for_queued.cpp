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

    void enqueue(const std::function<void()>& task);
    void wait();

  private:
    std::vector<std::thread> workers;
    std::queue<std::function<void()>> tasks;
    std::mutex tasks_mutex;
    std::condition_variable condition;
    std::condition_variable finished_condition;
    std::atomic<size_t> tasks_running;
    bool stop = false;

    void worker_loop(size_t thread_index);
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

void ThreadPool::enqueue(const std::function<void()>& task)
{
    {
        std::unique_lock<std::mutex> lock(tasks_mutex);
        tasks.push(task);
    }
    condition.notify_one();
}

void ThreadPool::wait()
{
    std::unique_lock<std::mutex> lock(tasks_mutex);
    finished_condition.wait(lock, [this] { return tasks.empty() && tasks_running == 0; });
}

void ThreadPool::worker_loop(size_t /*unused*/)
{
    // info("created worker ", worker_num);
    while (true) {
        std::function<void()> task;
        {
            std::unique_lock<std::mutex> lock(tasks_mutex);
            condition.wait(lock, [this] { return !tasks.empty() || stop; });

            if (tasks.empty() && stop) {
                break;
            }

            task = tasks.front();
            tasks.pop();
            tasks_running++;
        }
        // info("worker ", worker_num, " processing a task!");
        task();
        // info("task done");
        {
            std::unique_lock<std::mutex> lock(tasks_mutex);
            tasks_running--;
            if (tasks.empty() && tasks_running == 0) {
                // info("notifying main thread");
                finished_condition.notify_all();
            }
        }
        // info("worker ", worker_num, " done!");
    }
    // info("worker exit ", worker_num);
}
} // namespace

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