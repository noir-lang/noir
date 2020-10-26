#include "./scalar_multiplication.hpp"

#include "../../../groups/wnaf.hpp"
#include "../fq.hpp"
#include "../fr.hpp"
#include "../g1.hpp"
#include "./process_buckets.hpp"
#include "./runtime_states.hpp"

#include <common/mem.hpp>
#include <common/max_threads.hpp>
#include <numeric/bitop/get_msb.hpp>

#include <array>
#include <cstddef>
#include <cstdint>
#include <cstdlib>

#ifndef NO_MULTITHREADING
#include <omp.h>
#endif

#define BBERG_SCALAR_MULTIPLICATION_FETCH_BLOCK                                                                        \
    __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 16] >> 32ULL));                              \
    __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 17] >> 32ULL));                              \
    __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 18] >> 32ULL));                              \
    __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 19] >> 32ULL));                              \
    __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 20] >> 32ULL));                              \
    __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 21] >> 32ULL));                              \
    __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 22] >> 32ULL));                              \
    __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 23] >> 32ULL));                              \
    __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 24] >> 32ULL));                              \
    __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 25] >> 32ULL));                              \
    __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 26] >> 32ULL));                              \
    __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 27] >> 32ULL));                              \
    __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 28] >> 32ULL));                              \
    __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 29] >> 32ULL));                              \
    __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 30] >> 32ULL));                              \
    __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 31] >> 32ULL));                              \
                                                                                                                       \
    uint64_t schedule_a = state.point_schedule[schedule_it];                                                           \
    uint64_t schedule_b = state.point_schedule[schedule_it + 1];                                                       \
    uint64_t schedule_c = state.point_schedule[schedule_it + 2];                                                       \
    uint64_t schedule_d = state.point_schedule[schedule_it + 3];                                                       \
    uint64_t schedule_e = state.point_schedule[schedule_it + 4];                                                       \
    uint64_t schedule_f = state.point_schedule[schedule_it + 5];                                                       \
    uint64_t schedule_g = state.point_schedule[schedule_it + 6];                                                       \
    uint64_t schedule_h = state.point_schedule[schedule_it + 7];                                                       \
    uint64_t schedule_i = state.point_schedule[schedule_it + 8];                                                       \
    uint64_t schedule_j = state.point_schedule[schedule_it + 9];                                                       \
    uint64_t schedule_k = state.point_schedule[schedule_it + 10];                                                      \
    uint64_t schedule_l = state.point_schedule[schedule_it + 11];                                                      \
    uint64_t schedule_m = state.point_schedule[schedule_it + 12];                                                      \
    uint64_t schedule_n = state.point_schedule[schedule_it + 13];                                                      \
    uint64_t schedule_o = state.point_schedule[schedule_it + 14];                                                      \
    uint64_t schedule_p = state.point_schedule[schedule_it + 15];                                                      \
                                                                                                                       \
    g1::conditional_negate_affine(                                                                                     \
        state.points + (schedule_a >> 32ULL), state.point_pairs_1 + current_offset, (schedule_a >> 31ULL) & 1ULL);     \
    g1::conditional_negate_affine(                                                                                     \
        state.points + (schedule_b >> 32ULL), state.point_pairs_1 + current_offset + 1, (schedule_b >> 31ULL) & 1ULL); \
    g1::conditional_negate_affine(                                                                                     \
        state.points + (schedule_c >> 32ULL), state.point_pairs_1 + current_offset + 2, (schedule_c >> 31ULL) & 1ULL); \
    g1::conditional_negate_affine(                                                                                     \
        state.points + (schedule_d >> 32ULL), state.point_pairs_1 + current_offset + 3, (schedule_d >> 31ULL) & 1ULL); \
    g1::conditional_negate_affine(                                                                                     \
        state.points + (schedule_e >> 32ULL), state.point_pairs_1 + current_offset + 4, (schedule_e >> 31ULL) & 1ULL); \
    g1::conditional_negate_affine(                                                                                     \
        state.points + (schedule_f >> 32ULL), state.point_pairs_1 + current_offset + 5, (schedule_f >> 31ULL) & 1ULL); \
    g1::conditional_negate_affine(                                                                                     \
        state.points + (schedule_g >> 32ULL), state.point_pairs_1 + current_offset + 6, (schedule_g >> 31ULL) & 1ULL); \
    g1::conditional_negate_affine(                                                                                     \
        state.points + (schedule_h >> 32ULL), state.point_pairs_1 + current_offset + 7, (schedule_h >> 31ULL) & 1ULL); \
    g1::conditional_negate_affine(                                                                                     \
        state.points + (schedule_i >> 32ULL), state.point_pairs_1 + current_offset + 8, (schedule_i >> 31ULL) & 1ULL); \
    g1::conditional_negate_affine(                                                                                     \
        state.points + (schedule_j >> 32ULL), state.point_pairs_1 + current_offset + 9, (schedule_j >> 31ULL) & 1ULL); \
    g1::conditional_negate_affine(state.points + (schedule_k >> 32ULL),                                                \
                                  state.point_pairs_1 + current_offset + 10,                                           \
                                  (schedule_k >> 31ULL) & 1ULL);                                                       \
    g1::conditional_negate_affine(state.points + (schedule_l >> 32ULL),                                                \
                                  state.point_pairs_1 + current_offset + 11,                                           \
                                  (schedule_l >> 31ULL) & 1ULL);                                                       \
    g1::conditional_negate_affine(state.points + (schedule_m >> 32ULL),                                                \
                                  state.point_pairs_1 + current_offset + 12,                                           \
                                  (schedule_m >> 31ULL) & 1ULL);                                                       \
    g1::conditional_negate_affine(state.points + (schedule_n >> 32ULL),                                                \
                                  state.point_pairs_1 + current_offset + 13,                                           \
                                  (schedule_n >> 31ULL) & 1ULL);                                                       \
    g1::conditional_negate_affine(state.points + (schedule_o >> 32ULL),                                                \
                                  state.point_pairs_1 + current_offset + 14,                                           \
                                  (schedule_o >> 31ULL) & 1ULL);                                                       \
    g1::conditional_negate_affine(state.points + (schedule_p >> 32ULL),                                                \
                                  state.point_pairs_1 + current_offset + 15,                                           \
                                  (schedule_p >> 31ULL) & 1ULL);                                                       \
                                                                                                                       \
    current_offset += 16;                                                                                              \
    schedule_it += 16;

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
    const size_t num_threads = max_threads::compute_num_threads();
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

