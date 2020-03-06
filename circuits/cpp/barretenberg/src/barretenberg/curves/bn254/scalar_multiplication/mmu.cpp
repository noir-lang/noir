#include "./mmu.hpp"

namespace barretenberg {
namespace mmu {
namespace {
static bool* skew_memory = nullptr;
static uint64_t* wnaf_memory = nullptr;
static g1::element* bucket_memory = nullptr;
static g1::affine_element* point_pairs_1 = nullptr;
static g1::affine_element* point_pairs_2 = nullptr;
static fq* scratch_space = nullptr;
static uint32_t* bucket_count_memory = nullptr;
static uint32_t* bit_count_memory = nullptr;
static bool* bucket_empty_status = nullptr;

const auto init = []() {
    static_assert(PIPPENGER_BLOCK_SIZE < 27);
    constexpr size_t max_num_points = (1 << PIPPENGER_BLOCK_SIZE);
    constexpr size_t max_num_rounds = 8;
    constexpr size_t max_buckets =
        1 << barretenberg::scalar_multiplication::get_optimal_bucket_width(1 << PIPPENGER_BLOCK_SIZE);
    constexpr size_t thread_overspill = 1024;

    // size_t memory = max_num_points * max_num_rounds * 2 * sizeof(uint64_t);
    // memory += (max_buckets + thread_overspill) * sizeof(g1::element);
    // memory += (max_num_points * 2) * sizeof(g1::affine_element);
    // memory += (max_num_points * 2) * sizeof(g1::affine_element);
    // memory += (max_num_points) * sizeof(fq);
    // memory += max_num_points * 2 * sizeof(uint32_t);
    // memory += max_num_points * 2 * sizeof(uint32_t);
    // memory += max_num_points * 2 * sizeof(bool);
    // memory += max_num_points * 2 * sizeof(bool);
    // printf("total memory allocated in mmu = %lu mb \n", memory / (1024UL * 1024UL));

    wnaf_memory = (uint64_t*)(aligned_alloc(64, (max_num_points * max_num_rounds * 2 + 256) * sizeof(uint64_t)));
    bucket_memory = (g1::element*)(aligned_alloc(64, (max_buckets + thread_overspill) * sizeof(g1::element)));

    // TODO: we're allocating too much memory here, trim this down
    point_pairs_1 = (g1::affine_element*)(aligned_alloc(64, (max_num_points * 2 + 256) * sizeof(g1::affine_element)));
    point_pairs_2 = (g1::affine_element*)(aligned_alloc(64, (max_num_points * 2 + 256) * sizeof(g1::affine_element)));
    scratch_space = (fq*)(aligned_alloc(64, (max_num_points) * sizeof(fq)));

    bucket_count_memory = (uint32_t*)(aligned_alloc(64, max_num_points * 2 * sizeof(uint32_t)));
    bit_count_memory = (uint32_t*)(aligned_alloc(64, max_num_points * 2 * sizeof(uint32_t)));
    bucket_empty_status = (bool*)(aligned_alloc(64, max_num_points * 2 * sizeof(bool)));

    skew_memory = (bool*)(aligned_alloc(64, max_num_points * 2 * sizeof(bool)));
    memset((void*)skew_memory, 0, max_num_points * 2 * sizeof(bool));
    memset((void*)wnaf_memory, 1, max_num_points * max_num_rounds * 2 * sizeof(uint64_t));
    memset((void*)bucket_memory, 0xff, (max_buckets + thread_overspill) * sizeof(g1::element));
    memset((void*)point_pairs_1, 0xff, (max_num_points * 2) * sizeof(g1::affine_element));
    memset((void*)point_pairs_2, 0xff, (max_num_points * 2) * sizeof(g1::affine_element));
    memset((void*)scratch_space, 0xff, (max_num_points) * sizeof(fq));

    memset((void*)bucket_count_memory, 0x00, max_num_points * 2 * sizeof(uint32_t));
    memset((void*)bit_count_memory, 0x00, max_num_points * 2 * sizeof(uint32_t));
    memset((void*)bucket_empty_status, 0x00, max_num_points * 2 * sizeof(bool));

    return 1;
}();
} // namespace

bool* get_skew_pointer()
{
    return skew_memory;
}

uint64_t* get_wnaf_pointer()
{
    return wnaf_memory;
}

g1::element* get_bucket_pointer()
{
    return bucket_memory;
}

scalar_multiplication::affine_product_runtime_state get_affine_product_runtime_state(const size_t num_threads,
                                                                                     const size_t thread_index)
{
    constexpr size_t max_num_points = (2 << PIPPENGER_BLOCK_SIZE);
    const size_t points_per_thread = max_num_points / num_threads;

    scalar_multiplication::affine_product_runtime_state product_state;

    product_state.point_pairs_1 = point_pairs_1 + (thread_index * points_per_thread);
    product_state.point_pairs_2 = point_pairs_2 + (thread_index * points_per_thread);
    product_state.scratch_space = scratch_space + (thread_index * (points_per_thread / 2));
    product_state.bucket_counts = bucket_count_memory + (thread_index * (points_per_thread));
    product_state.bit_offsets = bit_count_memory + (thread_index * (points_per_thread));
    product_state.bucket_empty_status = bucket_empty_status + (thread_index * (points_per_thread));
    return product_state;
}
} // namespace mmu
} // namespace barretenberg