#pragma once
#include "thread.hpp"

namespace bb::thread_utils {

const size_t DEFAULT_MIN_ITERS_PER_THREAD = 1 << 4;

/**
 * @brief calculates number of threads to create based on minimum iterations per thread
 * @details Finds the number of cpus with get_num_cpus(), and calculates `desired_num_threads`
 * Returns the min of `desired_num_threads` and `max_num_theads`.
 * Note that it will not calculate a power of 2 necessarily, use `calculate_num_threads_pow2` instead
 *
 * @param num_iterations
 * @param min_iterations_per_thread
 * @return size_t
 */
size_t calculate_num_threads(size_t num_iterations, size_t min_iterations_per_thread = DEFAULT_MIN_ITERS_PER_THREAD);

/**
 * @brief calculates number of threads to create based on minimum iterations per thread, guaranteed power of 2
 * @details Same functionality as `calculate_num_threads` but guaranteed power of 2
 * @param num_iterations
 * @param min_iterations_per_thread
 * @return size_t
 */
size_t calculate_num_threads_pow2(size_t num_iterations,
                                  size_t min_iterations_per_thread = DEFAULT_MIN_ITERS_PER_THREAD);

} // namespace bb::thread_utils