/**
 * adds a bunch of points together using affine addition formulae.
 * Paradoxically, the affine formula is crazy efficient if you have a lot of independent point additiosn to perform.
 * Affine formula:
 *
 * \lambda = (y_2 - y_1) / (x_2 - x_1)
 * x_3 = \lambda^2 - (x_2 + x_1)
 * y_3 = \lambda*(x_1 - x_3) - y_1
 *
 * Traditionally, we avoid affine formulae like the plague, because computing lambda requires a modular inverse,
 * which is outrageously expensive.
 *
 * However! We can use Montgomery's batch inversion technique to amortise the cost of the inversion to ~0.
 *
 * The way batch inversion works is as follows. Let's say you want to compute \{ 1/x_1, 1/x_2, ..., 1/x_n \}
 * The trick is to compute the product x_1x_2...x_n , whilst storing all of the temporary products.
 * i.e. we have an array A = [x_1, x_1x_2, ..., x_1x_2...x_n]
 * We then compute a single inverse: I = 1 / x_1x_2...x_n
 * Finally, we can use our accumulated products, to quotient out individual inverses.
 * We can get an individual inverse at index i, by computing I.A_{i-1}.(x_nx_n-1...x_i+1)
 * The last product term we can compute on-the-fly, as it grows by one element for each additional inverse that we
 * require.
 *
 * TLDR: amortized cost of a modular inverse is 3 field multiplications per inverse.
 * Which means we can compute a point addition with SIX field multiplications in total.
 * The traditional Jacobian-coordinate formula requires 11.
 *
 * There is a catch though - we need large sequences of independent point additions!
 * i.e. the output from one point addition in the sequence is NOT an input to any other point addition in the sequence.
 *
 * We can re-arrange the Pippenger algorithm to get this property, but it's...complicated
 **/
