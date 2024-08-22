
#ifndef NO_MULTITHREADING

#include "thread_pool.hpp"
#include "barretenberg/common/log.hpp"
namespace bb {

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
} // namespace bb

#endif
