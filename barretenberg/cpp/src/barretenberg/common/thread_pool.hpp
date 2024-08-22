#pragma once

#ifndef NO_MULTITHREADING

#include <atomic>
#include <condition_variable>
#include <functional>
#include <mutex>
#include <queue>
#include <thread>
#include <vector>
namespace bb {
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
    size_t num_threads() { return workers.size(); };

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
} // namespace bb

#endif
