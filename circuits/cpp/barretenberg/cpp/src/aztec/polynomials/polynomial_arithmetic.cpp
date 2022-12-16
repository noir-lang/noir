#include "polynomial_arithmetic.hpp"
#include "iterate_over_domain.hpp"
#include <common/assert.hpp>
#include <common/mem.hpp>
#include <math.h>
#include <memory.h>
#include <numeric/bitop/get_msb.hpp>
#include <common/max_threads.hpp>

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

inline bool is_power_of_two(uint64_t x)
{
    return x && !(x & (x - 1));
}

void copy_polynomial(const fr* src, fr* dest, size_t num_src_coefficients, size_t num_target_coefficients)
{
    // TODO: fiddle around with avx asm to see if we can speed up
    memcpy((void*)dest, (void*)src, num_src_coefficients * sizeof(fr));

    if (num_target_coefficients > num_src_coefficients) {
        // fill out the polynomial coefficients with zeroes
        memset((void*)(dest + num_src_coefficients), 0, (num_target_coefficients - num_src_coefficients) * sizeof(fr));
    }
}

void fft_inner_serial(std::vector<fr*> coeffs, const size_t domain_size, const std::vector<fr*>& root_table)
{
    // Assert that the number of polynomials is a power of two.
    const size_t num_polys = coeffs.size();
    ASSERT(is_power_of_two(num_polys));
    const size_t poly_domain_size = domain_size / num_polys;
    ASSERT(is_power_of_two(poly_domain_size));

    fr temp;
    size_t log2_size = (size_t)numeric::get_msb(domain_size);
    size_t log2_poly_size = (size_t)numeric::get_msb(poly_domain_size);
    // efficiently separate odd and even indices - (An introduction to algorithms, section 30.3)

    for (size_t i = 0; i <= domain_size; ++i) {
        uint32_t swap_index = (uint32_t)reverse_bits((uint32_t)i, (uint32_t)log2_size);
        // TODO: should probably use CMOV here insead of an if statement
        if (i < swap_index) {
            size_t even_poly_idx = i >> log2_poly_size;
            size_t even_elem_idx = i % poly_domain_size;
            size_t odd_poly_idx = swap_index >> log2_poly_size;
            size_t odd_elem_idx = swap_index % poly_domain_size;
            fr::__swap(coeffs[even_poly_idx][even_elem_idx], coeffs[odd_poly_idx][odd_elem_idx]);
        }
    }

    // For butterfly operations, we use lazy reduction techniques.
    // Modulus is 254 bits, so we can 'overload' a field element by 4x and still fit it in 4 machine words.
    // We can validate that field elements are <2p and not risk overflowing. Means we can cut
    // two modular reductions from the main loop

    // perform first butterfly iteration explicitly: x0 = x0 + x1, x1 = x0 - x1
    for (size_t l = 0; l < num_polys; l++) {
        for (size_t k = 0; k < poly_domain_size; k += 2) {
            fr::__copy(coeffs[l][k + 1], temp);
            coeffs[l][k + 1] = coeffs[l][k] - coeffs[l][k + 1];
            coeffs[l][k] += temp;
        }
    }

    for (size_t m = 2; m < domain_size; m *= 2) {
        const size_t i = (size_t)numeric::get_msb(m);
        for (size_t k = 0; k < domain_size; k += (2 * m)) {
            for (size_t j = 0; j < m; ++j) {
                const size_t even_poly_idx = (k + j) >> log2_poly_size;
                const size_t even_elem_idx = (k + j) & (poly_domain_size - 1);
                const size_t odd_poly_idx = (k + j + m) >> log2_poly_size;
                const size_t odd_elem_idx = (k + j + m) & (poly_domain_size - 1);

                temp = root_table[i - 1][j] * coeffs[odd_poly_idx][odd_elem_idx];
                coeffs[odd_poly_idx][odd_elem_idx] = coeffs[even_poly_idx][even_elem_idx] - temp;
                coeffs[even_poly_idx][even_elem_idx] += temp;
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
/**
 * Compute multiplicative subgroup (g.X)^n.
 *
 * Compute the subgroup for X in roots of unity of (2^log2_subgroup_size)*n.
 * X^n will loop through roots of unity (2^log2_subgroup_size).
 *
 * @param log2_subgroup_size Log_2 of the subgroup size.
 * @param src_domain The domain of size n.
 * @param subgroup_roots Pointer to the array for saving subgroup members.
 * */
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

    // Step 3: fill array with subgroup_size values of (g.X)^n, scaled by the cofactor
    subgroup_roots[0] = accumulator;
    for (size_t i = 1; i < subgroup_size; ++i) {
        subgroup_roots[i] = subgroup_roots[i - 1] * subgroup_root;
    }
}

void fft_inner_parallel(std::vector<fr*> coeffs,
                        const evaluation_domain& domain,
                        const fr&,
                        const std::vector<fr*>& root_table)
{
    fr* scratch_space = get_scratch_space(domain.size);

    const size_t num_polys = coeffs.size();
    ASSERT(is_power_of_two(num_polys));
    const size_t poly_size = domain.size / num_polys;
    ASSERT(is_power_of_two(poly_size));
    const size_t poly_mask = poly_size - 1;
    const size_t log2_poly_size = (size_t)numeric::get_msb(poly_size);

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

                size_t poly_idx_1 = swap_index_1 >> log2_poly_size;
                size_t elem_idx_1 = swap_index_1 & poly_mask;
                size_t poly_idx_2 = swap_index_2 >> log2_poly_size;
                size_t elem_idx_2 = swap_index_2 & poly_mask;

                fr::__copy(coeffs[poly_idx_1][elem_idx_1], temp_1);
                fr::__copy(coeffs[poly_idx_2][elem_idx_2], temp_2);
                scratch_space[i + 1] = temp_1 - temp_2;
                scratch_space[i] = temp_1 + temp_2;
            }
        }

        // hard code exception for when the domain size is tiny - we won't execute the next loop, so need to manually
        // reduce + copy
        if (domain.size <= 2) {
            coeffs[0][0] = scratch_space[0];
            coeffs[0][1] = scratch_space[1];
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

                        size_t poly_idx_1 = (k1 + j1) >> log2_poly_size;
                        size_t elem_idx_1 = (k1 + j1) & poly_mask;
                        size_t poly_idx_2 = (k1 + j1 + m) >> log2_poly_size;
                        size_t elem_idx_2 = (k1 + j1 + m) & poly_mask;

                        temp = round_roots[j1] * scratch_space[k1 + j1 + m];
                        coeffs[poly_idx_2][elem_idx_2] = scratch_space[k1 + j1] - temp;
                        coeffs[poly_idx_1][elem_idx_1] = scratch_space[k1 + j1] + temp;
                    }
                }
            }
        }
    }
}

