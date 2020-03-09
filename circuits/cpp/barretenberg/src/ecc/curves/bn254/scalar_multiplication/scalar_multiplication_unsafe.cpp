#include "./scalar_multiplication.hpp"
#include <numeric/bitop/get_msb.hpp>
#include <chrono>
#include <iostream>

namespace barretenberg {
namespace scalar_multiplication {

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
    // std::chrono::steady_clock::time_point time_start = std::chrono::steady_clock::now();

    for (size_t i = 0; i < num_points; i += 2) {
        scratch_space[i >> 1] = points[i].x + points[i + 1].x; // x2 + x1
        points[i + 1].x -= points[i].x;                        // x2 - x1
        points[i + 1].y -= points[i].y;                        // y2 - y1
        points[i + 1].y *= batch_inversion_accumulator;        // (y2 - y1)*accumulator_old
        batch_inversion_accumulator *= (points[i + 1].x);
    }
    // std::chrono::steady_clock::time_point time_end = std::chrono::steady_clock::now();
    // std::chrono::microseconds diff = std::chrono::duration_cast<std::chrono::microseconds>(time_end - time_start);
    //     if (num_points > 100000)
    // {
    // std::cout << "forward run time: " << diff.count() << "us" << std::endl;
    // }
    // time_start = std::chrono::steady_clock::now();
    batch_inversion_accumulator = batch_inversion_accumulator.invert();
    // time_end = std::chrono::steady_clock::now();
    //  diff = std::chrono::duration_cast<std::chrono::microseconds>(time_end - time_start);
    // if (num_points > 100000)
    // {
    // std::cout << "invert time: " << diff.count() << "us" << std::endl;
    // }
    // time_start = std::chrono::steady_clock::now();

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

/**
 * evaluate a chain of pairwise additions.
 * The additions are sequenced into base-2 segments
 * i.e. pairs, pairs of pairs, pairs of pairs of pairs etc
 * `max_bucket_bits` indicates the largest set of nested pairs in the array,
 * which defines the iteration depth
 **/
void evaluate_addition_chains(affine_product_runtime_state& state, const size_t max_bucket_bits)
{
    size_t end = state.num_points;
    size_t start = 0;
    for (size_t i = 0; i < max_bucket_bits; ++i) {
        const size_t points_in_round = (state.num_points - state.bit_offsets[i + 1]) >> (i);
        start = end - points_in_round;
        add_affine_points(state.point_pairs_1 + start, points_in_round, state.scratch_space);
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
g1::affine_element* reduce_buckets(affine_product_runtime_state& state, bool first_round)
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
    evaluate_addition_chains(state, max_bucket_bits);

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
    return reduce_buckets(state, false);
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
    switch (max_bucket_bits) {
    case 21: {
        count_bits<21>(state.bucket_counts, &state.bit_offsets[0], state.num_buckets);
        break;
    }
    case 20: {
        count_bits<20>(state.bucket_counts, &state.bit_offsets[0], state.num_buckets);
        break;
    }
    case 19: {
        count_bits<19>(state.bucket_counts, &state.bit_offsets[0], state.num_buckets);
        break;
    }
    case 18: {
        count_bits<18>(state.bucket_counts, &state.bit_offsets[0], state.num_buckets);
        break;
    }
    case 17: {
        count_bits<17>(state.bucket_counts, &state.bit_offsets[0], state.num_buckets);
        break;
    }
    case 16: {
        count_bits<16>(state.bucket_counts, &state.bit_offsets[0], state.num_buckets);
        break;
    }
    case 15: {
        count_bits<15>(state.bucket_counts, &state.bit_offsets[0], state.num_buckets);
        break;
    }
    case 14: {
        count_bits<14>(state.bucket_counts, &state.bit_offsets[0], state.num_buckets);
        break;
    }
    case 13: {
        count_bits<13>(state.bucket_counts, &state.bit_offsets[0], state.num_buckets);
        break;
    }
    case 12: {
        count_bits<12>(state.bucket_counts, &state.bit_offsets[0], state.num_buckets);
        break;
    }
    case 11: {
        count_bits<11>(state.bucket_counts, &state.bit_offsets[0], state.num_buckets);
        break;
    }
    case 10: {
        count_bits<10>(state.bucket_counts, &state.bit_offsets[0], state.num_buckets);
        break;
    }
    case 9: {
        count_bits<9>(state.bucket_counts, &state.bit_offsets[0], state.num_buckets);
        break;
    }
    case 8: {
        count_bits<8>(state.bucket_counts, &state.bit_offsets[0], state.num_buckets);
        break;
    }
    case 7: {
        count_bits<7>(state.bucket_counts, &state.bit_offsets[0], state.num_buckets);
        break;
    }
    case 6: {
        count_bits<6>(state.bucket_counts, &state.bit_offsets[0], state.num_buckets);
        break;
    }
    case 5: {
        count_bits<5>(state.bucket_counts, &state.bit_offsets[0], state.num_buckets);
        break;
    }
    case 4: {
        count_bits<4>(state.bucket_counts, &state.bit_offsets[0], state.num_buckets);
        break;
    }
    case 3: {
        count_bits<3>(state.bucket_counts, &state.bit_offsets[0], state.num_buckets);
        break;
    }
    case 2: {
        count_bits<2>(state.bucket_counts, &state.bit_offsets[0], state.num_buckets);
        break;
    }
    case 1: {
        count_bits<1>(state.bucket_counts, &state.bit_offsets[0], state.num_buckets);
        break;
    }
    default: {
        break;
    };
    }

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
                {
                    BBERG_SCALAR_MULTIPLICATION_FETCH_BLOCK;
                }
                {
                    BBERG_SCALAR_MULTIPLICATION_FETCH_BLOCK;
                }
                {
                    BBERG_SCALAR_MULTIPLICATION_FETCH_BLOCK;
                }
                {
                    BBERG_SCALAR_MULTIPLICATION_FETCH_BLOCK;
                }
                break;
            }
            case 32: {
                {
                    BBERG_SCALAR_MULTIPLICATION_FETCH_BLOCK;
                }
                {
                    BBERG_SCALAR_MULTIPLICATION_FETCH_BLOCK;
                }
                break;
            }
            case 16: {
                BBERG_SCALAR_MULTIPLICATION_FETCH_BLOCK;
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
} // namespace scalar_multiplication
} // namespace barretenberg