void add_affine_points(g1::affine_element* points, const size_t num_points, fq* scratch_space)
{
    fq batch_inversion_accumulator = fq::one();

    for (size_t i = 0; i < num_points; i += 2) {
        scratch_space[i >> 1] = points[i].x + points[i + 1].x; // x2 + x1
        points[i + 1].x -= points[i].x;                        // x2 - x1
        points[i + 1].y -= points[i].y;                        // y2 - y1
        points[i + 1].y *= batch_inversion_accumulator;        // (y2 - y1)*accumulator_old
        batch_inversion_accumulator *= (points[i + 1].x);
    }
    batch_inversion_accumulator = batch_inversion_accumulator.invert();

    for (size_t i = (num_points)-2; i < num_points; i -= 2) {
        // Memory bandwidth is a bit of a bottleneck here.
        // There's probably a more elegant way of structuring our data so we don't need to do all of this prefetching
        __builtin_prefetch(points + i - 2);
        __builtin_prefetch(points + i - 1);
        __builtin_prefetch(points + ((i + num_points - 2) >> 1));
        __builtin_prefetch(scratch_space + ((i - 2) >> 1));

        points[i + 1].y *= batch_inversion_accumulator; // update accumulator
        batch_inversion_accumulator *= points[i + 1].x;
        points[i + 1].x = points[i + 1].y.sqr();
        points[(i + num_points) >> 1].x = points[i + 1].x - (scratch_space[i >> 1]); // x3 = lambda_squared - x2
                                                                                     // - x1
        points[i].x -= points[(i + num_points) >> 1].x;
        points[i].x *= points[i + 1].y;
        points[(i + num_points) >> 1].y = points[i].x - points[i].y;
    }
}

void add_affine_points_with_edge_cases(g1::affine_element* points, const size_t num_points, fq* scratch_space)
{
    fq batch_inversion_accumulator = fq::one();

    for (size_t i = 0; i < num_points; i += 2) {
        if (points[i].is_point_at_infinity() || points[i + 1].is_point_at_infinity()) {
            continue;
        }
        if (points[i].x == points[i + 1].x) {
            if (points[i].y == points[i + 1].y) {
                // double
                scratch_space[i >> 1] = points[i].x + points[i].x; // 2x
                fq x_squared = points[i].x.sqr();
                points[i + 1].x = points[i].y + points[i].y;         // 2y
                points[i + 1].y = x_squared + x_squared + x_squared; // 3x^2
                points[i + 1].y *= batch_inversion_accumulator;
                batch_inversion_accumulator *= (points[i + 1].x);
                continue;
            }
            points[i].self_set_infinity();
            points[i + 1].self_set_infinity();
            continue;
        }

        scratch_space[i >> 1] = points[i].x + points[i + 1].x; // x2 + x1
        points[i + 1].x -= points[i].x;                        // x2 - x1
        points[i + 1].y -= points[i].y;                        // y2 - y1
        points[i + 1].y *= batch_inversion_accumulator;        // (y2 - y1)*accumulator_old
        batch_inversion_accumulator *= (points[i + 1].x);
    }
    batch_inversion_accumulator = batch_inversion_accumulator.invert();
    for (size_t i = (num_points)-2; i < num_points; i -= 2) {
        // Memory bandwidth is a bit of a bottleneck here.
        // There's probably a more elegant way of structuring our data so we don't need to do all of this prefetching
        __builtin_prefetch(points + i - 2);
        __builtin_prefetch(points + i - 1);
        __builtin_prefetch(points + ((i + num_points - 2) >> 1));
        __builtin_prefetch(scratch_space + ((i - 2) >> 1));

        if (points[i].is_point_at_infinity()) {
            points[(i + num_points) >> 1] = points[i + 1];
            continue;
        }
        if (points[i + 1].is_point_at_infinity()) {
            points[(i + num_points) >> 1] = points[i];
            continue;
        }

        points[i + 1].y *= batch_inversion_accumulator; // update accumulator
        batch_inversion_accumulator *= points[i + 1].x;
        points[i + 1].x = points[i + 1].y.sqr();
        points[(i + num_points) >> 1].x = points[i + 1].x - (scratch_space[i >> 1]); // x3 = lambda_squared - x2
                                                                                     // - x1
        points[i].x -= points[(i + num_points) >> 1].x;
        points[i].x *= points[i + 1].y;
        points[(i + num_points) >> 1].y = points[i].x - points[i].y;
    }
}

