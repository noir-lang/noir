#include "./c_bind.hpp"
#include "./mem.hpp"
#include "./serialize.hpp"
#include "./slab_allocator.hpp"
#include "./timer.hpp"
#include <algorithm>

#ifndef NO_MULTITHREADING
#include <thread>

struct test_threads_data {
    std::atomic<uint32_t> counter = 0;
    size_t iterations = 0;
};

void thread_test_entry_point(test_threads_data* v)
{
    Timer t;
    // info("thread start with counter at: ", static_cast<size_t>(v->counter));
    std::vector<uint8_t> data(1024);
    for (size_t i = 0; i < v->iterations; ++i) {
        // Do some meaningless work.
        std::for_each(data.begin(), data.end(), [](auto& i) { i = i & 0x80; });
        std::for_each(data.begin(), data.end(), [](auto& i) { i = i | 0x01; });
        std::for_each(data.begin(), data.end(), [](auto& i) { i = static_cast<uint8_t>(i << 3); });
        (v->counter)++;
    }
    // info("thread end with counter at: ", static_cast<size_t>(v->counter), " ", t.seconds(), "s");
}

void thread_test_abort_entry_point(void* /*unused*/)
{
    info("thread_test_abort aborting");
    std::abort();
}

WASM_EXPORT void test_threads(uint32_t const* thread_num, uint32_t const* iterations, uint32_t* out)
{
    info("test starting...");
    Timer t;
    size_t NUM_THREADS = ntohl(*thread_num);
    std::vector<std::thread> threads(NUM_THREADS);
    test_threads_data test_data;
    test_data.iterations = ntohl(*iterations) / NUM_THREADS;

    for (size_t i = 0; i < NUM_THREADS; i++) {
        threads[i] = std::thread(thread_test_entry_point, &test_data);
    }

    info("joining...");
    for (size_t i = 0; i < NUM_THREADS; i++) {
        threads[i].join();
    }

    info("test complete with counter at: ", static_cast<size_t>(test_data.counter), " ", t.seconds(), "s");
    *out = htonl(test_data.counter);
}

WASM_EXPORT void test_thread_abort()
{
    std::thread thread(thread_test_abort_entry_point, (void*)nullptr);
    thread.join();
}

WASM_EXPORT void test_abort()
{
    info("test_abort aborting");
    std::abort();
}

#endif

WASM_EXPORT void test_stdout_stderr()
{
    // refactoring our file access methods to fix this warning is not worth it!
    // NOLINTBEGIN(cppcoreguidelines-pro-type-vararg)
    static_cast<void>(fprintf(stdout, "c: hello stdout!"));
    static_cast<void>(fflush(stdout));
    static_cast<void>(fprintf(stderr, "c: hello stderr!"));
    // NOLINTEND(cppcoreguidelines-pro-type-vararg)
    std::cout << "c++: hello stdout!" << std::flush;
    std::cerr << "c++: hello stderr!";
}

WASM_EXPORT void common_init_slab_allocator(uint32_t const* circuit_size)
{
    bb::init_slab_allocator(ntohl(*circuit_size));
}