void fft_inner_parallel(
    fr* coeffs, fr* target, const evaluation_domain& domain, const fr&, const std::vector<fr*>& root_table)
{
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
                for (size_t i = start; i < end; ++i) {
                    size_t k1 = (i & index_mask) << 1;
                    size_t j1 = i & block_mask;
                    temp = round_roots[j1] * target[k1 + j1 + m];
                    target[k1 + j1 + m] = target[k1 + j1] - temp;
                    target[k1 + j1] += temp;
                }
            }
        }
    }
}

void partial_fft_serial_inner(fr* coeffs,
                              fr* target,
                              const evaluation_domain& domain,
                              const std::vector<fr*>& root_table)
{
    // We wish to compute a partial modified FFT of 2 rounds from given coefficients.
    // We need a 2-round modified FFT for commiting to the 4n-sized quotient polynomial for
    // the PLONK prover.
    //
    // We assume that the number of coefficients is a multiplicand of 4, since the domain size
    // we use in PLONK would always be a power of 2, this is a reasonable assumption.
    // Let n = N / 4 where N is the input domain size, we wish to compute
    // R_{i,s} = \sum_{j=0}^{3} Y_{i + jn} * \omega^{(i + jn)(s + 1)}
    //
    // We should store the result in the following way:
    // (R_{0,3} , R_{1,3}, R_{3,3}, ..., R_{n, 3})  {coefficients of X^0}
    // (R_{0,2} , R_{1,2}, R_{3,2}, ..., R_{n, 2})  {coefficients of X^1}
    // (R_{0,1} , R_{1,1}, R_{3,1}, ..., R_{n, 1})  {coefficients of X^2}
    // (R_{0,0} , R_{1,0}, R_{3,0}, ..., R_{n, 0})  {coefficients of X^3}
    size_t n = domain.size >> 2;
    size_t index = 0;
    size_t full_mask = domain.size - 1;
    size_t m = domain.size >> 1;
    size_t half_mask = m - 1;
    const fr* round_roots = root_table[static_cast<size_t>(numeric::get_msb(m)) - 1];
    size_t root_index = 0;

    // iterate for s = 0, 1, 2, 3 to compute R_{i,s}
    for (size_t i = 0; i < n; ++i) {
        for (size_t s = 0; s < 4; s++) {
            target[(3 - s) * n + i] = 0;
            for (size_t j = 0; j < 4; ++j) {
                index = i + j * n;
                root_index = (index * (s + 1)) & full_mask;
                target[(3 - s) * n + i] +=
                    (root_index < m ? fr::one() : -fr::one()) * coeffs[index] * round_roots[root_index & half_mask];
            }
        }
    }
}