/**
 * evaluate a chain of pairwise additions.
 * The additions are sequenced into base-2 segments
 * i.e. pairs, pairs of pairs, pairs of pairs of pairs etc
 * `max_bucket_bits` indicates the largest set of nested pairs in the array,
 * which defines the iteration depth
 **/
void evaluate_addition_chains(affine_product_runtime_state& state, const size_t max_bucket_bits, bool handle_edge_cases)
{
    size_t end = state.num_points;
    size_t start = 0;
    for (size_t i = 0; i < max_bucket_bits; ++i) {
        const size_t points_in_round = (state.num_points - state.bit_offsets[i + 1]) >> (i);
        start = end - points_in_round;
        if (handle_edge_cases) {
            add_affine_points_with_edge_cases(state.point_pairs_1 + start, points_in_round, state.scratch_space);
        } else {
            add_affine_points(state.point_pairs_1 + start, points_in_round, state.scratch_space);
        }
    }
}

/**
 * This is the entry point for our 'find a way of evaluating a giant multi-product using affine coordinates' algorithm
 * By this point, we have already sorted our pippenger buckets. So we have the following situation:
 *
 * 1. We have a defined number of buckets points
 * 2. We have a defined number of points, that need to be added into these bucket points
 * 3. number of points >> number of buckets
 *
 * The algorithm begins by counting the number of points assigned to each bucket.
 * For each bucket, we then take this count and split it into its base-2 components.
 * e.g. if bucket[3] has 14 points, we split that into a sequence of (8, 4, 2)
 * This base-2 splitting is useful, because we can take the bucket's associated points, and
 * sort them into pairs, quads, octs etc. These mini-addition sequences are independent from one another,
 * which means that we can use the affine trick to evaluate them.
 * Once we're done, we have effectively reduced the number of points in the bucket to a logarithmic factor of the input.
 * e.g. in the above example, once we've evaluated our pairwise addition of 8, 4 and 2 elements,
 *      we're left with 3 points.
 * The next step is to 'play it again Sam', and recurse back into `reduce_buckets`, with our reduced number of points.
 * We repeat this process until every bucket only has one point assigned to it.
 **/
g1::affine_element* reduce_buckets(affine_product_runtime_state& state, bool first_round, bool handle_edge_cases)
{

    // std::chrono::steady_clock::time_point time_start = std::chrono::steady_clock::now();
    // This method sorts our points into our required base-2 sequences.
    // `max_bucket_bits` is log2(maximum bucket count).
    // This sets the upper limit on how many iterations we need to perform in `evaluate_addition_chains`.
    // e.g. if `max_bucket_bits == 3`, then we have at least one bucket with >= 8 points in it.
    // which means we need to repeat our pairwise addition algorithm 3 times
    // (e.g. add 4 pairs together to get 2 pairs, add those pairs together to get a single pair, which we add to reduce
    // to our final point)
    const size_t max_bucket_bits = construct_addition_chains(state, first_round);

    // if max_bucket_bits is 0, we're done! we can return
    if (max_bucket_bits == 0) {
        return state.point_pairs_1;
    }

    // compute our required additions using the affine trick
    evaluate_addition_chains(state, max_bucket_bits, handle_edge_cases);

    // this next step is a processing step, that computes a new point schedule for our reduced points.
    // In the pippenger algorithm, we use a 64-bit uint to categorize each point.
    // The high 32 bits describes the position of the point in a point array.
    // The low 31 bits describes the bucket index that the point maps to
    // The 32nd bit defines whether the point is actually a negation of our stored point.

    // We want to compute these 'point schedule' uints for our reduced points, so that we can recurse back into
    // `reduce_buckets`
    uint32_t start = 0;
    const uint32_t end = static_cast<uint32_t>(state.num_points);
    // The output of `evaluate_addition_chains` has a bit of an odd structure, should probably refactor.
    // Effectively, we used to have one big 1d array, and the act of computing these pair-wise point additions
    // has chopped it up into sequences of smaller 1d arrays, with gaps in between
    for (size_t i = 0; i < max_bucket_bits; ++i) {
        const uint32_t points_in_round =
            (static_cast<uint32_t>(state.num_points) - state.bit_offsets[i + 1]) >> static_cast<uint32_t>(i);
        const uint32_t points_removed = points_in_round / 2;

        start = end - points_in_round;
        const uint32_t modified_start = start + points_removed;
        state.bit_offsets[i + 1] = modified_start;
    }

    // iterate over each bucket. Identify how many remaining points there are, and compute their point scheduels
    uint32_t new_num_points = 0;
    for (size_t i = 0; i < state.num_buckets; ++i) {
        uint32_t& count = state.bucket_counts[i];
        uint32_t num_bits = numeric::get_msb(count) + 1;
        uint32_t new_bucket_count = 0;
        for (size_t j = 0; j < num_bits; ++j) {
            uint32_t& current_offset = state.bit_offsets[j];
            const bool has_entry = ((count >> j) & 1) == 1;
            if (has_entry) {
                uint64_t schedule = (static_cast<uint64_t>(current_offset) << 32ULL) + i;
                state.point_schedule[new_num_points++] = schedule;
                ++new_bucket_count;
                ++current_offset;
            }
        }
        count = new_bucket_count;
    }

    // modify `num_points` to reflect the new number of reduced points.
    // also swap around the `point_pairs` pointer; what used to be our temporary array
    // has now become our input point array
    g1::affine_element* temp = state.point_pairs_1;
    state.num_points = new_num_points;
    state.points = state.point_pairs_1;
    state.point_pairs_1 = state.point_pairs_2;
    state.point_pairs_2 = temp;

    // We could probably speed this up by unroling the recursion.
    // But each extra call to `reduce_buckets` has an input size that is ~log(previous input size)
    // so the extra run-time is meh
    return reduce_buckets(state, false, handle_edge_cases);
}

