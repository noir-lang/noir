#include "thread.hpp"
#include "log.hpp"

/**
 * There's a lot to talk about here. To bring threading to WASM, parallel_for was written to replace the OpenMP loops
 * we had scattered throughout our code. It provides a clean abstraction for the work division strategy we use (we
 * used OMP's`"#pragma omp parallel for` everywhere).
 *
 * The first implementation was `parallel_for_spawning`. You can read a description of each implementation in the
 * relevant source file, but parallel_for_spawning is the simplest approach imaginable.
 * Once WASM was working, I checked its performance in native code by running it against the polynomials benchmarks.
 * In doing so, OMP outperformed it significantly (at least for FFT algorithms). This set me on a course to try
 * and understand why and to provide a suitable alternative. Ultimately I found solutions that compared to OMP with
 * "moody" and "atomic_pool" solutions, although they were not *quite* as fast as OMP. However interestingly, when it
 * comes to actual "real world" testing (with proof construction), rather than raw benchmarking, most of the solutions
 * performed about the same, with OMP *actually slightly worse*. So maybe all this effort was a bit redundant.
 * Remember to always do real world testing...
 *
 * My theory as to why OMP performs so much better in benchmarks is because it runs the tests in a very tight loop,
 * and OMP seems well designed to handle this. It actually looks like OMP consumes more cpu time in htop, and this
 * maybe due to aggressive spin-locking and may explain why it performs well in these scenarios.
 *
 * My theory as to why spawning seems to counter-intuitively perform so well, is that spawning a new thread may actually
 * be cheaper than waking a sleeping thread. Or joining is somehow very efficient. Or it's because there's very low
 * other overhead. Or libc++ STL does some magic. Ok, that's not much of a theory...
 *
 * Ultimately though the takeaway is as follows:
 * - OMP maybe preferable when running benchmarks if you want to check for that kind of "optimal linear scaling".
 *   Although, if we want to get rid of OMP altogether, "atomic_pool" is a simple solution that seems to compare.
 * - The simplest "spawning" is probably best used everywhere else, and frees us from needing OMP to build the lib.
 *
 * UPDATE!: So although spawning is simple and fast, due to unstable pthreads in wasi-sdk that causes hangs when
 * joining threads, we use "atomic_pool" by default. We may just wish to revert to spawning once it stablises.
 *
 * UPDATE!: Interestingly "atomic_pool" performs worse than "mutex_pool" for some e.g. proving key construction.
 * Haven't done deeper analysis. Defaulting to mutex_pool.
 */

namespace bb {
// 64 core aws r5.
// pippenger run: pippenger_bench/1048576
// coset_fft run: coset_fft_bench_parallel/4194304
// proof run: 2m gate ultraplonk. average of 5.

// pippenger: 179ms
// coset_fft: 54776us
// proof: 11.33s
void parallel_for_omp(size_t num_iterations, const std::function<void(size_t)>& func);

// pippenger: 163ms
// coset_fft: 59993us
// proof: 11.11s
void parallel_for_moody(size_t num_iterations, const std::function<void(size_t)>& func);

// pippenger: 154ms
// coset_fft: 92997us
// proof: 10.84s
void parallel_for_spawning(size_t num_iterations, const std::function<void(size_t)>& func);

// pippenger: 178ms
// coset_fft: 70207us
// proof: 11.55s
void parallel_for_queued(size_t num_iterations, const std::function<void(size_t)>& func);

// pippenger: 152ms
// coset_fft: 56658us
// proof: 11.28s
void parallel_for_atomic_pool(size_t num_iterations, const std::function<void(size_t)>& func);

void parallel_for_mutex_pool(size_t num_iterations, const std::function<void(size_t)>& func);

void parallel_for(size_t num_iterations, const std::function<void(size_t)>& func)
{
#ifdef NO_MULTITHREADING
    for (size_t i = 0; i < num_iterations; ++i) {
        func(i);
    }
#else
#ifndef NO_OMP_MULTITHREADING
    parallel_for_omp(num_iterations, func);
#else
    // parallel_for_spawning(num_iterations, func);
    // parallel_for_moody(num_iterations, func);
    // parallel_for_atomic_pool(num_iterations, func);
    parallel_for_mutex_pool(num_iterations, func);
    // parallel_for_queued(num_iterations, func);
#endif
#endif
}

/**
 * @brief Split a loop into several loops running in parallel
 *
 * @details Splits the num_points into appropriate number of chunks to do parallel processing on and calls the function
 * that should contain the work loop
 * @param num_points Total number of elements
 * @param func A function or lambda expression with a for loop inside, for example:
 * [](size_t start, size_t end){for (size_t i=start; i<end; i++){(void)i;}}
 * @param no_multhreading_if_less_or_equal If num points is less or equal to this value, run without parallelization
 *
 */
void parallel_for_range(size_t num_points,
                        const std::function<void(size_t, size_t)>& func,
                        size_t no_multhreading_if_less_or_equal)
{
    if (num_points <= no_multhreading_if_less_or_equal) {
        func(0, num_points);
        return;
    }
    // Get number of cpus we can split into
    const size_t num_cpus = get_num_cpus();

    // Compute the size of a single chunk
    const size_t chunk_size = (num_points / num_cpus) + (num_points % num_cpus == 0 ? 0 : 1);
    // Parallelize over chunks
    parallel_for(num_cpus, [num_points, chunk_size, &func](size_t chunk_index) {
        // If num_points is small, sometimes we need fewer CPUs
        if (chunk_size * chunk_index > num_points) {
            return;
        }
        // Compute the current chunk size (can differ in case it's the last chunk)
        size_t current_chunk_size = std::min(num_points - (chunk_size * chunk_index), chunk_size);
        if (current_chunk_size == 0) {
            return;
        }
        size_t start = chunk_index * chunk_size;
        size_t end = chunk_index * chunk_size + current_chunk_size;
        func(start, end);
    });
};

void parallel_for_heuristic(size_t num_points,
                            const std::function<void(size_t, size_t, size_t)>& func,
                            size_t heuristic_cost)
{
    // We take the maximum observed parallel_for cost (388 us) and round it up.
    // The goals of these checks is to evade significantly (10x) increasing processing time for small workloads. So we
    // can accept not triggering parallel_for if the workload would become faster by half a millisecond for medium
    // workloads
    constexpr size_t PARALLEL_FOR_COST = 400000;
    // Get number of cpus we can split into
    const size_t num_cpus = get_num_cpus();

    // Compute the size of a single chunk
    const size_t chunk_size = (num_points / num_cpus) + (num_points % num_cpus == 0 ? 0 : 1);

    // Compute the cost of all operations done by other threads
    const size_t offset_cost = (num_points - chunk_size) * heuristic_cost;

    // If starting parallel for is longer than computing, just compute
    if (offset_cost < PARALLEL_FOR_COST) {
        func(0, num_points, 0);
        return;
    }
    // Parallelize over chunks
    parallel_for(num_cpus, [num_points, chunk_size, &func](size_t chunk_index) {
        // If num_points is small, sometimes we need fewer CPUs
        if (chunk_size * chunk_index > num_points) {
            return;
        }
        // Compute the current chunk size (can differ in case it's the last chunk)
        size_t current_chunk_size = std::min(num_points - (chunk_size * chunk_index), chunk_size);
        if (current_chunk_size == 0) {
            return;
        }
        size_t start = chunk_index * chunk_size;
        size_t end = chunk_index * chunk_size + current_chunk_size;

        func(start, end, chunk_index);
    });
};

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
} // namespace bb