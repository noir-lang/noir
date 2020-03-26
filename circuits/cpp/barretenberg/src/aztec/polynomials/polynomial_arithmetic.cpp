#include "polynomial_arithmetic.hpp"
#include "iterate_over_domain.hpp"
#include <common/assert.hpp>
#include <common/mem.hpp>
#include <math.h>
#include <memory.h>
#include <numeric/bitop/get_msb.hpp>

namespace barretenberg {
namespace polynomial_arithmetic {
namespace {
static fr* working_memory = nullptr;
static size_t current_size = 0;

// const auto init = []() {
//     constexpr size_t max_num_elements = (1 << 20);
//     working_memory = (fr*)(aligned_alloc(64, max_num_elements * 4 * sizeof(fr)));
//     memset((void*)working_memory, 1, max_num_elements * 4 * sizeof(fr));
//     current_size = (max_num_elements * 4);
//     return 1;
// }();

fr* get_scratch_space(const size_t num_elements)
{
    if (num_elements > current_size) {
        if (working_memory) {
            aligned_free(working_memory);
        }
        working_memory = (fr*)(aligned_alloc(64, num_elements * sizeof(fr)));
        current_size = num_elements;
    }
    return working_memory;
}

} // namespace
// namespace
// {
inline uint32_t reverse_bits(uint32_t x, uint32_t bit_length)
{
    x = (((x & 0xaaaaaaaa) >> 1) | ((x & 0x55555555) << 1));
    x = (((x & 0xcccccccc) >> 2) | ((x & 0x33333333) << 2));
    x = (((x & 0xf0f0f0f0) >> 4) | ((x & 0x0f0f0f0f) << 4));
    x = (((x & 0xff00ff00) >> 8) | ((x & 0x00ff00ff) << 8));
    return (((x >> 16) | (x << 16))) >> (32 - bit_length);
}

void copy_polynomial(fr* src, fr* dest, size_t num_src_coefficients, size_t num_target_coefficients)
{
    // TODO: fiddle around with avx asm to see if we can speed up
    memcpy((void*)dest, (void*)src, num_src_coefficients * sizeof(fr));

    if (num_target_coefficients > num_src_coefficients) {
        // fill out the polynomial coefficients with zeroes
        memset((void*)(dest + num_src_coefficients), 0, (num_target_coefficients - num_src_coefficients) * sizeof(fr));
    }
}

void fft_inner_serial(fr* coeffs, const size_t domain_size, const std::vector<fr*>& root_table)
{
    fr temp;
    size_t log2_size = (size_t)numeric::get_msb(domain_size);
    // efficiently separate odd and even indices - (An introduction to algorithms, section 30.3)

    for (size_t i = 0; i <= domain_size; ++i) {
        uint32_t swap_index = (uint32_t)reverse_bits((uint32_t)i, (uint32_t)log2_size);
        // TODO: should probably use CMOV here insead of an if statement
        if (i < swap_index) {
            fr::__swap(coeffs[i], coeffs[swap_index]);
        }
    }

    // For butterfly operations, we use lazy reduction techniques.
    // Modulus is 254 bits, so we can 'overload' a field element by 4x and still fit it in 4 machine words.
    // We can validate that field elements are <2p and not risk overflowing. Means we can cut
    // two modular reductions from the main loop

    // perform first butterfly iteration explicitly: x0 = x0 + x1, x1 = x0 - x1
    for (size_t k = 0; k < domain_size; k += 2) {
        fr::__copy(coeffs[k + 1], temp);
        coeffs[k + 1] = coeffs[k] - coeffs[k + 1];
        coeffs[k] += temp;
    }

    for (size_t m = 2; m < domain_size; m *= 2) {
        const size_t i = (size_t)numeric::get_msb(m);
        for (size_t k = 0; k < domain_size; k += (2 * m)) {
            for (size_t j = 0; j < m; ++j) {
                temp = root_table[i - 1][j] * coeffs[k + j + m];
                coeffs[k + j + m] = coeffs[k + j] - temp;
                coeffs[k + j] += temp;
            }
        }
    }
}

void scale_by_generator(fr* coeffs,
                        fr* target,
                        const evaluation_domain& domain,
                        const fr& generator_start,
                        const fr& generator_shift,
                        const size_t generator_size)
{
#ifndef NO_MULTITHREADING
#pragma omp parallel for
#endif
    for (size_t j = 0; j < domain.num_threads; ++j) {
        fr thread_shift = generator_shift.pow(static_cast<uint64_t>(j * (generator_size / domain.num_threads)));
        fr work_generator = generator_start * thread_shift;
        const size_t offset = j * (generator_size / domain.num_threads);
        const size_t end = offset + (generator_size / domain.num_threads);
        for (size_t i = offset; i < end; ++i) {
            target[i] = coeffs[i] * work_generator;
            work_generator *= generator_shift;
        }
    }
}

void compute_multiplicative_subgroup(const size_t log2_subgroup_size,
                                     const evaluation_domain& src_domain,
                                     fr* subgroup_roots)
{
    size_t subgroup_size = 1UL << log2_subgroup_size;
    // Step 1: get primitive 4th root of unity
    fr subgroup_root = fr::get_root_of_unity(log2_subgroup_size);

    // Step 2: compute the cofactor term g^n
    fr accumulator = src_domain.generator;
    for (size_t i = 0; i < src_domain.log2_size; ++i) {
        accumulator.self_sqr();
    }

    // Step 3: fill array with 4 values of (g.X)^n - 1, scaled by the cofactor
    subgroup_roots[0] = accumulator;
    for (size_t i = 1; i < subgroup_size; ++i) {
        subgroup_roots[i] = subgroup_roots[i - 1] * subgroup_root;
    }
}

void fft_inner_parallel(fr* coeffs, const evaluation_domain& domain, const fr&, const std::vector<fr*>& root_table)
{
    // hmm  // fr* scratch_space = (fr*)aligned_alloc(64, sizeof(fr) * domain.size);
    fr* scratch_space = get_scratch_space(domain.size);
#ifndef NO_MULTITHREADING
#pragma omp parallel
#endif
    {
// First FFT round is a special case - no need to multiply by root table, because all entries are 1.
// We also combine the bit reversal step into the first round, to avoid a redundant round of copying data
#ifndef NO_MULTITHREADING
#pragma omp for
#endif
        for (size_t j = 0; j < domain.num_threads; ++j) {
            fr temp_1;
            fr temp_2;
            for (size_t i = (j * domain.thread_size); i < ((j + 1) * domain.thread_size); i += 2) {
                uint32_t next_index_1 = (uint32_t)reverse_bits((uint32_t)i + 2, (uint32_t)domain.log2_size);
                uint32_t next_index_2 = (uint32_t)reverse_bits((uint32_t)i + 3, (uint32_t)domain.log2_size);
                __builtin_prefetch(&coeffs[next_index_1]);
                __builtin_prefetch(&coeffs[next_index_2]);

                uint32_t swap_index_1 = (uint32_t)reverse_bits((uint32_t)i, (uint32_t)domain.log2_size);
                uint32_t swap_index_2 = (uint32_t)reverse_bits((uint32_t)i + 1, (uint32_t)domain.log2_size);

                fr::__copy(coeffs[swap_index_1], temp_1);
                fr::__copy(coeffs[swap_index_2], temp_2);
                scratch_space[i + 1] = temp_1 - temp_2;
                scratch_space[i] = temp_1 + temp_2;
            }
        }

        // hard code exception for when the domain size is tiny - we won't execute the next loop, so need to manually
        // reduce + copy
        if (domain.size <= 2) {
            coeffs[0] = scratch_space[0];
            coeffs[1] = scratch_space[1];
        }

        // outer FFT loop
        for (size_t m = 2; m < (domain.size); m <<= 1) {
#ifndef NO_MULTITHREADING
#pragma omp for
#endif
            for (size_t j = 0; j < domain.num_threads; ++j) {
                fr temp;

                // Ok! So, what's going on here? This is the inner loop of the FFT algorithm, and we want to break it
                // out into multiple independent threads. For `num_threads`, each thread will evaluation `domain.size /
                // num_threads` of the polynomial. The actual iteration length will be half of this, because we leverage
                // the fact that \omega^{n/2} = -\omega (where \omega is a root of unity)

                // Here, `start` and `end` are used as our iterator limits, so that we can use our iterator `i` to
                // directly access the roots of unity lookup table
                const size_t start = j * (domain.thread_size >> 1);
                const size_t end = (j + 1) * (domain.thread_size >> 1);

                // For all but the last round of our FFT, the roots of unity that we need, will be a subset of our
                // lookup table. e.g. for a size 2^n FFT, the 2^n'th roots create a multiplicative subgroup of order 2^n
                //      the 1st round will use the roots from the multiplicative subgroup of order 2 : the 2'th roots of
                //      unity the 2nd round will use the roots from the multiplicative subgroup of order 4 : the 4'th
                //      roots of unity
                // i.e. each successive FFT round will double the set of roots that we need to index.
                // We have already laid out the `root_table` container so that each FFT round's roots are linearly
                // ordered in memory. For all FFT rounds, the number of elements we're iterating over is greater than
                // the size of our lookup table. We need to access this table in a cyclical fasion - i.e. for a subgroup
                // of size x, the first x iterations will index the subgroup elements in order, then for the next x
                // iterations, we loop back to the start.

                // We could implement the algorithm by having 2 nested loops (where the inner loop iterates over the
                // root table), but we want to flatten this out - as for the first few rounds, the inner loop will be
                // tiny and we'll have quite a bit of unneccesary branch checks For each iteration of our flattened
                // loop, indexed by `i`, the element of the root table we need to access will be `i % (current round
                // subgroup size)` Given that each round subgroup size is `m`, which is a power of 2, we can index the
                // root table with a very cheap `i & (m - 1)` Which is why we have this odd `block_mask` variable
                const size_t block_mask = m - 1;

                // The next problem to tackle, is we now need to efficiently index the polynomial element in
                // `scratch_space` in our flattened loop If we used nested loops, the outer loop (e.g. `y`) iterates
                // from 0 to 'domain size', in steps of 2 * m, with the inner loop (e.g. `z`) iterating from 0 to m. We
                // have our inner loop indexer with `i & (m - 1)`. We need to add to this our outer loop indexer, which
                // is equivalent to taking our indexer `i`, masking out the bits used in the 'inner loop', and doubling
                // the result. i.e. polynomial indexer = (i & (m - 1)) + ((i & ~(m - 1)) >> 1) To simplify this, we
                // cache index_mask = ~block_mask, meaning that our indexer is just `((i & index_mask) << 1 + (i &
                // block_mask)`
                const size_t index_mask = ~block_mask;

                // `round_roots` fetches the pointer to this round's lookup table. We use `numeric::get_msb(m) - 1` as
                // our indexer, because we don't store the precomputed root values for the 1st round (because they're
                // all 1).
                const fr* round_roots = root_table[static_cast<size_t>(numeric::get_msb(m)) - 1];

                // Finally, we want to treat the final round differently from the others,
                // so that we can reduce out of our 'coarse' reduction and store the output in `coeffs` instead of
                // `scratch_space`
                if (m != (domain.size >> 1)) {
                    for (size_t i = start; i < end; ++i) {
                        size_t k1 = (i & index_mask) << 1;
                        size_t j1 = i & block_mask;
                        temp = round_roots[j1] * scratch_space[k1 + j1 + m];
                        scratch_space[k1 + j1 + m] = scratch_space[k1 + j1] - temp;
                        scratch_space[k1 + j1] += temp;
                    }
                } else {
                    for (size_t i = start; i < end; ++i) {
                        size_t k1 = (i & index_mask) << 1;
                        size_t j1 = i & block_mask;
                        temp = round_roots[j1] * scratch_space[k1 + j1 + m];
                        coeffs[k1 + j1 + m] = scratch_space[k1 + j1] - temp;
                        coeffs[k1 + j1] = scratch_space[k1 + j1] + temp;
                    }
                }
            }
        }
    }
}

void fft_inner_parallel(
    fr* coeffs, fr* target, const evaluation_domain& domain, const fr&, const std::vector<fr*>& root_table)
{
    // hmm  // fr* scratch_space = (fr*)aligned_alloc(64, sizeof(fr) * domain.size);
#ifndef NO_MULTITHREADING
#pragma omp parallel
#endif
    {
// First FFT round is a special case - no need to multiply by root table, because all entries are 1.
// We also combine the bit reversal step into the first round, to avoid a redundant round of copying data
#ifndef NO_MULTITHREADING
#pragma omp for
#endif
        for (size_t j = 0; j < domain.num_threads; ++j) {
            fr temp_1;
            fr temp_2;
            for (size_t i = (j * domain.thread_size); i < ((j + 1) * domain.thread_size); i += 2) {
                uint32_t next_index_1 = (uint32_t)reverse_bits((uint32_t)i + 2, (uint32_t)domain.log2_size);
                uint32_t next_index_2 = (uint32_t)reverse_bits((uint32_t)i + 3, (uint32_t)domain.log2_size);
                __builtin_prefetch(&coeffs[next_index_1]);
                __builtin_prefetch(&coeffs[next_index_2]);

                uint32_t swap_index_1 = (uint32_t)reverse_bits((uint32_t)i, (uint32_t)domain.log2_size);
                uint32_t swap_index_2 = (uint32_t)reverse_bits((uint32_t)i + 1, (uint32_t)domain.log2_size);

                fr::__copy(coeffs[swap_index_1], temp_1);
                fr::__copy(coeffs[swap_index_2], temp_2);
                target[i + 1] = temp_1 - temp_2;
                target[i] = temp_1 + temp_2;
            }
        }

        // hard code exception for when the domain size is tiny - we won't execute the next loop, so need to manually
        // reduce + copy
        if (domain.size <= 2) {
            coeffs[0] = target[0];
            coeffs[1] = target[1];
        }

        // outer FFT loop
        for (size_t m = 2; m < (domain.size); m <<= 1) {
#ifndef NO_MULTITHREADING
#pragma omp for
#endif
            for (size_t j = 0; j < domain.num_threads; ++j) {
                fr temp;

                // Ok! So, what's going on here? This is the inner loop of the FFT algorithm, and we want to break it
                // out into multiple independent threads. For `num_threads`, each thread will evaluation `domain.size /
                // num_threads` of the polynomial. The actual iteration length will be half of this, because we leverage
                // the fact that \omega^{n/2} = -\omega (where \omega is a root of unity)

                // Here, `start` and `end` are used as our iterator limits, so that we can use our iterator `i` to
                // directly access the roots of unity lookup table
                const size_t start = j * (domain.thread_size >> 1);
                const size_t end = (j + 1) * (domain.thread_size >> 1);

                // For all but the last round of our FFT, the roots of unity that we need, will be a subset of our
                // lookup table. e.g. for a size 2^n FFT, the 2^n'th roots create a multiplicative subgroup of order 2^n
                //      the 1st round will use the roots from the multiplicative subgroup of order 2 : the 2'th roots of
                //      unity the 2nd round will use the roots from the multiplicative subgroup of order 4 : the 4'th
                //      roots of unity
                // i.e. each successive FFT round will double the set of roots that we need to index.
                // We have already laid out the `root_table` container so that each FFT round's roots are linearly
                // ordered in memory. For all FFT rounds, the number of elements we're iterating over is greater than
                // the size of our lookup table. We need to access this table in a cyclical fasion - i.e. for a subgroup
                // of size x, the first x iterations will index the subgroup elements in order, then for the next x
                // iterations, we loop back to the start.

                // We could implement the algorithm by having 2 nested loops (where the inner loop iterates over the
                // root table), but we want to flatten this out - as for the first few rounds, the inner loop will be
                // tiny and we'll have quite a bit of unneccesary branch checks For each iteration of our flattened
                // loop, indexed by `i`, the element of the root table we need to access will be `i % (current round
                // subgroup size)` Given that each round subgroup size is `m`, which is a power of 2, we can index the
                // root table with a very cheap `i & (m - 1)` Which is why we have this odd `block_mask` variable
                const size_t block_mask = m - 1;

                // The next problem to tackle, is we now need to efficiently index the polynomial element in
                // `scratch_space` in our flattened loop If we used nested loops, the outer loop (e.g. `y`) iterates
                // from 0 to 'domain size', in steps of 2 * m, with the inner loop (e.g. `z`) iterating from 0 to m. We
                // have our inner loop indexer with `i & (m - 1)`. We need to add to this our outer loop indexer, which
                // is equivalent to taking our indexer `i`, masking out the bits used in the 'inner loop', and doubling
                // the result. i.e. polynomial indexer = (i & (m - 1)) + ((i & ~(m - 1)) >> 1) To simplify this, we
                // cache index_mask = ~block_mask, meaning that our indexer is just `((i & index_mask) << 1 + (i &
                // block_mask)`
                const size_t index_mask = ~block_mask;

                // `round_roots` fetches the pointer to this round's lookup table. We use `numeric::get_msb(m) - 1` as
                // our indexer, because we don't store the precomputed root values for the 1st round (because they're
                // all 1).
                const fr* round_roots = root_table[static_cast<size_t>(numeric::get_msb(m)) - 1];

                // Finally, we want to treat the final round differently from the others,
                // so that we can reduce out of our 'coarse' reduction and store the output in `coeffs` instead of
                // `scratch_space`
                if (m != (domain.size >> 1)) {
                    for (size_t i = start; i < end; ++i) {
                        size_t k1 = (i & index_mask) << 1;
                        size_t j1 = i & block_mask;
                        temp = round_roots[j1] * target[k1 + j1 + m];
                        target[k1 + j1 + m] = target[k1 + j1] - temp;
                        target[k1 + j1] += temp;
                    }
                } else {
                    for (size_t i = start; i < end; ++i) {
                        size_t k1 = (i & index_mask) << 1;
                        size_t j1 = i & block_mask;
                        temp = round_roots[j1] * target[k1 + j1 + m];
                        target[k1 + j1 + m] = target[k1 + j1] - temp;
                        target[k1 + j1] = target[k1 + j1] + temp;
                    }
                }
            }
        }
    }
}

void fft(fr* coeffs, const evaluation_domain& domain)
{
    fft_inner_parallel(coeffs, domain, domain.root, domain.get_round_roots());
}

void ifft(fr* coeffs, const evaluation_domain& domain)
{
    fft_inner_parallel(coeffs, domain, domain.root_inverse, domain.get_inverse_round_roots());
    ITERATE_OVER_DOMAIN_START(domain);
    coeffs[i] *= domain.domain_inverse;
    ITERATE_OVER_DOMAIN_END;
}

void fft_with_constant(fr* coeffs, const evaluation_domain& domain, const fr& value)
{
    fft_inner_parallel(coeffs, domain, domain.root, domain.get_round_roots());
    ITERATE_OVER_DOMAIN_START(domain);
    coeffs[i] *= value;
    ITERATE_OVER_DOMAIN_END;
}

void coset_fft(fr* coeffs, const evaluation_domain& domain)
{
    scale_by_generator(coeffs, coeffs, domain, fr::one(), domain.generator, domain.generator_size);
    fft(coeffs, domain);
}

void coset_fft(fr* coeffs, const evaluation_domain& domain, const evaluation_domain&, const size_t domain_extension)
{
    const size_t log2_domain_extension = static_cast<size_t>(numeric::get_msb(domain_extension));
    fr primitive_root = fr::get_root_of_unity(domain.log2_size + log2_domain_extension);

    // fr work_root = domain.generator.sqr();
    // work_root = domain.generator.sqr();
    fr* scratch_space = get_scratch_space(domain.size * domain_extension);

    // fr* temp_memory = static_cast<fr*>(aligned_alloc(64, sizeof(fr) * domain.size *
    // domain_extension));

    std::vector<fr> coset_generators(domain_extension);
    coset_generators[0] = domain.generator;
    for (size_t i = 1; i < domain_extension; ++i) {
        coset_generators[i] = coset_generators[i - 1] * primitive_root;
    }
    for (size_t i = domain_extension - 1; i < domain_extension; --i) {
        scale_by_generator(coeffs, coeffs + (i * domain.size), domain, fr::one(), coset_generators[i], domain.size);
    }

    for (size_t i = 0; i < domain_extension; ++i) {
        fft_inner_parallel(coeffs + (i * domain.size),
                           scratch_space + (i * domain.size),
                           domain,
                           domain.root,
                           domain.get_round_roots());
    }

    if (domain_extension == 4) {
#ifndef NO_MULTITHREADING
#pragma omp parallel for
#endif
        for (size_t j = 0; j < domain.num_threads; ++j) {
            const size_t start = j * domain.thread_size;
            const size_t end = (j + 1) * domain.thread_size;
            for (size_t i = start; i < end; ++i) {
                fr::__copy(scratch_space[i], coeffs[(i << 2UL)]);
                fr::__copy(scratch_space[i + (1UL << domain.log2_size)], coeffs[(i << 2UL) + 1UL]);
                fr::__copy(scratch_space[i + (2UL << domain.log2_size)], coeffs[(i << 2UL) + 2UL]);
                fr::__copy(scratch_space[i + (3UL << domain.log2_size)], coeffs[(i << 2UL) + 3UL]);
            }
        }
        for (size_t i = 0; i < domain.size; ++i) {
            for (size_t j = 0; j < domain_extension; ++j) {
                fr::__copy(scratch_space[i + (j << domain.log2_size)], coeffs[(i << log2_domain_extension) + j]);
            }
        }
    } else {
        for (size_t i = 0; i < domain.size; ++i) {
            for (size_t j = 0; j < domain_extension; ++j) {
                fr::__copy(scratch_space[i + (j << domain.log2_size)], coeffs[(i << log2_domain_extension) + j]);
            }
        }
    }
}

void coset_fft_with_constant(fr* coeffs, const evaluation_domain& domain, const fr& constant)
{
    fr start = constant;
    scale_by_generator(coeffs, coeffs, domain, start, domain.generator, domain.generator_size);
    fft(coeffs, domain);
}

void ifft_with_constant(fr* coeffs, const evaluation_domain& domain, const fr& value)
{
    fft_inner_parallel(coeffs, domain, domain.root_inverse, domain.get_inverse_round_roots());
    fr T0 = domain.domain_inverse * value;
    ITERATE_OVER_DOMAIN_START(domain);
    coeffs[i] *= T0;
    ITERATE_OVER_DOMAIN_END;
}

void coset_ifft(fr* coeffs, const evaluation_domain& domain)
{
    ifft(coeffs, domain);
    scale_by_generator(coeffs, coeffs, domain, fr::one(), domain.generator_inverse, domain.size);
}

void add(const fr* a_coeffs, const fr* b_coeffs, fr* r_coeffs, const evaluation_domain& domain)
{
    ITERATE_OVER_DOMAIN_START(domain);
    r_coeffs[i] = a_coeffs[i] + b_coeffs[i];
    ITERATE_OVER_DOMAIN_END;
}

void mul(const fr* a_coeffs, const fr* b_coeffs, fr* r_coeffs, const evaluation_domain& domain)
{
    ITERATE_OVER_DOMAIN_START(domain);
    r_coeffs[i] = a_coeffs[i] * b_coeffs[i];
    ITERATE_OVER_DOMAIN_END;
}

fr evaluate(const fr* coeffs, const fr& z, const size_t n)
{
#ifndef NO_MULTITHREADING
    size_t num_threads = (size_t)omp_get_max_threads();
#else
    size_t num_threads = 1;
#endif
    size_t range_per_thread = n / num_threads;
    size_t leftovers = n - (range_per_thread * num_threads);
    fr* evaluations = new fr[num_threads];
#ifndef NO_MULTITHREADING
#pragma omp parallel for
#endif
    for (size_t j = 0; j < num_threads; ++j) {
        fr z_acc = z.pow(static_cast<uint64_t>(j * range_per_thread));
        size_t offset = j * range_per_thread;
        evaluations[j] = fr::zero();
        size_t end = (j == num_threads - 1) ? offset + range_per_thread + leftovers : offset + range_per_thread;
        for (size_t i = offset; i < end; ++i) {
            fr work_var = z_acc * coeffs[i];
            evaluations[j] += work_var;
            z_acc *= z;
        }
    }

    fr r = fr::zero();
    for (size_t j = 0; j < num_threads; ++j) {
        r += evaluations[j];
    }
    delete[] evaluations;
    return r;
}

// For L_1(X) = (X^{n} - 1 / (X - 1)) * (1 / n)
// Compute the 2n-fft of L_1(X)
// We can use this to compute the 2n-fft evaluations of any L_i(X).
// We can consider `l_1_coefficients` to be a 2n-sized vector of the evaluations of L_1(X),
// for all X = 2n'th roots of unity.
// To compute the vector for the 2n-fft transform of L_i(X), we perform a (2i)-left-shift of this vector
void compute_lagrange_polynomial_fft(fr* l_1_coefficients,
                                     const evaluation_domain& src_domain,
                                     const evaluation_domain& target_domain)
{
    // L_1(X) = (X^{n} - 1 / (X - 1)) * (1 / n)
    // when evaluated at the 2n'th roots of unity, the term X^{n} forms a subgroup of order 2
    // w = n'th root of unity
    // w' = 2n'th root of unity = w^{1/2}
    // for even powers of w', X^{n} = w^{2in/2} = 1
    // for odd powers of w', X = w^{i}w^{n/2} -> X^{n} = w^{in}w^{n/2} = -w

    // We also want to compute fft using subgroup union a coset (the multiplicative generator g), so we're not dividing
    // by zero

    // Step 1: compute the denominator for each evaluation: 1 / (X.g - 1)
    // fr work_root;
    fr multiplicand = target_domain.root;

#ifndef NO_MULTITHREADING
#pragma omp parallel for
#endif
    for (size_t j = 0; j < target_domain.num_threads; ++j) {
        const fr root_shift = multiplicand.pow(static_cast<uint64_t>(j * target_domain.thread_size));
        fr work_root = src_domain.generator * root_shift;
        size_t offset = j * target_domain.thread_size;
        for (size_t i = offset; i < offset + target_domain.thread_size; ++i) {
            l_1_coefficients[i] = work_root - fr::one();
            work_root *= multiplicand;
        }
    }

    // use Montgomery's trick to invert all of these at once
    fr::batch_invert(l_1_coefficients, target_domain.size);

    // next: compute numerator multiplicand: w'^{n}.g^n
    // Here, w' is the primitive 2n'th root of unity
    // and w is the primitive n'th root of unity
    // i.e. w' = w^{1/2}
    // The polynomial X^n, when evaluated at all 2n'th roots of unity, forms a subgroup of order 2.
    // For even powers of w', X^n = w'^{2in} = w^{in} = 1
    // For odd powers of w', X^n = w'^{1 + 2in} = w^{n/2}w^{in} = w^{n/2} = -1

    // The numerator term, therefore, can only take two values
    // For even indices: (X^{n} - 1)/n = (g^n - 1)/n
    // For odd indices: (X^{n} - 1)/n = (-g^n - 1)/n

    size_t log2_subgroup_size = target_domain.log2_size - src_domain.log2_size;
    size_t subgroup_size = 1UL << log2_subgroup_size;
    ASSERT(target_domain.log2_size >= src_domain.log2_size);

    fr* subgroup_roots = new fr[subgroup_size];
    compute_multiplicative_subgroup(log2_subgroup_size, src_domain, &subgroup_roots[0]);

    // Each element of `subgroup_roots[i]` contains some root wi^n
    // want to compute (1/n)(wi^n - 1)
    for (size_t i = 0; i < subgroup_size; ++i) {
        subgroup_roots[i] -= fr::one();
        subgroup_roots[i] *= src_domain.domain_inverse;
    }
    // TODO: this is disgusting! Fix it fix it fix it fix it...
    if (subgroup_size >= target_domain.thread_size) {
        for (size_t i = 0; i < target_domain.size; i += subgroup_size) {
            for (size_t j = 0; j < subgroup_size; ++j) {
                l_1_coefficients[i + j] *= subgroup_roots[j];
            }
        }
    } else {
#ifndef NO_MULTITHREADING
#pragma omp parallel for
#endif
        for (size_t k = 0; k < target_domain.num_threads; ++k) {
            size_t offset = k * target_domain.thread_size;
            for (size_t i = offset; i < offset + target_domain.thread_size; i += subgroup_size) {
                for (size_t j = 0; j < subgroup_size; ++j) {
                    l_1_coefficients[i + j] *= subgroup_roots[j];
                }
            }
        }
    }
    delete[] subgroup_roots;
}

void divide_by_pseudo_vanishing_polynomial(fr* coeffs,
                                           const evaluation_domain& src_domain,
                                           const evaluation_domain& target_domain)
{
    // the PLONK divisor polynomial is equal to the vanishing polynomial divided by the vanishing polynomial for the
    // last subgroup element Z_H(X) = \prod_{i=1}^{n-1}(X - w^i) = (X^n - 1) / (X - w^{n-1}) i.e. we divide by vanishing
    // polynomial, then multiply by degree-1 polynomial (X - w^{n-1})

    // `coeffs` should be in point-evaluation form, evaluated at the 4n'th roots of unity
    // P(X) = X^n - 1 will form a subgroup of order 4 when evaluated at these points
    // If X = w^i, P(X) = 1
    // If X = w^{i + j/4}, P(X) = w^{n/4} = w^{n/2}^{n/2} = sqrt(-1)
    // If X = w^{i + j/2}, P(X) = -1
    // If X = w^{i + j/2 + k/4}, P(X) = w^{n/4}.-1 = -w^{i} = -sqrt(-1)
    // i.e. the 4th roots of unity
    size_t log2_subgroup_size = target_domain.log2_size - src_domain.log2_size;
    size_t subgroup_size = 1UL << log2_subgroup_size;
    ASSERT(target_domain.log2_size >= src_domain.log2_size);

    fr* subgroup_roots = new fr[subgroup_size];
    compute_multiplicative_subgroup(log2_subgroup_size, src_domain, &subgroup_roots[0]);

    // Step 3: fill array with values of (g.X)^n - 1, scaled by the cofactor
    for (size_t i = 0; i < subgroup_size; ++i) {
        subgroup_roots[i] -= fr::one();
    }

    // Step 4: invert array entries to compute denominator term of 1/Z_H*(X)
    fr::batch_invert(&subgroup_roots[0], subgroup_size);

    // The numerator term of Z_H*(X) is the polynomial (X - w^{n-1})
    // => (g.w_i - w^{n-1})
    // Compute w^{n-1}
    fr numerator_constant = -src_domain.root_inverse;

    // Compute first value of g.w_i

    // Step 5: iterate over point evaluations, scaling each one by the inverse of the vanishing polynomial
    if (subgroup_size >= target_domain.thread_size) {
        fr work_root = src_domain.generator;
        for (size_t i = 0; i < target_domain.size; i += subgroup_size) {
            for (size_t j = 0; j < subgroup_size; ++j) {
                coeffs[i + j] *= subgroup_roots[j];
                fr T0 = work_root + numerator_constant;
                coeffs[i + j] *= T0;
                work_root *= target_domain.root;
            }
        }
    } else {
#ifndef NO_MULTITHREADING
#pragma omp parallel for
#endif
        for (size_t k = 0; k < target_domain.num_threads; ++k) {
            size_t offset = k * target_domain.thread_size;
            const fr root_shift = target_domain.root.pow(static_cast<uint64_t>(offset));
            fr work_root = src_domain.generator * root_shift;
            for (size_t i = offset; i < offset + target_domain.thread_size; i += subgroup_size) {
                for (size_t j = 0; j < subgroup_size; ++j) {
                    coeffs[i + j] *= subgroup_roots[j];
                    fr T0 = work_root + numerator_constant;
                    coeffs[i + j] *= T0;
                    work_root *= target_domain.root;
                }
            }
        }
    }
    delete[] subgroup_roots;
}

fr compute_kate_opening_coefficients(const fr* src, fr* dest, const fr& z, const size_t n)
{
    // if `coeffs` represents F(X), we want to compute W(X)
    // where W(X) = F(X) - F(z) / (X - z)
    // i.e. divide by the degree-1 polynomial [-z, 1]

    // We assume that the commitment is well-formed and that there is no remainder term.
    // Under these conditions we can perform this polynomial division in linear time with good constants

    fr f = evaluate(src, z, n);
    // compute (1 / -z)
    fr divisor = -z.invert();

    // we're about to shove these coefficients into a pippenger multi-exponentiation routine, where we need
    // to convert out of montgomery form. So, we can use lazy reduction techniques here without triggering overflows
    dest[0] = src[0] - f;
    dest[0] *= divisor;
    for (size_t i = 1; i < n; ++i) {
        dest[i] = src[i] - dest[i - 1];
        dest[i] *= divisor;
    }

    return f;
}

// compute Z_H*(z), l_1(z), l_{n-1}(z)
barretenberg::polynomial_arithmetic::lagrange_evaluations get_lagrange_evaluations(const fr& z,
                                                                                   const evaluation_domain& domain)
{
    fr z_pow = z;
    for (size_t i = 0; i < domain.log2_size; ++i) {
        z_pow.self_sqr();
    }

    fr numerator = z_pow - fr::one();

    fr denominators[3];
    denominators[0] = z - domain.root_inverse;
    denominators[1] = z - fr::one();
    denominators[2] = (z * domain.root.sqr()) - fr::one();
    fr::batch_invert(denominators, 3);

    barretenberg::polynomial_arithmetic::lagrange_evaluations result;
    result.vanishing_poly = numerator * denominators[0];
    numerator = numerator * domain.domain_inverse;
    result.l_1 = numerator * denominators[1];
    result.l_n_minus_1 = numerator * denominators[2];

    return result;
}

// computes r = \sum_{i=0}^{num_coeffs}(L_i(z).f_i)
// start with L_1(z) = ((z^n - 1)/n).(1 / z - 1)
// L_i(z) = L_1(z.w^{1-i}) = ((z^n - 1) / n).(1 / z.w^{1-i} - 1)
fr compute_barycentric_evaluation(fr* coeffs, const size_t num_coeffs, const fr& z, const evaluation_domain& domain)
{
    fr* denominators = static_cast<fr*>(aligned_alloc(64, sizeof(fr) * num_coeffs));

    fr numerator = z;
    for (size_t i = 0; i < domain.log2_size; ++i) {
        numerator.self_sqr();
    }
    numerator -= fr::one();
    numerator *= domain.domain_inverse;

    denominators[0] = z - fr::one();
    fr work_root = domain.root_inverse;
    for (size_t i = 1; i < num_coeffs; ++i) {
        denominators[i] = work_root * z;
        denominators[i] -= fr::one();
        work_root *= domain.root_inverse;
    }

    fr::batch_invert(denominators, num_coeffs);

    fr result = fr::zero();

    for (size_t i = 0; i < num_coeffs; ++i) {
        fr temp = coeffs[i] * denominators[i];
        result = result + temp;
    }

    result = result * numerator;

    aligned_free(denominators);

    return result;
}

// Convert an fft with `current_size` point evaluations, to one with `current_size >> compress_factor` point evaluations
void compress_fft(const fr* src, fr* dest, const size_t cur_size, const size_t compress_factor)
{
    // iterate from top to bottom, allows `dest` to overlap with `src`
    size_t log2_compress_factor = (size_t)numeric::get_msb(compress_factor);
    ASSERT(1UL << log2_compress_factor == compress_factor);
    size_t new_size = cur_size >> log2_compress_factor;
    for (size_t i = 0; i < new_size; ++i) {
        fr::__copy(src[i << log2_compress_factor], dest[i]);
    }
}

} // namespace polynomial_arithmetic
} // namespace barretenberg
