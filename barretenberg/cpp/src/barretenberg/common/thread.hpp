#pragma once
#include "barretenberg/common/compiler_hints.hpp"
#include <atomic>
#include <barretenberg/env/hardware_concurrency.hpp>
#include <barretenberg/numeric/bitop/get_msb.hpp>
#include <functional>
#include <iostream>
#include <vector>

namespace bb {

inline size_t get_num_cpus()
{
    return env_hardware_concurrency();
}

// For algorithms that need to be divided amongst power of 2 threads.
inline size_t get_num_cpus_pow2()
{
    return static_cast<size_t>(1ULL << numeric::get_msb(get_num_cpus()));
}

/**
 * Creates a thread pool and runs the function in parallel.
 * @param num_iterations Number of iterations
 * @param func Function to run in parallel
 * Observe that num_iterations is NOT the thread pool size.
 * The size will be chosen based on the hardware concurrency (i.e., env or cpus).
 */
void parallel_for(size_t num_iterations, const std::function<void(size_t)>& func);
void parallel_for_range(size_t num_points,
                        const std::function<void(size_t, size_t)>& func,
                        size_t no_multhreading_if_less_or_equal = 0);

/**
 * @brief Split a loop into several loops running in parallel based on operations in 1 iteration
 *
 * @details Splits the num_points into appropriate number of chunks to do parallel processing on and calls the function
 * that should contain the work loop, but only if it's worth it
 * @param num_points Total number of elements
 * @param func A function or lambda expression with a for loop inside, for example:
 * [&](size_t start, size_t end, size_t thread_index){for (size_t i=start; i<end; i++){ ... work ... }
 * @param heuristic_cost the estimated cost of the operation, see namespace thread_heuristics below
 */
void parallel_for_heuristic(size_t num_points,
                            const std::function<void(size_t, size_t, size_t)>& func,
                            size_t heuristic_cost);

template <typename Func>
    requires std::invocable<Func, std::size_t>
void parallel_for_heuristic(size_t num_points, const Func& func, size_t heuristic_cost)
{
    parallel_for_heuristic(
        num_points,
        [&](size_t start_idx, size_t end_idx, BB_UNUSED size_t chunk_index) {
            for (size_t i = start_idx; i < end_idx; i++) {
                func(i);
            }
        },
        heuristic_cost);
}

/**
 * @brief parallel_for_heuristic variant that takes an accumulator initializer
 * that is allocated in a vector, one accumulator per thread/chunk.
 * This allows for thread-safe accumulation, see sum() or sum_pairs() in container.hpp
 * for an easy way to combine the thread/chunk contributions into a final result.
 */
template <typename Func, typename Accum>
    requires std::invocable<Func, std::size_t, Accum&>
std::vector<Accum> parallel_for_heuristic(size_t num_points,
                                          const Accum& initial_accum,
                                          const Func& func,
                                          size_t heuristic_cost)
{
    // thread-safe accumulators
    std::vector<Accum> accumulators(get_num_cpus(), initial_accum);
    parallel_for_heuristic(
        num_points,
        [&](size_t start_idx, size_t end_idx, size_t chunk_index) {
            for (size_t i = start_idx; i < end_idx; i++) {
                func(i, accumulators[chunk_index]);
            }
        },
        heuristic_cost);
    return accumulators;
}

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

namespace thread_heuristics {
// Rough cost of operations (the operation costs are derives in basics_bench and the units are nanoseconds)
// Field element (16 byte) addition cost
constexpr size_t FF_ADDITION_COST = 4;
// Field element (16 byte) multiplication cost
constexpr size_t FF_MULTIPLICATION_COST = 21;
// Field element (16 byte) inversion cost
constexpr size_t FF_INVERSION_COST = 7000;
// Group element projective addition number
constexpr size_t GE_ADDITION_COST = 350;
// Group element projective doubling number
constexpr size_t GE_DOUBLING_COST = 194;
// Group element scalar multiplication cost
constexpr size_t SM_COST = 50000;
// Field element (16 byte) sequential copy number
constexpr size_t FF_COPY_COST = 3;
// Fine default if something looks 'chunky enough that I don't want to calculate'
constexpr size_t ALWAYS_MULTITHREAD = 100000;
} // namespace thread_heuristics

} // namespace bb
