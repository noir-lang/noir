#include "runtime_states.hpp"

#include "barretenberg/common/mem.hpp"
#include "barretenberg/common/slab_allocator.hpp"
#include "barretenberg/common/thread.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"

// NOLINTBEGIN(cppcoreguidelines-pro-type-reinterpret-cast)
namespace barretenberg::scalar_multiplication {

size_t get_num_pippenger_rounds(const size_t num_points)
{
    const auto num_points_floor = static_cast<size_t>(1ULL << (numeric::get_msb(num_points)));
    const auto num_rounds =
        static_cast<size_t>(barretenberg::scalar_multiplication::get_num_rounds(static_cast<size_t>(num_points_floor)));
    return num_rounds;
}
template <typename Curve>
pippenger_runtime_state<Curve>::pippenger_runtime_state(const size_t num_initial_points) noexcept
    : num_points(num_initial_points * 2)
    , num_buckets(static_cast<size_t>(1ULL << barretenberg::scalar_multiplication::get_optimal_bucket_width(
                                          static_cast<size_t>(num_initial_points))))
    , num_rounds(get_num_pippenger_rounds(static_cast<size_t>(num_points)))
    , num_threads(get_num_cpus_pow2())
    , prefetch_overflow(num_threads * 16)
    , point_schedule_ptr(
          get_mem_slab((static_cast<size_t>(num_points) * num_rounds + prefetch_overflow) * sizeof(uint64_t)))
    , point_pairs_1_ptr(
          get_mem_slab((static_cast<size_t>(num_points) * 2 + (num_threads * 16)) * sizeof(AffineElement)))
    , point_pairs_2_ptr(
          get_mem_slab((static_cast<size_t>(num_points) * 2 + (num_threads * 16)) * sizeof(AffineElement)))
    , scratch_space_ptr(get_mem_slab(static_cast<size_t>(num_points) * sizeof(AffineElement)))
    , point_schedule(reinterpret_cast<uint64_t*>(point_schedule_ptr.get()))
    , point_pairs_1(reinterpret_cast<AffineElement*>(point_pairs_1_ptr.get()))
    , point_pairs_2(reinterpret_cast<AffineElement*>(point_pairs_2_ptr.get()))
    , scratch_space(reinterpret_cast<Fq*>(scratch_space_ptr.get()))
    , skew_table(reinterpret_cast<bool*>(aligned_alloc(64, pad(static_cast<size_t>(num_points) * sizeof(bool), 64))))
    , bucket_counts(reinterpret_cast<uint32_t*>(aligned_alloc(64, num_threads * num_buckets * sizeof(uint32_t))))
    , bit_counts(reinterpret_cast<uint32_t*>(aligned_alloc(64, num_threads * num_buckets * sizeof(uint32_t))))
    , bucket_empty_status(reinterpret_cast<bool*>(aligned_alloc(64, num_threads * num_buckets * sizeof(bool))))
    , round_counts(reinterpret_cast<uint64_t*>(aligned_alloc(32, MAX_NUM_ROUNDS * sizeof(uint64_t))))
{
    using Fq = typename Curve::BaseField;
    using AffineElement = typename Curve::AffineElement;

    const auto num_points_floor = static_cast<size_t>(1ULL << (numeric::get_msb(num_points)));
    const auto num_buckets = static_cast<size_t>(
        1ULL << barretenberg::scalar_multiplication::get_optimal_bucket_width(static_cast<size_t>(num_initial_points)));
    const auto num_rounds =
        static_cast<size_t>(barretenberg::scalar_multiplication::get_num_rounds(static_cast<size_t>(num_points_floor)));

    const size_t points_per_thread = static_cast<size_t>(num_points) / num_threads;
    parallel_for(num_threads, [&](size_t i) {
        const size_t thread_offset = i * points_per_thread;
        memset(reinterpret_cast<void*>(point_pairs_1 + thread_offset + (i * 16)),
               0,
               (points_per_thread + 16) * sizeof(AffineElement));
        memset(reinterpret_cast<void*>(point_pairs_2 + thread_offset + (i * 16)),
               0,
               (points_per_thread + 16) * sizeof(AffineElement));
        memset(reinterpret_cast<void*>(scratch_space + thread_offset), 0, (points_per_thread) * sizeof(Fq));
        for (size_t j = 0; j < num_rounds; ++j) {
            const size_t round_offset = (j * static_cast<size_t>(num_points));
            memset(reinterpret_cast<void*>(point_schedule + round_offset + thread_offset),
                   0,
                   points_per_thread * sizeof(uint64_t));
        }
        memset(reinterpret_cast<void*>(skew_table + thread_offset), 0, points_per_thread * sizeof(bool));
    });

    memset(reinterpret_cast<void*>(bucket_counts), 0, num_threads * num_buckets * sizeof(uint32_t));
    memset(reinterpret_cast<void*>(bit_counts), 0, num_threads * num_buckets * sizeof(uint32_t));
    memset(reinterpret_cast<void*>(bucket_empty_status), 0, num_threads * num_buckets * sizeof(bool));
    memset(reinterpret_cast<void*>(round_counts), 0, MAX_NUM_ROUNDS * sizeof(uint64_t));
}

template <typename Curve>
pippenger_runtime_state<Curve>::pippenger_runtime_state(pippenger_runtime_state&& other) noexcept
    : num_points(other.num_points)
    , num_buckets(other.num_buckets)
    , num_rounds(other.num_rounds)
    , num_threads(other.num_threads)
    , prefetch_overflow(other.prefetch_overflow)
    , point_schedule_ptr(std::move(other.point_schedule_ptr))
    , point_pairs_1_ptr(std::move(other.point_pairs_1_ptr))
    , point_pairs_2_ptr(std::move(other.point_pairs_2_ptr))
    , scratch_space_ptr(std::move(other.scratch_space_ptr))
    , point_schedule(other.point_schedule)
    , point_pairs_1(other.point_pairs_1)
    , point_pairs_2(other.point_pairs_2)
    , scratch_space(other.scratch_space)
    , skew_table(other.skew_table)
    , bucket_counts(other.bucket_counts)
    , bit_counts(other.bit_counts)
    , bucket_empty_status(other.bucket_empty_status)
    , round_counts(other.round_counts)

{
    other.point_schedule = nullptr;
    other.skew_table = nullptr;
    other.point_pairs_1 = nullptr;
    other.point_pairs_2 = nullptr;
    other.scratch_space = nullptr;
    other.bit_counts = nullptr;
    other.bucket_counts = nullptr;
    other.bucket_empty_status = nullptr;
    other.round_counts = nullptr;
}

template <typename Curve>
pippenger_runtime_state<Curve>& pippenger_runtime_state<Curve>::operator=(
    pippenger_runtime_state<Curve>&& other) noexcept
{
    if (skew_table != nullptr) {
        aligned_free(skew_table);
    }

    if (bit_counts != nullptr) {
        aligned_free(bit_counts);
    }

    if (bucket_counts != nullptr) {
        aligned_free(bucket_counts);
    }

    if (bucket_empty_status != nullptr) {
        aligned_free(bucket_empty_status);
    }

    if (round_counts != nullptr) {
        aligned_free(round_counts);
    }

    point_schedule_ptr = std::move(other.point_schedule_ptr);
    point_pairs_1_ptr = std::move(other.point_pairs_1_ptr);
    point_pairs_2_ptr = std::move(other.point_pairs_2_ptr);
    scratch_space_ptr = std::move(other.scratch_space_ptr);

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

template <typename Curve>
affine_product_runtime_state<Curve> pippenger_runtime_state<Curve>::get_affine_product_runtime_state(
    const size_t num_threads, const size_t thread_index)
{
    const auto points_per_thread = static_cast<size_t>(num_points / num_threads);
    const auto num_buckets =
        static_cast<size_t>(1U << scalar_multiplication::get_optimal_bucket_width(static_cast<size_t>(num_points) / 2));

    scalar_multiplication::affine_product_runtime_state<Curve> product_state;

    product_state.point_pairs_1 = point_pairs_1 + (thread_index * points_per_thread) + (thread_index * 16);
    product_state.point_pairs_2 = point_pairs_2 + (thread_index * points_per_thread) + (thread_index * 16);
    product_state.scratch_space = scratch_space + (thread_index * (points_per_thread / 2));
    product_state.bucket_counts = bucket_counts + (thread_index * (num_buckets));
    product_state.bit_offsets = bit_counts + (thread_index * (num_buckets));
    product_state.bucket_empty_status = bucket_empty_status + (thread_index * (num_buckets));
    return product_state;
}

template <typename Curve> pippenger_runtime_state<Curve>::~pippenger_runtime_state() noexcept
{
    if (skew_table != nullptr) {
        aligned_free(skew_table);
    }

    if (bit_counts != nullptr) {
        aligned_free(bit_counts);
    }

    if (bucket_counts != nullptr) {
        aligned_free(bucket_counts);
    }

    if (bucket_empty_status != nullptr) {
        aligned_free(bucket_empty_status);
    }

    if (round_counts != nullptr) {
        aligned_free(round_counts);
    }
}

template struct affine_product_runtime_state<curve::BN254>;
template struct affine_product_runtime_state<curve::Grumpkin>;
template struct pippenger_runtime_state<curve::BN254>;
template struct pippenger_runtime_state<curve::Grumpkin>;
} // namespace barretenberg::scalar_multiplication

// NOLINTEND(cppcoreguidelines-pro-type-reinterpret-cast)