uint32_t construct_addition_chains(affine_product_runtime_state& state, bool empty_bucket_counts)
{
    // if this is the first call to `construct_addition_chains`, we need to count up our buckets
    if (empty_bucket_counts) {
        memset((void*)state.bucket_counts, 0x00, sizeof(uint32_t) * state.num_buckets);
        const uint32_t first_bucket = static_cast<uint32_t>(state.point_schedule[0] & 0x7fffffffUL);
        for (size_t i = 0; i < state.num_points; ++i) {
            size_t bucket_index = static_cast<size_t>(state.point_schedule[i] & 0x7fffffffUL);
            ++state.bucket_counts[bucket_index - first_bucket];
        }
        for (size_t i = 0; i < state.num_buckets; ++i) {
            state.bucket_empty_status[i] = (state.bucket_counts[i] == 0);
        }
    }

    uint32_t max_count = 0;
    for (size_t i = 0; i < state.num_buckets; ++i) {
        max_count = state.bucket_counts[i] > max_count ? state.bucket_counts[i] : max_count;
    }

    const uint32_t max_bucket_bits = numeric::get_msb(max_count);

    for (size_t i = 0; i < max_bucket_bits + 1; ++i) {
        state.bit_offsets[i] = 0;
    }

    // TODO: measure whether this is useful. `count_bits` has a nasty nested loop that,
    // theoretically, can be unrolled using templated methods.
    // However, explicitly unrolling the loop by using recursive template calls was slower!
    // Inner loop is currently bounded by a constexpr variable, need to see what the compiler does with that...
    count_bits(state.bucket_counts, &state.bit_offsets[0], state.num_buckets, max_bucket_bits);

    // we need to update `bit_offsets` to compute our point shuffle,
    // but we need the original array later on, so make a copy.
    std::array<uint32_t, 22> bit_offsets_copy = { 0 };
    for (size_t i = 0; i < max_bucket_bits + 1; ++i) {
        bit_offsets_copy[i] = state.bit_offsets[i];
    }

    // this is where we take each bucket's associated points, and arrange them
    // in a pairwise order, so that we can compute large sequences of additions using the affine trick
    size_t schedule_it = 0;
    uint32_t* bucket_count_it = state.bucket_counts;

    for (size_t i = 0; i < state.num_buckets; ++i) {
        uint32_t count = *bucket_count_it;
        ++bucket_count_it;
        uint32_t num_bits = numeric::get_msb(count) + 1;
        for (size_t j = 0; j < num_bits; ++j) {
            uint32_t& current_offset = bit_offsets_copy[j];
            const size_t k_end = count & (1UL << j);
            // This section is a bottleneck - to populate our point array, we need
            // to read from memory locations that are effectively uniformly randomly distributed!
            // (assuming our scalar multipliers are uniformly random...)
            // In the absence of a more elegant solution, we use ugly macro hacks to try and
            // unroll loops, and prefetch memory a few cycles before we need it
            switch (k_end) {
            case 64: {
                [[fallthrough]];
            }
            case 32: {
                [[fallthrough]];
            }
            case 16: {
                for (size_t k = 0; k < (k_end >> 4); ++k) {
                    BBERG_SCALAR_MULTIPLICATION_FETCH_BLOCK;
                }
                break;
            }
            case 8: {
                __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 8] >> 32ULL));
                __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 9] >> 32ULL));
                __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 10] >> 32ULL));
                __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 11] >> 32ULL));
                __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 12] >> 32ULL));
                __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 13] >> 32ULL));
                __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 14] >> 32ULL));
                __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 15] >> 32ULL));

                const uint64_t schedule_a = state.point_schedule[schedule_it];
                const uint64_t schedule_b = state.point_schedule[schedule_it + 1];
                const uint64_t schedule_c = state.point_schedule[schedule_it + 2];
                const uint64_t schedule_d = state.point_schedule[schedule_it + 3];
                const uint64_t schedule_e = state.point_schedule[schedule_it + 4];
                const uint64_t schedule_f = state.point_schedule[schedule_it + 5];
                const uint64_t schedule_g = state.point_schedule[schedule_it + 6];
                const uint64_t schedule_h = state.point_schedule[schedule_it + 7];

                g1::conditional_negate_affine(state.points + (schedule_a >> 32ULL),
                                              state.point_pairs_1 + current_offset,
                                              (schedule_a >> 31ULL) & 1ULL);
                g1::conditional_negate_affine(state.points + (schedule_b >> 32ULL),
                                              state.point_pairs_1 + current_offset + 1,
                                              (schedule_b >> 31ULL) & 1ULL);
                g1::conditional_negate_affine(state.points + (schedule_c >> 32ULL),
                                              state.point_pairs_1 + current_offset + 2,
                                              (schedule_c >> 31ULL) & 1ULL);
                g1::conditional_negate_affine(state.points + (schedule_d >> 32ULL),
                                              state.point_pairs_1 + current_offset + 3,
                                              (schedule_d >> 31ULL) & 1ULL);
                g1::conditional_negate_affine(state.points + (schedule_e >> 32ULL),
                                              state.point_pairs_1 + current_offset + 4,
                                              (schedule_e >> 31ULL) & 1ULL);
                g1::conditional_negate_affine(state.points + (schedule_f >> 32ULL),
                                              state.point_pairs_1 + current_offset + 5,
                                              (schedule_f >> 31ULL) & 1ULL);
                g1::conditional_negate_affine(state.points + (schedule_g >> 32ULL),
                                              state.point_pairs_1 + current_offset + 6,
                                              (schedule_g >> 31ULL) & 1ULL);
                g1::conditional_negate_affine(state.points + (schedule_h >> 32ULL),
                                              state.point_pairs_1 + current_offset + 7,
                                              (schedule_h >> 31ULL) & 1ULL);

                current_offset += 8;
                schedule_it += 8;
                break;
            }
            case 4: {
                __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 4] >> 32ULL));
                __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 5] >> 32ULL));
                __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 6] >> 32ULL));
                __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 7] >> 32ULL));
                const uint64_t schedule_a = state.point_schedule[schedule_it];
                const uint64_t schedule_b = state.point_schedule[schedule_it + 1];
                const uint64_t schedule_c = state.point_schedule[schedule_it + 2];
                const uint64_t schedule_d = state.point_schedule[schedule_it + 3];

                g1::conditional_negate_affine(state.points + (schedule_a >> 32ULL),
                                              state.point_pairs_1 + current_offset,
                                              (schedule_a >> 31ULL) & 1ULL);
                g1::conditional_negate_affine(state.points + (schedule_b >> 32ULL),
                                              state.point_pairs_1 + current_offset + 1,
                                              (schedule_b >> 31ULL) & 1ULL);
                g1::conditional_negate_affine(state.points + (schedule_c >> 32ULL),
                                              state.point_pairs_1 + current_offset + 2,
                                              (schedule_c >> 31ULL) & 1ULL);
                g1::conditional_negate_affine(state.points + (schedule_d >> 32ULL),
                                              state.point_pairs_1 + current_offset + 3,
                                              (schedule_d >> 31ULL) & 1ULL);
                current_offset += 4;
                schedule_it += 4;
                break;
            }
            case 2: {
                __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 4] >> 32ULL));
                __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 5] >> 32ULL));
                __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 6] >> 32ULL));
                __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 7] >> 32ULL));
                const uint64_t schedule_a = state.point_schedule[schedule_it];
                const uint64_t schedule_b = state.point_schedule[schedule_it + 1];

                g1::conditional_negate_affine(state.points + (schedule_a >> 32ULL),
                                              state.point_pairs_1 + current_offset,
                                              (schedule_a >> 31ULL) & 1ULL);
                g1::conditional_negate_affine(state.points + (schedule_b >> 32ULL),
                                              state.point_pairs_1 + current_offset + 1,
                                              (schedule_b >> 31ULL) & 1ULL);
                current_offset += 2;
                schedule_it += 2;
                break;
            }
            case 1: {
                __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 4] >> 32ULL));
                __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 5] >> 32ULL));
                __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 6] >> 32ULL));
                __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 7] >> 32ULL));
                const uint64_t schedule_a = state.point_schedule[schedule_it];

                g1::conditional_negate_affine(state.points + (schedule_a >> 32ULL),
                                              state.point_pairs_1 + current_offset,
                                              (schedule_a >> 31ULL) & 1ULL);
                ++current_offset;
                ++schedule_it;
                break;
            }
            case 0: {
                break;
            }
            default: {
                for (size_t k = 0; k < k_end; ++k) {
                    uint64_t schedule = state.point_schedule[schedule_it];
                    __builtin_prefetch(state.points + (state.point_schedule[schedule_it + 1] >> 32ULL));

                    const uint64_t predicate = (schedule >> 31UL) & 1UL;

                    g1::conditional_negate_affine(
                        state.points + (schedule >> 32ULL), state.point_pairs_1 + current_offset, predicate);
                    ++current_offset;
                    ++schedule_it;
                }
            }
            }
        }
    }
    return max_bucket_bits;
}

