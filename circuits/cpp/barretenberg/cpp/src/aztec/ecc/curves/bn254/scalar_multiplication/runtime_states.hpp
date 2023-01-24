#pragma once

#include "../g1.hpp"

namespace barretenberg {
// simple helper functions to retrieve pointers to pre-allocated memory for the scalar multiplication algorithm.
// This is to eliminate page faults when allocating (and writing) to large tranches of memory.
namespace scalar_multiplication {
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

struct affine_product_runtime_state {
    g1::affine_element* points;
    g1::affine_element* point_pairs_1;
    g1::affine_element* point_pairs_2;
    fq* scratch_space;
    uint32_t* bucket_counts;
    uint32_t* bit_offsets;
    uint64_t* point_schedule;
    uint32_t num_points;
    uint32_t num_buckets;
    bool* bucket_empty_status;
};

struct pippenger_runtime_state {
    uint64_t* point_schedule;
    bool* skew_table;
    g1::affine_element* point_pairs_1;
    g1::affine_element* point_pairs_2;
    fq* scratch_space;
    uint32_t* bucket_counts;
    uint32_t* bit_counts;
    bool* bucket_empty_status;
    uint64_t* round_counts;
    uint64_t num_points;

    pippenger_runtime_state(const size_t num_initial_points);
    pippenger_runtime_state(pippenger_runtime_state&& other);
    pippenger_runtime_state& operator=(pippenger_runtime_state&& other);
    ~pippenger_runtime_state();

    affine_product_runtime_state get_affine_product_runtime_state(const size_t num_threads, const size_t thread_index);
};
} // namespace scalar_multiplication
} // namespace barretenberg