void partial_fft_parellel_inner(
    fr* coeffs, const evaluation_domain& domain, const std::vector<fr*>& root_table, fr constant, bool is_coset)
{
    // We wish to compute a partial modified FFT of 2 rounds from given coefficients.
    // We need a 2-round modified FFT for commiting to the 4n-sized quotient polynomial for
    // the PLONK prover.
    //
    // We assume that the number of coefficients is a multiplicand of 4, since the domain size
    // we use in PLONK would always be a power of 2, this is a reasonable assumption.
    // Let n = N / 4 where N is the input domain size, we wish to compute
    // R_{i,s} = \sum_{j=0}^{3} Y_{i + jn} * \omega^{(i + jn)(s + 1)}
    //
    // Input `coeffs` is the evaluation form (FFT) of a polynomial.
    // (Y_{0,0} , Y_{1,0}, Y_{3,0}, ..., Y_{n, 0})
    // (Y_{0,1} , Y_{1,1}, Y_{3,1}, ..., Y_{n, 1})
    // (Y_{0,2} , Y_{1,2}, Y_{3,2}, ..., Y_{n, 2})
    // (Y_{0,3} , Y_{1,3}, Y_{3,3}, ..., Y_{n, 3})
    //
    // We should store the result in the following way:
    // (R_{0,3} , R_{1,3}, R_{3,3}, ..., R_{n, 3})  {coefficients of X^0}
    // (R_{0,2} , R_{1,2}, R_{3,2}, ..., R_{n, 2})  {coefficients of X^1}
    // (R_{0,1} , R_{1,1}, R_{3,1}, ..., R_{n, 1})  {coefficients of X^2}
    // (R_{0,0} , R_{1,0}, R_{3,0}, ..., R_{n, 0})  {coefficients of X^3}

    size_t n = domain.size >> 2;
    size_t full_mask = domain.size - 1;
    size_t m = domain.size >> 1;
    size_t half_mask = m - 1;
    const fr* round_roots = root_table[static_cast<size_t>(numeric::get_msb(m)) - 1];

    evaluation_domain small_domain(n);

    // iterate for s = 0, 1, 2, 3 to compute R_{i,s}
    ITERATE_OVER_DOMAIN_START(small_domain);
    fr temp[4];
    temp[0] = coeffs[i];
    temp[1] = coeffs[i + n];
    temp[2] = coeffs[i + 2 * n];
    temp[3] = coeffs[i + 3 * n];
    coeffs[i] = 0;
    coeffs[i + n] = 0;
    coeffs[i + 2 * n] = 0;
    coeffs[i + 3 * n] = 0;

    size_t index, root_index;
    fr temp_constant = constant;
    fr root_multiplier = 1;

    for (size_t s = 0; s < 4; s++) {
        for (size_t j = 0; j < 4; ++j) {
            index = i + j * n;
            root_index = (index * (s + 1));
            if (is_coset) {
                root_index -= 4 * i;
            }
            root_index &= full_mask;
            root_multiplier = round_roots[root_index & half_mask];
            if (root_index >= m) {
                root_multiplier = -round_roots[root_index & half_mask];
            }
            coeffs[(3 - s) * n + i] += root_multiplier * temp[j];
        }
        if (is_coset) {
            temp_constant *= domain.generator;
            coeffs[(3 - s) * n + i] *= temp_constant;
        }
    }
    ITERATE_OVER_DOMAIN_END;
}

void partial_fft_serial(fr* coeffs, fr* target, const evaluation_domain& domain)
{
    partial_fft_serial_inner(coeffs, target, domain, domain.get_round_roots());
}

void partial_fft(fr* coeffs, const evaluation_domain& domain, fr constant, bool is_coset)
{
    partial_fft_parellel_inner(coeffs, domain, domain.get_round_roots(), constant, is_coset);
}

void fft(fr* coeffs, const evaluation_domain& domain)
{
    fft_inner_parallel({ coeffs }, domain, domain.root, domain.get_round_roots());
}

void fft(fr* coeffs, fr* target, const evaluation_domain& domain)
{
    fft_inner_parallel(coeffs, target, domain, domain.root, domain.get_round_roots());
}

void fft(std::vector<fr*> coeffs, const evaluation_domain& domain)
{
    fft_inner_parallel(coeffs, domain.size, domain.root, domain.get_round_roots());
}

void ifft(fr* coeffs, const evaluation_domain& domain)
{
    fft_inner_parallel({ coeffs }, domain, domain.root_inverse, domain.get_inverse_round_roots());
    ITERATE_OVER_DOMAIN_START(domain);
    coeffs[i] *= domain.domain_inverse;
    ITERATE_OVER_DOMAIN_END;
}

void ifft(fr* coeffs, fr* target, const evaluation_domain& domain)
{
    fft_inner_parallel(coeffs, target, domain, domain.root_inverse, domain.get_inverse_round_roots());
    ITERATE_OVER_DOMAIN_START(domain);
    target[i] *= domain.domain_inverse;
    ITERATE_OVER_DOMAIN_END;
}

void ifft(std::vector<fr*> coeffs, const evaluation_domain& domain)
{
    fft_inner_parallel(coeffs, domain, domain.root_inverse, domain.get_inverse_round_roots());

    const size_t num_polys = coeffs.size();
    ASSERT(is_power_of_two(num_polys));
    const size_t poly_size = domain.size / num_polys;
    ASSERT(is_power_of_two(poly_size));
    const size_t poly_mask = poly_size - 1;
    const size_t log2_poly_size = (size_t)numeric::get_msb(poly_size);

    ITERATE_OVER_DOMAIN_START(domain);
    coeffs[i >> log2_poly_size][i & poly_mask] *= domain.domain_inverse;
    ITERATE_OVER_DOMAIN_END;
}

void fft_with_constant(fr* coeffs, const evaluation_domain& domain, const fr& value)
{
    fft_inner_parallel({ coeffs }, domain, domain.root, domain.get_round_roots());
    ITERATE_OVER_DOMAIN_START(domain);
    coeffs[i] *= value;
    ITERATE_OVER_DOMAIN_END;
}