g1::element evaluate_pippenger_rounds(pippenger_runtime_state& state,
                                      g1::affine_element* points,
                                      const size_t num_points,
                                      bool handle_edge_cases)
{
    const size_t num_rounds = get_num_rounds(num_points);
#ifndef NO_MULTITHREADING
    const size_t num_threads = max_threads::compute_num_threads();
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
                g1::affine_element* output_buckets = reduce_buckets(product_state, true, handle_edge_cases);
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

g1::element pippenger_internal(g1::affine_element* points,
                               fr* scalars,
                               const size_t num_initial_points,
                               pippenger_runtime_state& state,
                               bool handle_edge_cases)
{
    // multiplication_runtime_state state;
    compute_wnaf_states(state.point_schedule, state.skew_table, state.round_counts, scalars, num_initial_points);
    organize_buckets(state.point_schedule, state.round_counts, num_initial_points * 2);
    g1::element result = evaluate_pippenger_rounds(state, points, num_initial_points * 2, handle_edge_cases);
    return result;
}

g1::element pippenger(fr* scalars,
                      g1::affine_element* points,
                      const size_t num_initial_points,
                      pippenger_runtime_state& state,
                      bool handle_edge_cases)
{
    // our windowed non-adjacent form algorthm requires that each thread can work on at least 8 points.
    // If we fall below this theshold, fall back to the traditional scalar multiplication algorithm.
    // For 8 threads, this neatly coincides with the threshold where Strauss scalar multiplication outperforms Pippenger
#ifndef NO_MULTITHREADING
    const size_t threshold = std::max(max_threads::compute_num_threads() * 8, 8UL);
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

    g1::element result = pippenger_internal(points, scalars, num_slice_points, state, handle_edge_cases);

    if (num_slice_points != num_initial_points) {
        const uint64_t leftover_points = num_initial_points - num_slice_points;
        return result + pippenger(scalars + num_slice_points,
                                  points + static_cast<size_t>(num_slice_points * 2),
                                  static_cast<size_t>(leftover_points),
                                  state,
                                  handle_edge_cases);
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
                             pippenger_runtime_state& state)
{
    return pippenger(scalars, points, num_initial_points, state, false);
}
} // namespace scalar_multiplication
} // namespace barretenberg