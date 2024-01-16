#include "thread_utils.hpp"

namespace bb::thread_utils {
/**
 * @brief calculates number of threads to create based on minimum iterations per thread
 * @details Finds the number of cpus with get_num_cpus(), and calculates `desired_num_threads`
 * Returns the min of `desired_num_threads` and `max_num_threads`.
 * Note that it will not calculate a power of 2 necessarily, use `calculate_num_threads_pow2` instead
 *
 * @param num_iterations
 * @param min_iterations_per_thread
 * @return size_t
 */
size_t calculate_num_threads(size_t num_iterations, size_t min_iterations_per_thread)
{
    size_t max_num_threads = get_num_cpus(); // number of available threads
    size_t desired_num_threads = num_iterations / min_iterations_per_thread;
    size_t num_threads = std::min(desired_num_threads, max_num_threads); // fewer than max if justified
    num_threads = num_threads > 0 ? num_threads : 1;                     // ensure num_threads is at least 1
    return num_threads;
}

/**
 * @brief calculates number of threads to create based on minimum iterations per thread, guaranteed power of 2
 * @details Same functionality as `calculate_num_threads` but guaranteed power of 2
 * @param num_iterations
 * @param min_iterations_per_thread
 * @return size_t
 */
size_t calculate_num_threads_pow2(size_t num_iterations, size_t min_iterations_per_thread)
{
    size_t max_num_threads = get_num_cpus_pow2(); // number of available threads (power of 2)
    size_t desired_num_threads = num_iterations / min_iterations_per_thread;
    desired_num_threads = static_cast<size_t>(1ULL << numeric::get_msb(desired_num_threads));
    size_t num_threads = std::min(desired_num_threads, max_num_threads); // fewer than max if justified
    num_threads = num_threads > 0 ? num_threads : 1;                     // ensure num_threads is at least 1
    return num_threads;
}

} // namespace bb::thread_utils