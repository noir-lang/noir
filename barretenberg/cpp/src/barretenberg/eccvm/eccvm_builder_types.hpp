#pragma once

#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"

namespace bb::eccvm {
static constexpr size_t NUM_SCALAR_BITS = 128;   // The length of scalars handled by the ECCVVM
static constexpr size_t NUM_WNAF_DIGIT_BITS = 4; // Scalars are decompose into base 16 in wNAF form
static constexpr size_t NUM_WNAF_DIGITS_PER_SCALAR = NUM_SCALAR_BITS / NUM_WNAF_DIGIT_BITS; // 32
static constexpr uint64_t WNAF_MASK = static_cast<uint64_t>((1ULL << NUM_WNAF_DIGIT_BITS) - 1ULL);
static constexpr size_t POINT_TABLE_SIZE = 1ULL << (NUM_WNAF_DIGIT_BITS);
static constexpr size_t WNAF_DIGITS_PER_ROW = 4;
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
    bool operator==(const VMOperation<CycleGroup>& other) const = default;
};
template <typename CycleGroup> struct ScalarMul {
    uint32_t pc;
    uint256_t scalar;
    typename CycleGroup::affine_element base_point;
    std::array<int, NUM_WNAF_DIGITS_PER_SCALAR> wnaf_digits;
    bool wnaf_skew;
    // size bumped by 1 to record base_point.dbl()
    std::array<typename CycleGroup::affine_element, POINT_TABLE_SIZE + 1> precomputed_table;
};

template <typename CycleGroup> using MSM = std::vector<ScalarMul<CycleGroup>>;

} // namespace bb::eccvm