void coset_fft(fr* coeffs, const evaluation_domain& domain)
{
    scale_by_generator(coeffs, coeffs, domain, fr::one(), domain.generator, domain.generator_size);
    fft(coeffs, domain);
}

void coset_fft(fr* coeffs, fr* target, const evaluation_domain& domain)
{
    scale_by_generator(coeffs, target, domain, fr::one(), domain.generator, domain.generator_size);
    fft(coeffs, target, domain);
}

void coset_fft(std::vector<fr*> coeffs, const evaluation_domain& domain)
{
    const size_t num_polys = coeffs.size();
    ASSERT(is_power_of_two(num_polys));
    const size_t poly_size = domain.size / num_polys;
    const fr generator_pow_n = domain.generator.pow(poly_size);
    fr generator_start = 1;

    for (size_t i = 0; i < num_polys; i++) {
        scale_by_generator(coeffs[i], coeffs[i], domain, generator_start, domain.generator, poly_size);
        generator_start *= generator_pow_n;
    }
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

void coset_fft_with_generator_shift(fr* coeffs, const evaluation_domain& domain, const fr& constant)
{
    scale_by_generator(coeffs, coeffs, domain, fr::one(), domain.generator * constant, domain.generator_size);
    fft(coeffs, domain);
}

void ifft_with_constant(fr* coeffs, const evaluation_domain& domain, const fr& value)
{
    fft_inner_parallel({ coeffs }, domain, domain.root_inverse, domain.get_inverse_round_roots());
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

void coset_ifft(std::vector<fr*> coeffs, const evaluation_domain& domain)
{
    ifft(coeffs, domain);

    const size_t num_polys = coeffs.size();
    ASSERT(is_power_of_two(num_polys));
    const size_t poly_size = domain.size / num_polys;
    const fr generator_inv_pow_n = domain.generator_inverse.pow(poly_size);
    fr generator_start = 1;

    for (size_t i = 0; i < num_polys; i++) {
        scale_by_generator(coeffs[i], coeffs[i], domain, generator_start, domain.generator_inverse, poly_size);
        generator_start *= generator_inv_pow_n;
    }
}

void add(const fr* a_coeffs, const fr* b_coeffs, fr* r_coeffs, const evaluation_domain& domain)
{
    ITERATE_OVER_DOMAIN_START(domain);
    r_coeffs[i] = a_coeffs[i] + b_coeffs[i];
    ITERATE_OVER_DOMAIN_END;
}

void sub(const fr* a_coeffs, const fr* b_coeffs, fr* r_coeffs, const evaluation_domain& domain)
{
    ITERATE_OVER_DOMAIN_START(domain);
    r_coeffs[i] = a_coeffs[i] - b_coeffs[i];
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
    size_t num_threads = max_threads::compute_num_threads();
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

fr evaluate(const std::vector<fr*> coeffs, const fr& z, const size_t large_n)
{
    const size_t num_polys = coeffs.size();
    const size_t poly_size = large_n / num_polys;
    ASSERT(is_power_of_two(poly_size));
    const size_t log2_poly_size = (size_t)numeric::get_msb(poly_size);
#ifndef NO_MULTITHREADING
    size_t num_threads = max_threads::compute_num_threads();
#else
    size_t num_threads = 1;
#endif
    size_t range_per_thread = large_n / num_threads;
    size_t leftovers = large_n - (range_per_thread * num_threads);
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
            fr work_var = z_acc * coeffs[i >> log2_poly_size][i & (poly_size - 1)];
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

/**
 * @brief Compute evaluations of lagrange polynomial L_1(X) on the specified domain
 *
 * @param l_1_coefficients
 * @param src_domain
 * @param target_domain
 * @details Let the size of the target domain be k*n, where k is a power of 2.
 * Evaluate L_1(X) = (X^{n} - 1 / (X - 1)) * (1 / n) at the k*n points X_i = w'^i.g,
 * i = 0, 1,..., k*n-1, where w' is the target domain (kn'th) root of unity, and g is the
 * source domain multiplicative generator. The evaluation domain is taken to be the coset
 * w'^i.g, rather than just the kn'th roots, to avoid division by zero in L_1(X).
 * The computation is done in three steps:
 * Step 1) (Parallelized) Compute the evaluations of 1/denominator of L_1 at X_i using
 * Montgomery batch inversion.
 * Step 2) Compute the evaluations of the numerator of L_1 using the fact that (X_i)^n forms
 * a subgroup of order k.
 * Step 3) (Parallelized) Construct the evaluations of L_1 on X_i using the numerator and
 * denominator evaluations from Steps 1 and 2.
 *
 * Note 1: Let w = n'th root of unity. When evaluated at the k*n'th roots of unity, the term
 * X^{n} forms a subgroup of order k, since (w'^i)^n = w^{in/k} = w^{1/k}. Similarly, for X_i
 * we have (X_i)^n = (w'^i.g)^n = w^{in/k}.g^n = w^{1/k}.g^n.
 * For example, if k = 2:
 * for even powers of w', X^{n} = w^{2in/2} = 1
 * for odd powers of w', X = w^{i}w^{n/2} -> X^{n} = w^{in}w^{n/2} = -1
 * The numerator term, therefore, can only take two values (for k = 2):
 * For even indices: (X^{n} - 1)/n = (g^n - 1)/n
 * For odd indices: (X^{n} - 1)/n = (-g^n - 1)/n
 *
 * Note 2: We can use the evaluations of L_1 to compute the k*n-fft evaluations of any L_i(X).
 * We can consider `l_1_coefficients` to be a k*n-sized vector of the evaluations of L_1(X),
 * for all X = k*n'th roots of unity. To compute the vector for the k*n-fft transform of
 * L_i(X), we perform a (k*i)-left-shift of this vector.
 */
void compute_lagrange_polynomial_fft(fr* l_1_coefficients,
                                     const evaluation_domain& src_domain,
                                     const evaluation_domain& target_domain)
{
    // Step 1: Compute the 1/denominator for each evaluation: 1 / (X_i - 1)
    fr multiplicand = target_domain.root; // kn'th root of unity w'

#ifndef NO_MULTITHREADING
#pragma omp parallel for
#endif
    // First compute X_i - 1, i = 0,...,kn-1
    for (size_t j = 0; j < target_domain.num_threads; ++j) {
        const fr root_shift = multiplicand.pow(static_cast<uint64_t>(j * target_domain.thread_size));
        fr work_root = src_domain.generator * root_shift; // g.(w')^{j*thread_size}
        size_t offset = j * target_domain.thread_size;
        for (size_t i = offset; i < offset + target_domain.thread_size; ++i) {
            l_1_coefficients[i] = work_root - fr::one(); // (w')^{j*thread_size + i}.g - 1
            work_root *= multiplicand;                   // (w')^{j*thread_size + i + 1}
        }
    }

    // Compute 1/(X_i - 1) using Montgomery batch inversion
    fr::batch_invert(l_1_coefficients, target_domain.size);

    // Step 2: Compute numerator (1/n)*(X_i^n - 1)
    // First compute X_i^n (which forms a multiplicative subgroup of order k)
    size_t log2_subgroup_size = target_domain.log2_size - src_domain.log2_size; // log_2(k)
    size_t subgroup_size = 1UL << log2_subgroup_size;                           // k
    ASSERT(target_domain.log2_size >= src_domain.log2_size);
    fr* subgroup_roots = new fr[subgroup_size];
    compute_multiplicative_subgroup(log2_subgroup_size, src_domain, &subgroup_roots[0]);

    // Subtract 1 and divide by n to get the k elements (1/n)*(X_i^n - 1)
    for (size_t i = 0; i < subgroup_size; ++i) {
        subgroup_roots[i] -= fr::one();
        subgroup_roots[i] *= src_domain.domain_inverse;
    }
    // Step 3: Construct L_1(X_i) by multiplying the 1/denominator evaluations in
    // l_1_coefficients by the numerator evaluations in subgroup_roots
    size_t subgroup_mask = subgroup_size - 1;
#ifndef NO_MULTITHREADING
#pragma omp parallel for
#endif
    for (size_t i = 0; i < target_domain.num_threads; ++i) {
        for (size_t j = 0; j < target_domain.thread_size; ++j) {
            size_t eval_idx = i * target_domain.thread_size + j;
            l_1_coefficients[eval_idx] *= subgroup_roots[eval_idx & subgroup_mask];
        }
    }
    delete[] subgroup_roots;
}

void divide_by_pseudo_vanishing_polynomial(std::vector<fr*> coeffs,
                                           const evaluation_domain& src_domain,
                                           const evaluation_domain& target_domain,
                                           const size_t num_roots_cut_out_of_vanishing_polynomial)
{
    // Older version:
    // the PLONK divisor polynomial is equal to the vanishing polynomial divided by the vanishing polynomial for the
    // last subgroup element Z_H(X) = \prod_{i=1}^{n-1}(X - w^i) = (X^n - 1) / (X - w^{n-1}) i.e. we divide by vanishing
    // polynomial, then multiply by degree-1 polynomial (X - w^{n-1})

    // Updated version:
    // We wish to implement this function such that it supports a modified vanishing polynomial, in which
    // k (= num_roots_cut_out_of_vanishing_polynomial) roots are cut out. i.e.
    //                           (X^n - 1)
    // Z*_H(X) = ------------------------------------------
    //           (X - w^{n-1}).(X - w^{n-2})...(X - w^{k})
    //
    // We set the default value of k as 4 so as to ensure that the evaluation domain is 4n. The reason for cutting out
    // some roots is described here: https://hackmd.io/@zacwilliamson/r1dm8Rj7D#The-problem-with-this-approach.
    // Briefly, the reason we need to cut roots is because on adding randomness to permutation polynomial z(X),
    // its degree becomes (n + 2), so for fft evaluation, we will need an evaluation domain of size >= 4(n + 2) = 8n
    // since size of evalutation domain needs to be a power of two. To avoid this, we need to bring down the degree
    // of the permutation polynomial (after adding randomness) to <= n.
    //
    //
    // NOTE: If in future, there arises a need to cut off more zeros, this method will not require any changes.
    //

    // Assert that the number of polynomials in coeffs is a power of 2.
    const size_t num_polys = coeffs.size();
    ASSERT(is_power_of_two(num_polys));
    const size_t poly_size = target_domain.size / num_polys;
    ASSERT(is_power_of_two(poly_size));
    const size_t poly_mask = poly_size - 1;
    const size_t log2_poly_size = (size_t)numeric::get_msb(poly_size);

    // `fft_point_evaluations` should be in point-evaluation form, evaluated at the 4n'th roots of unity mulitplied by
    // `target_domain`'s coset generator P(X) = X^n - 1 will form a subgroup of order 4 when evaluated at these points
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

    // The numerator term of Z_H*(X) is the polynomial (X - w^{n-1})(X - w^{n-2})...(X - w^{n-k})
    // => (g.w_i - w^{n-1})(g.w_i - w^{n-2})...(g.w_i - w^{n-k})
    // Compute w^{n-1}
    std::vector<fr> numerator_constants(num_roots_cut_out_of_vanishing_polynomial);
    if (num_roots_cut_out_of_vanishing_polynomial > 0) {
        numerator_constants[0] = -src_domain.root_inverse;
        for (size_t i = 1; i < num_roots_cut_out_of_vanishing_polynomial; ++i) {
            numerator_constants[i] = numerator_constants[i - 1] * src_domain.root_inverse;
        }
    }
    // Compute first value of g.w_i

    // Step 5: iterate over point evaluations, scaling each one by the inverse of the vanishing polynomial
    if (subgroup_size >= target_domain.thread_size) {
        fr work_root = src_domain.generator;
        for (size_t i = 0; i < target_domain.size; i += subgroup_size) {
            for (size_t j = 0; j < subgroup_size; ++j) {
                size_t poly_idx = (i + j) >> log2_poly_size;
                size_t elem_idx = (i + j) & poly_mask;
                coeffs[poly_idx][elem_idx] *= subgroup_roots[j];

                for (size_t k = 0; k < num_roots_cut_out_of_vanishing_polynomial; ++k) {
                    coeffs[poly_idx][elem_idx] *= work_root + numerator_constants[k];
                }
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
                    size_t poly_idx = (i + j) >> log2_poly_size;
                    size_t elem_idx = (i + j) & poly_mask;
                    coeffs[poly_idx][elem_idx] *= subgroup_roots[j];

                    for (size_t k = 0; k < num_roots_cut_out_of_vanishing_polynomial; ++k) {
                        coeffs[poly_idx][elem_idx] *= work_root + numerator_constants[k];
                    }

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

/**
 * @param zeta - the name given (in our code) to the evaluation challenge ʓ from the Plonk paper.
 */
barretenberg::polynomial_arithmetic::lagrange_evaluations get_lagrange_evaluations(
    const fr& zeta, const evaluation_domain& domain, const size_t num_roots_cut_out_of_vanishing_polynomial)
{
    // Compute Z_H*(ʓ), l_start(ʓ), l_{end}(ʓ)
    // Note that as we modify the vanishing polynomial by cutting out some roots, we must simultaneously ensure that
    // the lagrange polynomials we require would be l_1(ʓ) and l_{n-k}(ʓ) where k =
    // num_roots_cut_out_of_vanishing_polynomial. For notational simplicity, we call l_1 as l_start and l_{n-k} as
    // l_end.
    //
    // NOTE: If in future, there arises a need to cut off more zeros, this method will not require any changes.
    //

    fr z_pow_n = zeta;
    for (size_t i = 0; i < domain.log2_size; ++i) {
        z_pow_n.self_sqr();
    }

    fr numerator = z_pow_n - fr::one();

    fr denominators[3];

    // Compute the denominator of Z_H*(ʓ)
    //   (ʓ - ω^{n-1})(ʓ - ω^{n-2})...(ʓ - ω^{n - num_roots_cut_out_of_vanishing_poly})
    // = (ʓ - ω^{ -1})(ʓ - ω^{ -2})...(ʓ - ω^{  - num_roots_cut_out_of_vanishing_poly})
    fr work_root = domain.root_inverse;
    denominators[0] = fr::one();
    for (size_t i = 0; i < num_roots_cut_out_of_vanishing_polynomial; ++i) {
        denominators[0] *= (zeta - work_root);
        work_root *= domain.root_inverse;
    }

    // The expressions of the lagrange polynomials are:
    //
    //           ω^0.(X^n - 1)      (X^n - 1)
    // L_1(X) = --------------- =  -----------
    //            n.(X - ω^0)       n.(X - 1)
    //
    // Notice: here (in this comment), the index i of L_i(X) counts from 1 (not from 0). So L_1 corresponds to the
    // _first_ root of unity ω^0, and not to the 1-th root of unity ω^1.
    //
    //
    //             ω^{i-1}.(X^n - 1)         X^n - 1          X^n.(ω^{-i+1})^n - 1
    // L_{i}(X) = ------------------ = -------------------- = -------------------- = L_1(X.ω^{-i+1})
    //              n.(X - ω^{i-1})    n.(X.ω^{-(i-1)} - 1) |  n.(X.ω^{-i+1} - 1)
    //                                                      |
    //                                                      since (ω^{-i+1})^n = 1 trivially
    //
    //                                                          (X^n - 1)
    // => L_{n-k}(X) = L_1(X.ω^{k-n+1}) = L_1(X.ω^{k+1}) =  -----------------
    //                                                      n.(X.ω^{k+1} - 1)
    //
    denominators[1] = zeta - fr::one();

    // Compute ω^{num_roots_cut_out_of_vanishing_polynomial + 1}
    fr l_end_root = (num_roots_cut_out_of_vanishing_polynomial & 1) ? domain.root.sqr() : domain.root;
    for (size_t i = 0; i < num_roots_cut_out_of_vanishing_polynomial / 2; ++i) {
        l_end_root *= domain.root.sqr();
    }
    denominators[2] = (zeta * l_end_root) - fr::one();
    fr::batch_invert(denominators, 3);

    barretenberg::polynomial_arithmetic::lagrange_evaluations result;
    result.vanishing_poly = numerator * denominators[0]; // (ʓ^n - 1) / (ʓ-ω^{-1}).(ʓ-ω^{-2})...(ʓ-ω^{-k}) =: Z_H*(ʓ)
    numerator = numerator * domain.domain_inverse;       // (ʓ^n - 1) / n
    result.l_start = numerator * denominators[1];        // (ʓ^n - 1) / (n.(ʓ - 1))         =: L_1(ʓ)
    result.l_end = numerator * denominators[2];          // (ʓ^n - 1) / (n.(ʓ.ω^{k+1} - 1)) =: L_{n-k}(ʓ)

    return result;
}

// Computes r = \sum_{i=0}^{num_coeffs-1} (L_{i+1}(ʓ).f_i)
//
//                     (ʓ^n - 1)
// Start with L_1(ʓ) = ---------
//                     n.(ʓ - 1)
//
//                                 ʓ^n - 1
// L_i(z) = L_1(ʓ.ω^{1-i}) = ------------------
//                           n.(ʓ.ω^{1-i)} - 1)
//
fr compute_barycentric_evaluation(const fr* coeffs,
                                  const size_t num_coeffs,
                                  const fr& z,
                                  const evaluation_domain& domain)
{
    fr* denominators = static_cast<fr*>(aligned_alloc(64, sizeof(fr) * num_coeffs));

    fr numerator = z;
    for (size_t i = 0; i < domain.log2_size; ++i) {
        numerator.self_sqr();
    }
    numerator -= fr::one();
    numerator *= domain.domain_inverse; // (ʓ^n - 1) / n

    denominators[0] = z - fr::one();
    fr work_root = domain.root_inverse; // ω^{-1}
    for (size_t i = 1; i < num_coeffs; ++i) {
        denominators[i] =
            work_root * z; // denominators[i] will correspond to L_[i+1] (since our 'commented maths' notation indexes
                           // L_i from 1). So ʓ.ω^{-i} = ʓ.ω^{1-(i+1)} is correct for L_{i+1}.
        denominators[i] -= fr::one(); // ʓ.ω^{-i} - 1
        work_root *= domain.root_inverse;
    }

    fr::batch_invert(denominators, num_coeffs);

    fr result = fr::zero();

    for (size_t i = 0; i < num_coeffs; ++i) {
        fr temp = coeffs[i] * denominators[i]; // f_i * 1/(ʓ.ω^{-i} - 1)
        result = result + temp;
    }

    result = result *
             numerator; //   \sum_{i=0}^{num_coeffs-1} f_i * [ʓ^n - 1]/[n.(ʓ.ω^{-i} - 1)]
                        // = \sum_{i=0}^{num_coeffs-1} f_i * L_{i+1}
                        // (with our somewhat messy 'commented maths' convention that L_1 corresponds to the 0th coeff).

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

fr evaluate_from_fft(const fr* poly_coset_fft,
                     const evaluation_domain& large_domain,
                     const fr& z,
                     const evaluation_domain& small_domain)
{
    size_t n = small_domain.size;
    fr* small_poly_coset_fft = static_cast<fr*>(aligned_alloc(64, sizeof(fr) * n));
    for (size_t i = 0; i < n; ++i) {
        small_poly_coset_fft[i] = poly_coset_fft[4 * i];
    }

    fr zeta_by_g = z * large_domain.generator_inverse;

    const auto result = compute_barycentric_evaluation(small_poly_coset_fft, n, zeta_by_g, small_domain);
    aligned_free(small_poly_coset_fft);
    return result;
}

// This function computes sum of all scalars in a given array.
fr compute_sum(const fr* src, const size_t n)
{
    fr result = 0;
    for (size_t i = 0; i < n; ++i) {
        result += src[i];
    }
    return result;
}

// This function computes the polynomial (x - a)(x - b)(x - c)... given n distinct roots (a, b, c, ...).
void compute_linear_polynomial_product(const fr* roots, fr* dest, const size_t n)
{
    fr* scratch_space = get_scratch_space(n);
    memcpy((void*)scratch_space, (void*)roots, n * sizeof(fr));

    dest[n] = 1;
    dest[n - 1] = -compute_sum(scratch_space, n);

    fr temp;
    fr constant = 1;
    for (size_t i = 0; i < n - 1; ++i) {
        temp = 0;
        for (size_t j = 0; j < n - 1 - i; ++j) {
            scratch_space[j] = roots[j] * compute_sum(&scratch_space[j + 1], n - 1 - i - j);
            temp += scratch_space[j];
        }
        dest[n - 2 - i] = temp * constant;
        constant *= fr::neg_one();
    }
}

fr compute_linear_polynomial_product_evaluation(const fr* roots, const fr z, const size_t n)
{
    fr result = 1;
    for (size_t i = 0; i < n; ++i) {
        result *= (z - roots[i]);
    }
    return result;
}

void fft_linear_polynomial_product(
    const fr* roots, fr* dest, const size_t n, const evaluation_domain& domain, const bool is_coset)
{
    size_t m = domain.size >> 1;
    const fr* round_roots = domain.get_round_roots()[static_cast<size_t>(numeric::get_msb(m)) - 1];

    fr current_root = 0;
    for (size_t i = 0; i < m; ++i) {
        current_root = round_roots[i];
        current_root *= (is_coset ? domain.generator : 1);
        dest[i] = 1;
        dest[i + m] = 1;
        for (size_t j = 0; j < n; ++j) {
            dest[i] *= (current_root - roots[j]);
            dest[i + m] *= (-current_root - roots[j]);
        }
    }
}

void compute_interpolation(const fr* src, fr* dest, const fr* evaluation_points, const size_t n)
{
    std::vector<fr> local_roots;
    fr local_polynomial[n];
    fr denominator = 1;
    fr multiplicand;
    fr temp_dest[n];

    if (n == 1) {
        temp_dest[0] = src[0];
        return;
    }

    // Initialize dest
    for (size_t i = 0; i < n; ++i) {
        temp_dest[i] = 0;
    }

    for (size_t i = 0; i < n; ++i) {

        // fill in local roots
        denominator = 1;
        for (size_t j = 0; j < n; ++j) {
            if (j == i) {
                continue;
            }
            local_roots.push_back(evaluation_points[j]);
            denominator *= (evaluation_points[i] - evaluation_points[j]);
        }

        // bring local roots to coefficient form
        compute_linear_polynomial_product(&local_roots[0], local_polynomial, n - 1);

        // store the resulting coefficients
        multiplicand = src[i] / denominator;
        for (size_t j = 0; j < n; ++j) {
            temp_dest[j] += multiplicand * local_polynomial[j];
        }

        // clear up local roots
        local_roots.clear();
    }

    memcpy((void*)dest, (void*)temp_dest, n * sizeof(fr));
}

void compute_efficient_interpolation(const fr* src, fr* dest, const fr* evaluation_points, const size_t n)
{
    /*
        We use Lagrange technique to compute polynomial interpolation.
        Given: (x_i, y_i) for i ∈ {0, 1, ..., n} =: [n]
        Compute function f(X) such that f(x_i) = y_i for all i ∈ [n].
                   (X - x1)(X - x2)...(X - xn)             (X - x0)(X - x2)...(X - xn)
        F(X) = y0--------------------------------  +  y1----------------------------------  + ...
                 (x0 - x_1)(x0 - x_2)...(x0 - xn)       (x1 - x_0)(x1 - x_2)...(x1 - xn)
        We write this as:
                      [          yi        ]
        F(X) = N(X) * |∑_i --------------- |
                      [     (X - xi) * di  ]
        where:
        N(X) = ∏_{i \in [n]} (X - xi),
        di = ∏_{j != i} (xi - xj)
        For division of N(X) by (X - xi), we use the same trick that was used in compute_opening_polynomial()
        function in the kate commitment scheme.
    */
    fr numerator_polynomial[n + 1];
    polynomial_arithmetic::compute_linear_polynomial_product(evaluation_points, numerator_polynomial, n);

    fr roots_and_denominators[2 * n];
    fr temp_src[n];
    for (size_t i = 0; i < n; ++i) {
        roots_and_denominators[i] = -evaluation_points[i];
        temp_src[i] = src[i];
        dest[i] = 0;

        // compute constant denominator
        roots_and_denominators[n + i] = 1;
        for (size_t j = 0; j < n; ++j) {
            if (j == i) {
                continue;
            }
            roots_and_denominators[n + i] *= (evaluation_points[i] - evaluation_points[j]);
        }
    }

    fr::batch_invert(roots_and_denominators, 2 * n);

    fr z, multiplier;
    fr temp_dest[n];
    for (size_t i = 0; i < n; ++i) {
        z = roots_and_denominators[i];
        multiplier = temp_src[i] * roots_and_denominators[n + i];
        temp_dest[0] = multiplier * numerator_polynomial[0];
        temp_dest[0] *= z;
        dest[0] += temp_dest[0];
        for (size_t j = 1; j < n; ++j) {
            temp_dest[j] = multiplier * numerator_polynomial[j] - temp_dest[j - 1];
            temp_dest[j] *= z;
            dest[j] += temp_dest[j];
        }
    }
}

} // namespace polynomial_arithmetic
} // namespace barretenberg
