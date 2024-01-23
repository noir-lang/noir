#pragma once

#include "./eccvm_builder_types.hpp"

namespace bb {

template <typename Flavor> class ECCVMPrecomputedTablesBuilder {
  public:
    using CycleGroup = typename Flavor::CycleGroup;
    using FF = typename Flavor::FF;
    using Element = typename CycleGroup::element;
    using AffineElement = typename CycleGroup::affine_element;

    static constexpr size_t NUM_WNAF_SLICES = bb::eccvm::NUM_WNAF_SLICES;
    static constexpr size_t WNAF_SLICES_PER_ROW = bb::eccvm::WNAF_SLICES_PER_ROW;
    static constexpr size_t WNAF_SLICE_BITS = bb::eccvm::WNAF_SLICE_BITS;

    struct PrecomputeState {
        int s1 = 0;
        int s2 = 0;
        int s3 = 0;
        int s4 = 0;
        int s5 = 0;
        int s6 = 0;
        int s7 = 0;
        int s8 = 0;
        bool skew = false;
        bool point_transition = false;
        uint32_t pc = 0;
        uint32_t round = 0;
        uint256_t scalar_sum = 0;
        AffineElement precompute_accumulator{ 0, 0 };
        AffineElement precompute_double{ 0, 0 };
    };

    static std::vector<PrecomputeState> compute_precompute_state(
        const std::vector<bb::eccvm::ScalarMul<CycleGroup>>& ecc_muls)
    {
        std::vector<PrecomputeState> precompute_state;

        // start with empty row (shiftable polynomials must have 0 as first coefficient)
        precompute_state.push_back(PrecomputeState{});
        static constexpr size_t num_rows_per_scalar = NUM_WNAF_SLICES / WNAF_SLICES_PER_ROW;

        // current impl doesn't work if not 4
        static_assert(WNAF_SLICES_PER_ROW == 4);

        for (const auto& entry : ecc_muls) {
            const auto& slices = entry.wnaf_slices;
            uint256_t scalar_sum = 0;

            const Element point = entry.base_point;
            const Element d2 = point.dbl();

            for (size_t i = 0; i < num_rows_per_scalar; ++i) {
                PrecomputeState row;
                const int slice0 = slices[i * WNAF_SLICES_PER_ROW];
                const int slice1 = slices[i * WNAF_SLICES_PER_ROW + 1];
                const int slice2 = slices[i * WNAF_SLICES_PER_ROW + 2];
                const int slice3 = slices[i * WNAF_SLICES_PER_ROW + 3];

                const int slice0base2 = (slice0 + 15) / 2;
                const int slice1base2 = (slice1 + 15) / 2;
                const int slice2base2 = (slice2 + 15) / 2;
                const int slice3base2 = (slice3 + 15) / 2;

                // convert into 2-bit chunks
                row.s1 = slice0base2 >> 2;
                row.s2 = slice0base2 & 3;
                row.s3 = slice1base2 >> 2;
                row.s4 = slice1base2 & 3;
                row.s5 = slice2base2 >> 2;
                row.s6 = slice2base2 & 3;
                row.s7 = slice3base2 >> 2;
                row.s8 = slice3base2 & 3;
                bool last_row = (i == num_rows_per_scalar - 1);

                row.skew = last_row ? entry.wnaf_skew : false;

                row.scalar_sum = scalar_sum;

                // N.B. we apply a constraint that requires slice1 to be positive for the 1st row of each scalar sum.
                //      This ensures we do not have WNAF representations of negative values
                const int row_chunk = slice3 + slice2 * (1 << 4) + slice1 * (1 << 8) + slice0 * (1 << 12);

                bool chunk_negative = row_chunk < 0;

                scalar_sum = scalar_sum << (WNAF_SLICE_BITS * WNAF_SLICES_PER_ROW);
                if (chunk_negative) {
                    scalar_sum -= static_cast<uint64_t>(-row_chunk);
                } else {
                    scalar_sum += static_cast<uint64_t>(row_chunk);
                }
                row.round = static_cast<uint32_t>(i);
                row.point_transition = last_row;
                row.pc = entry.pc;

                if (last_row) {
                    ASSERT(scalar_sum - entry.wnaf_skew == entry.scalar);
                }

                row.precompute_double = d2;
                // fill accumulator in reverse order i.e. first row = 15[P], then 13[P], ..., 1[P]
                row.precompute_accumulator = entry.precomputed_table[bb::eccvm::POINT_TABLE_SIZE - 1 - i];
                precompute_state.emplace_back(row);
            }
        }
        return precompute_state;
    }
};
} // namespace bb