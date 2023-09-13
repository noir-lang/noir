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
 * In doing so, OMP outperformed it significantly (at least for FFT algorithims). This set me on a course to try
 * and understand why and to provide a suitable alternative. Ultimately I found solutions that compared to OMP with
 * "moody" and "atomic_pool" solutions, although they were not *quite* as fast as OMP. However interestingly, when it
 * comes to actual "real world" testing (with proof construction), rather than raw benchmarking, most of the solutions
 * performaed about the same, with OMP *actually slightly worse*. So maybe all this effort was a bit redundant.
 * Remember to always do real world testing...
 *
 * My theory as to why OMP performs so much better in benchmarks is because it runs the tests in a very tight loop,
 * and OMP seems well designed to handle this. It actually looks like OMP consumes more cpu time in htop, and this
 * maybe due to aggressive spin-locking and may explain why it performs well in these scenarios.
 *
 * My theory as to why spawning seems to counter-intuitively perfrom so well, is that spawning a new thread may actually
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
