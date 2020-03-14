#include "./scalar_multiplication.hpp"
#include "../../../groups/wnaf.hpp"
#include "../fq.hpp"
#include "../fr.hpp"
#include "../g1.hpp"
#include "./process_buckets.hpp"
#include "./runtime_states.hpp"
#include "./scalar_multiplication.hpp"
#include <algorithm>
#include <array>
#include <cstdlib>
#include <math.h>
#include <numeric/bitop/get_msb.hpp>
#include <stddef.h>
#include <stdint.h>

#ifndef NO_MULTITHREADING
#include <omp.h>
#endif

namespace barretenberg {
namespace scalar_multiplication {

void generate_pippenger_point_table(g1::affine_element* points, g1::affine_element* table, size_t num_points)
{
    // iterate backwards, so that `points` and `table` can point to the same memory location
    for (size_t i = num_points - 1; i < num_points; --i) {
        table[i * 2] = points[i];
        table[i * 2 + 1].x = fq::beta() * points[i].x;
        table[i * 2 + 1].y = -points[i].y;
    }
}

/**
 * Compute the windowed-non-adjacent-form versions of our scalar multipliers.
 *
 * We start by splitting our 254 bit scalars into 2 127-bit scalars, using the short weierstrass curve endomorphism
 * (for a point P \in \G === (x, y) \in \Fq, then (\beta x, y) = (\lambda) * P , where \beta = 1^{1/3} mod Fq and
 *\lambda = 1^{1/3} mod Fr) (which means we can represent a scalar multiplication (k * P) as (k1 * P + k2 * \lambda *
 *P), where k1, k2 have 127 bits) (see field::split_into_endomorphism_scalars for more details)
 *
 * Once we have our 127-bit scalar multipliers, we determine the optimal number of pippenger rounds, given the number of
 *points we're multiplying. Once we have the number of rounds, `m`, we need to split our scalar into `m` bit-slices.
 *Each pippenger round will work on one bit-slice.
 *
 * Pippenger's algorithm works by, for each round, iterating over the points we're multplying. For each point, we
 *examing the point's scalar multiplier and extract the bit-slice associated with the current pippenger round (we start
 *with the most significant slice). We then use the bit-slice to index a 'bucket', which we add the point into. For
 *example, if the bit slice is 01101, we add the corresponding point into bucket[13].
 *
 * At the end of each pippenger round we concatenate the buckets together. E.g. if we have 8 buckets, we compute:
 * sum = bucket[0] + 2 * bucket[1] + 3 * bucket[2] + 4 * bucket[3] + 5 * bucket[4] + 6 * bucket[5] + 7 * bucket[6] + 8 *
 *bucket[7].
 *
 * At the end of each pippenger round, the bucket sum will contain the scalar multiplication result for one bit slice.
 * For example, say we have 16 rounds, where each bit slice contains 8 bits (8 * 16 = 128, enough to represent our 127
 *bit scalars). At the end of the first round, we will have taken the 8 most significant bits from every scalar
 *multiplier. Our bucket sum will be the result of a mini-scalar-multiplication, where we have multiplied every point by
 *the 8 most significant bits of each point's scalar multiplier.
 *
 * We repeat this process for every pippenger round. In our example, this gives us 16 bucket sums.
 * We need to multiply the most significant bucket sum by 2^{120}, the second most significant bucket sum by 2^{112}
 *etc. Once this is done we can add the bucket sums together, to evaluate our scalar multiplication result.
 *
 * Pippenger has complexity O(n / logn), because of two factors at play: the number of buckets we need to concatenate
 *per round, and the number of points we need to add into buckets per round.
 *
 * To minimize the number of point additions per round, we want fewer rounds. But fewer rounds increases the number of
 *bucket concatenations. The more points we have, the greater the time saving when reducing the number of rounds, which
 *means we can afford to have more buckets per round.
 *
 * For a concrete example, with 2^20 points, the sweet spot is 2^15 buckets - with 2^15 buckets we can evaluate our 127
 *bit scalar multipliers in 8 rounds (we can represent b-bit windows with 2^{b-1} buckets, more on that below).
 *
 * This means that, for each round, we add 2^21 points into buckets (we've split our scalar multpliers into two
 *half-width multipliers, so each round has twice the number of points. This is the reason why the endormorphism is
 *useful here; without the endomorphism, we would need twice the number of buckets for each round).
 *
 * We also concatenate 2^15 buckets for each round. This requires 2^16 point additions.
 *
 * Meaning that the total number of point additions is (8 * 2^21) + (8 * 2^16) = 33 * 2^19 ~ 2^24 point additions.
 * If we were to use a simple Montgomery double-and-add ladder to exponentiate each point, we would need 2^27 point
 *additions (each scalar multiplier has ~2^7 non-zero bits, and there are 2^20 points).
 *
 * This makes pippenger 8 times faster than the naive O(n) equivalent. Given that a circuit with 1 million gates will
 *require 9 multiple-scalar-multiplications with 2^20 points, efficiently using Pippenger's algorithm is essential for
 *fast provers
 *
 * One additional efficiency gain is the use of 2^{b-1} buckets to represent b bits. To do this we represent our
 *bit-slices in non-adjacent form. Non-adjacent form represents values using a base, where each 'bit' can take the
 *values (-1, 0, 1). This is considerably more efficient than binary form for scalar multiplication, as inverting a
 *point can be done by negating the y-coordinate.
 *
 * We actually use a slightly different representation than simple non-adjacent form. To represent b bits, a bit slice
 *contains values from (-2^{b} - 1, ..., -1, 1, ..., 2^{b} - 1). i.e. we only have odd values. We do this to eliminate
 *0-valued windows, as having a conditional branch in our hot loop to check if an entry is 0 is somethin we want to
 *avoid.
 *
 * The above representation can be used to represent any binary number as long as we add a 'skew' factor. Each scalar
 *multiplier's `skew` tracks if the scalar multiplier is even or odd. If it's even, `skew = true`, and we add `1` to our
 *multiplier to make it odd.
 *
 * We then, at the end of the Pippenger algorithm, subtract a point from the total result, if that point's skew is
 *`true`.
 *
 * At the end of `compute_wnaf_states`, `state.wnaf_table` will contain our wnaf entries, but unsorted.
 **/
void compute_wnaf_states(uint64_t* point_schedule,
                         bool* input_skew_table,
                         uint64_t* round_counts,
                         const fr* scalars,
                         const size_t num_initial_points)
{
    const size_t num_points = num_initial_points * 2;
    constexpr size_t MAX_NUM_ROUNDS = 256;
    constexpr size_t MAX_NUM_THREADS = 128;
    const size_t num_rounds = get_num_rounds(num_points);
    const size_t bits_per_bucket = get_optimal_bucket_width(num_initial_points);
    const size_t wnaf_bits = bits_per_bucket + 1;
#ifndef NO_MULTITHREADING
    const size_t num_threads = static_cast<size_t>(omp_get_max_threads());
#else
    const size_t num_threads = 1;
#endif
    const size_t num_initial_points_per_thread = num_initial_points / num_threads;
    const size_t num_points_per_thread = num_points / num_threads;
    std::array<std::array<uint64_t, MAX_NUM_ROUNDS>, MAX_NUM_THREADS> thread_round_counts;
    for (size_t i = 0; i < num_threads; ++i) {
        for (size_t j = 0; j < num_rounds; ++j) {
            thread_round_counts[i][j] = 0;
        }
    }
#ifndef NO_MULTITHREADING
#pragma omp parallel for
#endif
    for (size_t i = 0; i < num_threads; ++i) {
        fr T0;
        uint64_t* wnaf_table = &point_schedule[(2 * i) * num_initial_points_per_thread];
        const fr* thread_scalars = &scalars[i * num_initial_points_per_thread];
        bool* skew_table = &input_skew_table[(2 * i) * num_initial_points_per_thread];
        uint64_t offset = i * num_points_per_thread;

        for (uint64_t j = 0; j < num_initial_points_per_thread; ++j) {
            T0 = thread_scalars[j].from_montgomery_form();
            fr::split_into_endomorphism_scalars(T0, T0, *(fr*)&T0.data[2]);

            wnaf::fixed_wnaf_with_counts(&T0.data[0],
                                         &wnaf_table[(j << 1UL)],
                                         skew_table[j << 1ULL],
                                         &thread_round_counts[i][0],
                                         ((j << 1ULL) + offset) << 32ULL,
                                         num_points,
                                         wnaf_bits);
            wnaf::fixed_wnaf_with_counts(&T0.data[2],
                                         &wnaf_table[(j << 1UL) + 1],
                                         skew_table[(j << 1UL) + 1],
                                         &thread_round_counts[i][0],
                                         ((j << 1UL) + offset + 1) << 32UL,
                                         num_points,
                                         wnaf_bits);
        }
    }

    for (size_t i = 0; i < num_rounds; ++i) {
        round_counts[i] = 0;
    }
    for (size_t i = 0; i < num_threads; ++i) {
        for (size_t j = 0; j < num_rounds; ++j) {
            round_counts[j] += thread_round_counts[i][j];
        }
    }
}

/**
 *  Sorts our wnaf entries in increasing bucket order (per round).
 *  We currently don't multi-thread the inner sorting algorithm, and just split our threads over the number of rounds.
 *  A multi-threaded sorting algorithm could be more efficient, but the total runtime of `organize_buckets` is <5% of
 *  pippenger's runtime, so not a priority.
 **/
void organize_buckets(uint64_t* point_schedule, const uint64_t*, const size_t num_points)
{
    const size_t num_rounds = get_num_rounds(num_points);
#ifndef NO_MULTITHREADING
#pragma omp parallel for
#endif
    for (size_t i = 0; i < num_rounds; ++i) {
        scalar_multiplication::process_buckets(&point_schedule[i * num_points],
                                               num_points,
                                               static_cast<uint32_t>(get_optimal_bucket_width(num_points / 2)) + 1);
    }
}

inline void scalar_multiplication_round_inner(multiplication_thread_state& state,
                                              const size_t num_points,
                                              const uint64_t bucket_offset,
                                              g1::affine_element* points)
{
    g1::affine_element* current_point;
    g1::element* current_bucket;
    g1::affine_element* next_point = points + ((state.point_schedule[0]) >> 32UL);
    g1::element* next_bucket = state.buckets + (state.point_schedule[0] & 0x7fffffffUL) - bucket_offset;
    uint64_t current_negative;
    uint64_t next_negative = ((state.point_schedule[0] >> 31UL) & 1UL);

    for (size_t i = 1; i < num_points; ++i) {
        current_point = next_point;
        current_bucket = next_bucket;
        current_negative = next_negative;

        next_point = points + ((state.point_schedule[i]) >> 32UL);
        next_bucket = state.buckets + (state.point_schedule[i] & 0x7fffffffUL) - bucket_offset;
        next_negative = ((state.point_schedule[i] >> 31UL) & 1UL);

        __builtin_prefetch(next_point);

        (*current_bucket).self_mixed_add_or_sub(*current_point, current_negative);
    }

    (*next_bucket).self_mixed_add_or_sub(*next_point, next_negative);
}

g1::element scalar_multiplication_internal(pippenger_runtime_state& state,
                                           g1::affine_element* points,
                                           const size_t num_points)
{
    const size_t num_rounds = get_num_rounds(num_points);
#ifndef NO_MULTITHREADING
    const size_t num_threads = static_cast<size_t>(omp_get_max_threads());
#else
    const size_t num_threads = 1;
#endif
    const size_t bits_per_bucket = get_optimal_bucket_width(num_points / 2);

    g1::element* thread_accumulators = static_cast<g1::element*>(aligned_alloc(64, num_threads * sizeof(g1::element)));

    std::vector<uint64_t> bucket_offsets(num_threads);
    for (size_t j = 0; j < num_threads; ++j) {
        uint64_t max_buckets = 0;
        for (size_t i = 0; i < num_rounds; ++i) {
            const uint64_t num_round_points = state.round_counts[i];
            if ((num_round_points == 0) || (num_round_points < num_threads && (j != num_threads - 1))) {
                continue;
            }
            const uint64_t num_round_points_per_thread = num_round_points / num_threads;
            const uint64_t leftovers =
                (j == num_threads - 1) ? (num_round_points) - (num_round_points_per_thread * num_threads) : 0;
            const uint64_t* thread_point_schedule =
                &state.point_schedule[(i * num_points) + (j * num_round_points_per_thread)];
            const uint64_t first_bucket = thread_point_schedule[0] & 0x7fffffffU;
            const uint64_t last_bucket =
                thread_point_schedule[num_round_points_per_thread - 1 + leftovers] & 0x7fffffffU;
            const uint64_t num_thread_buckets = (last_bucket - first_bucket) + 1;
            if (num_thread_buckets > max_buckets) {
                max_buckets = num_thread_buckets;
            }
        }
        bucket_offsets[j] = max_buckets;
    }
    for (size_t j = 1; j < num_threads; ++j) {
        bucket_offsets[j] += bucket_offsets[j - 1];
    }

    g1::element* buckets = state.buckets;
#ifndef NO_MULTITHREADING
#pragma omp parallel for
#endif
    for (size_t j = 0; j < num_threads; ++j) {
        thread_accumulators[j].self_set_infinity();

        g1::element* thread_buckets = buckets + (j == 0 ? 0 : bucket_offsets[j - 1]);
        for (size_t i = 0; i < num_rounds; ++i) {
            const uint64_t num_round_points = state.round_counts[i];
            // if (num_round_points == 0) {
            //     continue;
            // }

            g1::element accumulator;
            accumulator.self_set_infinity();

            if ((num_round_points == 0) || (num_round_points < num_threads && (j != num_threads - 1))) {
            } else {
                uint64_t num_round_points_per_thread = num_round_points / num_threads;
                uint64_t leftovers =
                    (j == num_threads - 1) ? (num_round_points) - (num_round_points_per_thread * num_threads) : 0;
                const uint64_t* thread_point_schedule =
                    &state.point_schedule[(i * num_points) + j * num_round_points_per_thread];
                const size_t first_bucket = thread_point_schedule[0] & 0x7fffffffU;
                const size_t last_bucket =
                    thread_point_schedule[num_round_points_per_thread - 1 + leftovers] & 0x7fffffffU;
                const size_t num_thread_buckets = (last_bucket - first_bucket) + 1;
                for (size_t k = 0; k < num_thread_buckets; ++k) {
                    thread_buckets[k].self_set_infinity();
                }
                multiplication_thread_state thread_state{ thread_buckets, thread_point_schedule };
                scalar_multiplication_round_inner(
                    thread_state, num_round_points_per_thread + leftovers, first_bucket, points);
                g1::element running_sum;
                running_sum.self_set_infinity();
                for (size_t k = num_thread_buckets - 1; k > 0; --k) {
                    running_sum += thread_buckets[k];
                    accumulator += running_sum;
                }
                running_sum += thread_buckets[0];
                accumulator.self_dbl();
                accumulator += running_sum;

                // we now need to scale up 'running sum' up to the value of the first bucket.
                // e.g. if first bucket is 0, no scaling
                // if first bucket is 1, we need to add (2 * running_sum)
                if (first_bucket > 0) {
                    uint32_t multiplier = static_cast<uint32_t>(first_bucket << 1UL);
                    size_t shift = numeric::get_msb(multiplier);
                    g1::element rolling_accumulator = g1::point_at_infinity;
                    bool init = false;
                    while (shift != static_cast<size_t>(-1)) {
                        if (init) {
                            rolling_accumulator.self_dbl();
                            if (((multiplier >> shift) & 1)) {
                                rolling_accumulator += running_sum;
                            }
                        } else {
                            rolling_accumulator += running_sum;
                        }
                        init = true;
                        shift -= 1;
                    }
                    accumulator += rolling_accumulator;
                }
            }
            const size_t num_points_per_thread = num_points / num_threads;
            if (i == (num_rounds - 1)) {
                bool* skew_table = &state.skew_table[j * num_points_per_thread];
                g1::affine_element* point_table = &points[j * num_points_per_thread];
                g1::affine_element addition_temporary;
                for (size_t k = 0; k < num_points_per_thread; ++k) {
                    if (skew_table[k]) {
                        addition_temporary = -point_table[k];
                        accumulator += addition_temporary;
                    }
                }
            }

            if (i > 0) {
                for (size_t k = 0; k < bits_per_bucket + 1; ++k) {
                    thread_accumulators[j].self_dbl();
                }
            }
            thread_accumulators[j] += accumulator;
        }
    }

    g1::element result;
    result.self_set_infinity();
    for (size_t i = 0; i < num_threads; ++i) {
        result += thread_accumulators[i];
    }
    free(thread_accumulators);
    if (result.is_point_at_infinity()) {
        std::cout << "result is point at infinity" << std::endl;
    }
    return result;
}

inline g1::element pippenger_internal(g1::affine_element* points,
                                      fr* scalars,
                                      const size_t num_initial_points,
                                      pippenger_runtime_state& state)
{
    compute_wnaf_states(state.point_schedule, state.skew_table, state.round_counts, scalars, num_initial_points);
    organize_buckets(state.point_schedule, state.round_counts, num_initial_points * 2);
    g1::element result = scalar_multiplication_internal(state, points, num_initial_points * 2);
    return result;
}

// TODO: this is a lot of code duplication, need to fix that once the method has stabilized
inline g1::element unsafe_scalar_multiplication_internal(unsafe_pippenger_runtime_state& state,
                                                         g1::affine_element* points,
                                                         const size_t num_points)
{
    const size_t num_rounds = get_num_rounds(num_points);
#ifndef NO_MULTITHREADING
    const size_t num_threads = static_cast<size_t>(omp_get_max_threads());
#else
    const size_t num_threads = 1;
#endif
    const size_t bits_per_bucket = get_optimal_bucket_width(num_points / 2);

    g1::element* thread_accumulators = static_cast<g1::element*>(aligned_alloc(64, num_threads * sizeof(g1::element)));

#ifndef NO_MULTITHREADING
#pragma omp parallel for
#endif
    for (size_t j = 0; j < num_threads; ++j) {
        thread_accumulators[j].self_set_infinity();

        for (size_t i = 0; i < num_rounds; ++i) {

            const uint64_t num_round_points = state.round_counts[i];

            g1::element accumulator;
            accumulator.self_set_infinity();

            if ((num_round_points == 0) || (num_round_points < num_threads && j != num_threads - 1)) {
            } else {

                const uint64_t num_round_points_per_thread = num_round_points / num_threads;
                const uint64_t leftovers =
                    (j == num_threads - 1) ? (num_round_points) - (num_round_points_per_thread * num_threads) : 0;

                uint64_t* thread_point_schedule =
                    &state.point_schedule[(i * num_points) + j * num_round_points_per_thread];
                const size_t first_bucket = thread_point_schedule[0] & 0x7fffffffU;
                const size_t last_bucket =
                    thread_point_schedule[(num_round_points_per_thread - 1 + leftovers)] & 0x7fffffffU;
                const size_t num_thread_buckets = (last_bucket - first_bucket) + 1;

                affine_product_runtime_state product_state = state.get_affine_product_runtime_state(num_threads, j);
                product_state.num_points = static_cast<uint32_t>(num_round_points_per_thread + leftovers);
                product_state.points = points;
                product_state.point_schedule = thread_point_schedule;
                product_state.num_buckets = static_cast<uint32_t>(num_thread_buckets);
                g1::affine_element* output_buckets = reduce_buckets(product_state, true);
                g1::element running_sum;
                running_sum.self_set_infinity();

                // one nice side-effect of the affine trick, is that half of the bucket concatenation
                // algorithm can use mixed addition formulae, instead of full addition formulae
                size_t output_it = product_state.num_points - 1;
                for (size_t k = num_thread_buckets - 1; k > 0; --k) {
                    if (__builtin_expect(!product_state.bucket_empty_status[k], 1)) {
                        running_sum += (output_buckets[output_it]);
                        --output_it;
                    }
                    accumulator += running_sum;
                }
                running_sum += output_buckets[0];
                accumulator.self_dbl();
                accumulator += running_sum;

                // we now need to scale up 'running sum' up to the value of the first bucket.
                // e.g. if first bucket is 0, no scaling
                // if first bucket is 1, we need to add (2 * running_sum)
                if (first_bucket > 0) {
                    uint32_t multiplier = static_cast<uint32_t>(first_bucket << 1UL);
                    size_t shift = numeric::get_msb(multiplier);
                    g1::element rolling_accumulator = g1::point_at_infinity;
                    bool init = false;
                    while (shift != static_cast<size_t>(-1)) {
                        if (init) {
                            rolling_accumulator.self_dbl();
                            if (((multiplier >> shift) & 1)) {
                                rolling_accumulator += running_sum;
                            }
                        } else {
                            rolling_accumulator += running_sum;
                        }
                        init = true;
                        shift -= 1;
                    }
                    accumulator += rolling_accumulator;
                }
            }

            if (i == (num_rounds - 1)) {
                const size_t num_points_per_thread = num_points / num_threads;
                bool* skew_table = &state.skew_table[j * num_points_per_thread];
                g1::affine_element* point_table = &points[j * num_points_per_thread];
                g1::affine_element addition_temporary;
                for (size_t k = 0; k < num_points_per_thread; ++k) {
                    if (skew_table[k]) {
                        addition_temporary = -point_table[k];
                        accumulator += addition_temporary;
                    }
                }
            }

            if (i > 0) {
                for (size_t k = 0; k < bits_per_bucket + 1; ++k) {
                    thread_accumulators[j].self_dbl();
                }
            }
            thread_accumulators[j] += accumulator;
        }
    }

    g1::element result;
    result.self_set_infinity();
    for (size_t i = 0; i < num_threads; ++i) {
        result += thread_accumulators[i];
    }
    free(thread_accumulators);
    return result;
}

inline g1::element pippenger_unsafe_internal(g1::affine_element* points,
                                             fr* scalars,
                                             const size_t num_initial_points,
                                             unsafe_pippenger_runtime_state& state)
{
    // multiplication_runtime_state state;
    compute_wnaf_states(state.point_schedule, state.skew_table, state.round_counts, scalars, num_initial_points);
    organize_buckets(state.point_schedule, state.round_counts, num_initial_points * 2);
    g1::element result = unsafe_scalar_multiplication_internal(state, points, num_initial_points * 2);
    return result;
}

g1::element pippenger(fr* scalars,
                      g1::affine_element* points,
                      const size_t num_initial_points,
                      pippenger_runtime_state& state)
{
    // our windowed non-adjacent form algorthm requires that each thread can work on at least 8 points.
    // If we fall below this theshold, fall back to the traditional scalar multiplication algorithm.
    // For 8 threads, this neatly coincides with the threshold where Strauss scalar multiplication outperforms Pippenger
#ifndef NO_MULTITHREADING
    const size_t threshold = std::max(static_cast<size_t>(omp_get_max_threads() * 8), 8UL);
#else
    const size_t threshold = 8UL;
#endif

    if (num_initial_points == 0) {
        g1::element out = g1::one;
        out.self_set_infinity();
        return out;
    }

    if (num_initial_points <= threshold) {
        std::vector<g1::element> exponentiation_results(num_initial_points);
        // might as well multithread this...
        // TODO: implement Strauss algorithm for small numbers of points.
#ifndef NO_MULTITHREADING
#pragma omp parallel for
#endif
        for (size_t i = 0; i < num_initial_points; ++i) {
            exponentiation_results[i] = g1::element(points[i * 2]) * scalars[i];
        }

        for (size_t i = num_initial_points - 1; i > 0; --i) {
            exponentiation_results[i - 1] += exponentiation_results[i];
        }
        return exponentiation_results[0];
    }

    const size_t slice_bits = static_cast<size_t>(numeric::get_msb(static_cast<uint64_t>(num_initial_points)));
    const size_t num_slice_points = static_cast<size_t>(1ULL << slice_bits);

    g1::element result = pippenger_internal(points, scalars, num_slice_points, state);

    if (num_slice_points != num_initial_points) {
        const uint64_t leftover_points = num_initial_points - num_slice_points;
        return result + pippenger(scalars + num_slice_points, points + (num_slice_points * 2), leftover_points, state);
    } else {
        return result;
    }
}

/**
 * It's pippenger! But this one has go-faster stripes and a prediliction for questionable life choices.
 * We use affine-addition formula in this method, which paradoxically is ~45% faster than the mixed addition formulae.
 * See `scalar_multiplication.cpp` for a more detailed description.
 *
 * It's...unsafe, because we assume that the incomplete addition formula exceptions are not triggered.
 * We don't bother to check for this to avoid conditional branches in a critical section of our code.
 * This is fine for situations where your bases are linearly independent (i.e. KZG10 polynomial commitments),
 * because triggering the incomplete addition exceptions is about as hard as solving the disrete log problem.
 *
 * This is ok for the prover, but GIANT RED CLAXON WARNINGS FOR THE VERIFIER
 * Don't use this in a verification algorithm! That would be a really bad idea.
 * Unless you're a malicious adversary, then it would be a great idea!
 *
 **/
g1::element pippenger_unsafe(fr* scalars,
                             g1::affine_element* points,
                             const size_t num_initial_points,
                             unsafe_pippenger_runtime_state& state)
{
    // our windowed non-adjacent form algorthm requires that each thread can work on at least 8 points.
    // If we fall below this theshold, fall back to the traditional scalar multiplication algorithm.
    // For 8 threads, this neatly coincides with the threshold where Strauss scalar multiplication outperforms Pippenger
#ifndef NO_MULTITHREADING
    const size_t threshold = std::max(static_cast<size_t>(omp_get_max_threads() * 8), 8UL);
#else
    const size_t threshold = 8UL;
#endif

    if (num_initial_points == 0) {
        g1::element out = g1::one;
        out.self_set_infinity();
        return out;
    }

    if (num_initial_points <= threshold) {
        std::vector<g1::element> exponentiation_results(num_initial_points);
        // might as well multithread this...
        // TODO: implement Strauss algorithm for small numbers of points.
#ifndef NO_MULTITHREADING
#pragma omp parallel for
#endif
        for (size_t i = 0; i < num_initial_points; ++i) {
            exponentiation_results[i] = g1::element(points[i * 2]) * scalars[i];
        }

        for (size_t i = num_initial_points - 1; i > 0; --i) {
            exponentiation_results[i - 1] += exponentiation_results[i];
        }
        return exponentiation_results[0];
    }

    const size_t slice_bits = static_cast<size_t>(numeric::get_msb(static_cast<uint64_t>(num_initial_points)));
    const size_t num_slice_points = static_cast<size_t>(1ULL << slice_bits);

    g1::element result = pippenger_unsafe_internal(points, scalars, num_slice_points, state);

    if (num_slice_points != num_initial_points) {
        const uint64_t leftover_points = num_initial_points - num_slice_points;
        return result +
               pippenger_unsafe(scalars + num_slice_points, points + (num_slice_points * 2), leftover_points, state);
    } else {
        return result;
    }
}
} // namespace scalar_multiplication
} // namespace barretenberg