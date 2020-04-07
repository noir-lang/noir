#include "runtime_states.hpp"

#include <common/mem.hpp>

#ifndef NO_MULTITHREADING
#include <omp.h>
#endif

namespace barretenberg {
namespace scalar_multiplication {

pippenger_runtime_state::pippenger_runtime_state(const size_t num_initial_points)
{
    constexpr size_t MAX_NUM_ROUNDS = 256;
    num_points = num_initial_points * 2;
    const size_t num_buckets = static_cast<size_t>(
        1U << barretenberg::scalar_multiplication::get_optimal_bucket_width(static_cast<size_t>(num_initial_points)));
#ifndef NO_MULTITHREADING
    const size_t num_threads = static_cast<size_t>(omp_get_max_threads());
#else
    const size_t num_threads = 1;
#endif
    const size_t prefetch_overflow = 16 * num_threads;
    const size_t num_rounds =
        static_cast<size_t>(barretenberg::scalar_multiplication::get_num_rounds(static_cast<size_t>(num_points)));
    point_schedule = (uint64_t*)(aligned_alloc(64, (static_cast<size_t>(num_points) * num_rounds + prefetch_overflow) * sizeof(uint64_t)));
    skew_table = (bool*)(aligned_alloc(64, static_cast<size_t>(num_points) * sizeof(bool)));
    point_pairs_1 = (g1::affine_element*)(aligned_alloc(
        64, (static_cast<size_t>(num_points) * 2 + (num_threads * 16)) * sizeof(g1::affine_element)));
    point_pairs_2 = (g1::affine_element*)(aligned_alloc(
        64, (static_cast<size_t>(num_points) * 2 + (num_threads * 16)) * sizeof(g1::affine_element)));
    scratch_space = (fq*)(aligned_alloc(64, static_cast<size_t>(num_points) * sizeof(g1::affine_element)));
    bucket_counts = (uint32_t*)(aligned_alloc(64, num_threads * num_buckets * sizeof(uint32_t)));
    bit_counts = (uint32_t*)(aligned_alloc(64, num_threads * num_buckets * sizeof(uint32_t)));
    bucket_empty_status = (bool*)(aligned_alloc(64, num_threads * num_buckets * sizeof(bool)));
    round_counts = (uint64_t*)(aligned_alloc(32, MAX_NUM_ROUNDS * sizeof(uint64_t)));

    const size_t points_per_thread = static_cast<size_t>(num_points) / num_threads;
#ifndef NO_MULTITHREADING
#pragma omp parallel for
#endif
    for (size_t i = 0; i < num_threads; ++i) {
        const size_t thread_offset = i * points_per_thread;
        memset((void*)(point_pairs_1 + thread_offset + (i * 16)),
               0,
               (points_per_thread + 16) * sizeof(g1::affine_element));
        memset((void*)(point_pairs_2 + thread_offset + (i * 16)),
               0,
               (points_per_thread + 16) * sizeof(g1::affine_element));
        memset((void*)(scratch_space + thread_offset), 0, (points_per_thread) * sizeof(fq));
        for (size_t j = 0; j < num_rounds; ++j) {
            const size_t round_offset = (j * static_cast<size_t>(num_points));
            memset((void*)(point_schedule + round_offset + thread_offset), 0, points_per_thread * sizeof(uint64_t));
        }
        memset((void*)(skew_table + thread_offset), 0, points_per_thread * sizeof(bool));
    }

    memset((void*)bucket_counts, 0, num_threads * num_buckets * sizeof(uint32_t));
    memset((void*)bit_counts, 0, num_threads * num_buckets * sizeof(uint32_t));
    memset((void*)bucket_empty_status, 0, num_threads * num_buckets * sizeof(bool));
    memset((void*)round_counts, 0, MAX_NUM_ROUNDS * sizeof(uint64_t));
}

pippenger_runtime_state::pippenger_runtime_state(pippenger_runtime_state&& other)
{
    point_schedule = other.point_schedule;
    skew_table = other.skew_table;
    point_pairs_1 = other.point_pairs_1;
    point_pairs_2 = other.point_pairs_2;
    scratch_space = other.scratch_space;
    bit_counts = other.bit_counts;
    bucket_counts = other.bucket_counts;
    bucket_empty_status = other.bucket_empty_status;
    round_counts = other.round_counts;

    other.point_schedule = nullptr;
    other.skew_table = nullptr;
    other.point_pairs_1 = nullptr;
    other.point_pairs_2 = nullptr;
    other.scratch_space = nullptr;
    other.bit_counts = nullptr;
    other.bucket_counts = nullptr;
    other.bucket_empty_status = nullptr;
    other.round_counts = nullptr;

    num_points = other.num_points;
}

pippenger_runtime_state& pippenger_runtime_state::operator=(pippenger_runtime_state&& other)
{
    if (point_schedule) {
        aligned_free(point_schedule);
    }

    if (skew_table) {
        aligned_free(skew_table);
    }

    if (point_pairs_1) {
        aligned_free(point_pairs_1);
    }

    if (point_pairs_2) {
        aligned_free(point_pairs_2);
    }

    if (scratch_space) {
        aligned_free(scratch_space);
    }

    if (bit_counts) {
        aligned_free(bit_counts);
    }

    if (bucket_counts) {
        aligned_free(bucket_counts);
    }

    if (bucket_empty_status) {
        aligned_free(bucket_empty_status);
    }

    if (round_counts) {
        aligned_free(round_counts);
    }

    point_schedule = other.point_schedule;
    skew_table = other.skew_table;
    point_pairs_1 = other.point_pairs_1;
    point_pairs_2 = other.point_pairs_2;
    scratch_space = other.scratch_space;
    bit_counts = other.bit_counts;
    bucket_counts = other.bucket_counts;
    bucket_empty_status = other.bucket_empty_status;
    round_counts = other.round_counts;

    other.point_schedule = nullptr;
    other.skew_table = nullptr;
    other.point_pairs_1 = nullptr;
    other.point_pairs_2 = nullptr;
    other.scratch_space = nullptr;
    other.bit_counts = nullptr;
    other.bucket_counts = nullptr;
    other.bucket_empty_status = nullptr;
    other.round_counts = nullptr;

    num_points = other.num_points;
    return *this;
}

affine_product_runtime_state pippenger_runtime_state::get_affine_product_runtime_state(const size_t num_threads,
                                                                                       const size_t thread_index)
{
    // if (!point_pairs_1) {
    //     init();
    // }
    const size_t points_per_thread = static_cast<size_t>(num_points / num_threads);
    const size_t num_buckets = static_cast<size_t>(
        1U << barretenberg::scalar_multiplication::get_optimal_bucket_width(static_cast<size_t>(num_points) / 2));

    scalar_multiplication::affine_product_runtime_state product_state;

    product_state.point_pairs_1 = point_pairs_1 + (thread_index * points_per_thread) + (thread_index * 16);
    product_state.point_pairs_2 = point_pairs_2 + (thread_index * points_per_thread) + (thread_index * 16);
    product_state.scratch_space = scratch_space + (thread_index * (points_per_thread / 2));
    product_state.bucket_counts = bucket_counts + (thread_index * (num_buckets));
    product_state.bit_offsets = bit_counts + (thread_index * (num_buckets));
    product_state.bucket_empty_status = bucket_empty_status + (thread_index * (num_buckets));
    return product_state;
}

pippenger_runtime_state::~pippenger_runtime_state()
{
    if (point_schedule) {
        aligned_free(point_schedule);
    }

    if (skew_table) {
        aligned_free(skew_table);
    }

    if (point_pairs_1) {
        aligned_free(point_pairs_1);
    }

    if (point_pairs_2) {
        aligned_free(point_pairs_2);
    }

    if (scratch_space) {
        aligned_free(scratch_space);
    }

    if (bit_counts) {
        aligned_free(bit_counts);
    }

    if (bucket_counts) {
        aligned_free(bucket_counts);
    }

    if (bucket_empty_status) {
        aligned_free(bucket_empty_status);
    }

    if (round_counts) {
        aligned_free(round_counts);
    }
}
} // namespace scalar_multiplication
} // namespace barretenberg