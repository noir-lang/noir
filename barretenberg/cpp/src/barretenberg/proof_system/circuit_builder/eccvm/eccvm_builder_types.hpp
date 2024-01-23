#pragma once

#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"

namespace bb::eccvm {

static constexpr size_t NUM_SCALAR_BITS = 128;
static constexpr size_t WNAF_SLICE_BITS = 4;
static constexpr size_t NUM_WNAF_SLICES = (NUM_SCALAR_BITS + WNAF_SLICE_BITS - 1) / WNAF_SLICE_BITS;
static constexpr uint64_t WNAF_MASK = static_cast<uint64_t>((1ULL << WNAF_SLICE_BITS) - 1ULL);
static constexpr size_t POINT_TABLE_SIZE = 1ULL << (WNAF_SLICE_BITS);
static constexpr size_t WNAF_SLICES_PER_ROW = 4;
static constexpr size_t ADDITIONS_PER_ROW = 4;

template <typename CycleGroup> struct VMOperation {
    bool add = false;
    bool mul = false;
    bool eq = false;
    bool reset = false;
    typename CycleGroup::affine_element base_point = typename CycleGroup::affine_element{ 0, 0 };
    uint256_t z1 = 0;
    uint256_t z2 = 0;
    typename CycleGroup::subgroup_field mul_scalar_full = 0;
    [[nodiscard]] uint32_t get_opcode_value() const
    {
        auto res = static_cast<uint32_t>(add);
        res += res;
        res += static_cast<uint32_t>(mul);
        res += res;
        res += static_cast<uint32_t>(eq);
        res += res;
        res += static_cast<uint32_t>(reset);
        return res;
    }
};
template <typename CycleGroup> struct ScalarMul {
    uint32_t pc;
    uint256_t scalar;
    typename CycleGroup::affine_element base_point;
    std::array<int, NUM_WNAF_SLICES> wnaf_slices;
    bool wnaf_skew;
    std::array<typename CycleGroup::affine_element, POINT_TABLE_SIZE> precomputed_table;
};

template <typename CycleGroup> using MSM = std::vector<ScalarMul<CycleGroup>>;

} // namespace bb::eccvm