#pragma once

#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/ecc/groups/wnaf.hpp"

namespace bb::scalar_multiplication {
// simple helper functions to retrieve pointers to pre-allocated memory for the scalar multiplication algorithm.
// This is to eliminate page faults when allocating (and writing) to large tranches of memory.
constexpr size_t get_optimal_bucket_width(const size_t num_points)
{
    if (num_points >= 14617149) {
        return 21;
    }
    if (num_points >= 1139094) {
        return 18;
    }
    // if (num_points >= 100000)
    if (num_points >= 155975) {
        return 15;
    }
    if (num_points >= 144834)
    // if (num_points >= 100000)
    {
        return 14;
    }
    if (num_points >= 25067) {
        return 12;
    }
    if (num_points >= 13926) {
        return 11;
    }
    if (num_points >= 7659) {
        return 10;
    }
    if (num_points >= 2436) {
        return 9;
    }
    if (num_points >= 376) {
        return 7;
    }
    if (num_points >= 231) {
        return 6;
    }
    if (num_points >= 97) {
        return 5;
    }
    if (num_points >= 35) {
        return 4;
    }
    if (num_points >= 10) {
        return 3;
    }
    if (num_points >= 2) {
        return 2;
    }
    return 1;
}

constexpr size_t get_num_rounds(const size_t num_points)
{
    const size_t bits_per_bucket = get_optimal_bucket_width(num_points / 2);
    return WNAF_SIZE(bits_per_bucket + 1);
}

template <typename Curve> struct affine_product_runtime_state {
    typename Curve::AffineElement* points;
    typename Curve::AffineElement* point_pairs_1;
    typename Curve::AffineElement* point_pairs_2;
    typename Curve::BaseField* scratch_space;
    uint32_t* bucket_counts;
    uint32_t* bit_offsets;
    uint64_t* point_schedule;
    uint32_t num_points;
    uint32_t num_buckets;
    bool* bucket_empty_status;
};

template <typename Curve> struct pippenger_runtime_state {
    using Fq = typename Curve::BaseField;
    using AffineElement = typename Curve::AffineElement;

    static constexpr size_t MAX_NUM_ROUNDS = 256;
    uint64_t num_points;
    size_t num_buckets;
    size_t num_rounds;
    size_t num_threads;
    size_t prefetch_overflow;
    std::shared_ptr<void> point_schedule_ptr;
    std::shared_ptr<void> point_pairs_1_ptr;
    std::shared_ptr<void> point_pairs_2_ptr;
    std::shared_ptr<void> scratch_space_ptr;
    uint64_t* point_schedule;
    typename Curve::AffineElement* point_pairs_1;
    typename Curve::AffineElement* point_pairs_2;
    typename Curve::BaseField* scratch_space;

    bool* skew_table;
    uint32_t* bucket_counts;
    uint32_t* bit_counts;
    bool* bucket_empty_status;
    uint64_t* round_counts;

    pippenger_runtime_state(size_t num_initial_points) noexcept;
    pippenger_runtime_state(pippenger_runtime_state&& other) noexcept;
    pippenger_runtime_state& operator=(pippenger_runtime_state&& other) noexcept;
    ~pippenger_runtime_state() noexcept;

    // explicitly delete copy constructor and copy assignment operator.
    // This is an expensive, large data structure. No copy! Bad!
    pippenger_runtime_state& operator=(pippenger_runtime_state& other) = delete;
    pippenger_runtime_state(pippenger_runtime_state& other) = delete;

    affine_product_runtime_state<Curve> get_affine_product_runtime_state(size_t num_threads, size_t thread_index);
};

} // namespace bb::scalar_multiplication
