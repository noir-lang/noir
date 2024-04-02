#pragma once

#include "./eccvm_builder_types.hpp"
#include "./msm_builder.hpp"
#include "./precomputed_tables_builder.hpp"
#include "./transcript_builder.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/honk/proof_system/logderivative_library.hpp"
#include "barretenberg/honk/proof_system/permutation_library.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/stdlib_circuit_builders/op_queue/ecc_op_queue.hpp"

namespace bb {

class ECCVMCircuitBuilder {
  public:
    using CycleGroup = bb::g1;
    using FF = grumpkin::fr;
    using Polynomial = bb::Polynomial<FF>;

    using CycleScalar = typename CycleGroup::subgroup_field;
    using Element = typename CycleGroup::element;
    using AffineElement = typename CycleGroup::affine_element;

    static constexpr size_t NUM_SCALAR_BITS = bb::eccvm::NUM_SCALAR_BITS;
    static constexpr size_t WNAF_SLICE_BITS = bb::eccvm::WNAF_SLICE_BITS;
    static constexpr size_t NUM_WNAF_SLICES = bb::eccvm::NUM_WNAF_SLICES;
    static constexpr uint64_t WNAF_MASK = bb::eccvm::WNAF_MASK;
    static constexpr size_t POINT_TABLE_SIZE = bb::eccvm::POINT_TABLE_SIZE;
    static constexpr size_t WNAF_SLICES_PER_ROW = bb::eccvm::WNAF_SLICES_PER_ROW;
    static constexpr size_t ADDITIONS_PER_ROW = bb::eccvm::ADDITIONS_PER_ROW;

    using MSM = bb::eccvm::MSM<CycleGroup>;
    using VMOperation = bb::eccvm::VMOperation<CycleGroup>;
    std::shared_ptr<ECCOpQueue> op_queue;
    using ScalarMul = bb::eccvm::ScalarMul<CycleGroup>;

    ECCVMCircuitBuilder(std::shared_ptr<ECCOpQueue>& op_queue)
        : op_queue(op_queue){};

    [[nodiscard]] uint32_t get_number_of_muls() const
    {
        return op_queue->cached_num_muls + op_queue->cached_active_msm_count;
    }

    std::vector<MSM> get_msms() const
    {
        const uint32_t num_muls = get_number_of_muls();
        /**
         * For input point [P], return { -15[P], -13[P], ..., -[P], [P], ..., 13[P], 15[P] }
         */
        const auto compute_precomputed_table = [](const AffineElement& base_point) {
            const auto d2 = Element(base_point).dbl();
            std::array<Element, POINT_TABLE_SIZE + 1> table;
            table[POINT_TABLE_SIZE] = d2; // need this for later
            table[POINT_TABLE_SIZE / 2] = base_point;
            for (size_t i = 1; i < POINT_TABLE_SIZE / 2; ++i) {
                table[i + POINT_TABLE_SIZE / 2] = Element(table[i + POINT_TABLE_SIZE / 2 - 1]) + d2;
            }
            for (size_t i = 0; i < POINT_TABLE_SIZE / 2; ++i) {
                table[i] = -table[POINT_TABLE_SIZE - 1 - i];
            }

            Element::batch_normalize(&table[0], POINT_TABLE_SIZE + 1);
            std::array<AffineElement, POINT_TABLE_SIZE + 1> result;
            for (size_t i = 0; i < POINT_TABLE_SIZE + 1; ++i) {
                result[i] = AffineElement(table[i].x, table[i].y);
            }
            return result;
        };
        const auto compute_wnaf_slices = [](uint256_t scalar) {
            std::array<int, NUM_WNAF_SLICES> output;
            int previous_slice = 0;
            for (size_t i = 0; i < NUM_WNAF_SLICES; ++i) {
                // slice the scalar into 4-bit chunks, starting with the least significant bits
                uint64_t raw_slice = static_cast<uint64_t>(scalar) & WNAF_MASK;

                bool is_even = ((raw_slice & 1ULL) == 0ULL);

                int wnaf_slice = static_cast<int>(raw_slice);

                if (i == 0 && is_even) {
                    // if least significant slice is even, we add 1 to create an odd value && set 'skew' to true
                    wnaf_slice += 1;
                } else if (is_even) {
                    // for other slices, if it's even, we add 1 to the slice value
                    // and subtract 16 from the previous slice to preserve the total scalar sum
                    static constexpr int borrow_constant = static_cast<int>(1ULL << WNAF_SLICE_BITS);
                    previous_slice -= borrow_constant;
                    wnaf_slice += 1;
                }

                if (i > 0) {
                    const size_t idx = i - 1;
                    output[NUM_WNAF_SLICES - idx - 1] = previous_slice;
                }
                previous_slice = wnaf_slice;

                // downshift raw_slice by 4 bits
                scalar = scalar >> WNAF_SLICE_BITS;
            }

            ASSERT(scalar == 0);

            output[0] = previous_slice;

            return output;
        };

        // a vector of MSMs = a vector of a vector of scalar muls
        // each mul
        size_t msm_count = 0;
        size_t active_mul_count = 0;
        std::vector<size_t> msm_opqueue_index;
        std::vector<std::pair<size_t, size_t>> msm_mul_index;
        std::vector<size_t> msm_sizes;

        // std::vector<std::vector<size_t>> msm_indices;
        // std::vector<size_t> active_msm_indices;
        for (size_t i = 0; i < op_queue->raw_ops.size(); ++i) {
            const auto& op = op_queue->raw_ops[i];
            if (op.mul) {
                if (op.z1 != 0 || op.z2 != 0) {
                    msm_opqueue_index.push_back(i);
                    msm_mul_index.emplace_back(msm_count, active_mul_count);
                }
                if (op.z1 != 0) {
                    active_mul_count++;
                }
                if (op.z2 != 0) {
                    active_mul_count++;
                }
            } else if (active_mul_count > 0) {
                msm_sizes.push_back(active_mul_count);
                msm_count++;
                active_mul_count = 0;
            }
        }
        // if last op is a mul we have not correctly computed the total number of msms
        if (op_queue->raw_ops.back().mul) {
            msm_sizes.push_back(active_mul_count);
            msm_count++;
        }
        std::vector<MSM> msms_test(msm_count);
        for (size_t i = 0; i < msm_count; ++i) {
            auto& msm = msms_test[i];
            msm.resize(msm_sizes[i]);
        }

        run_loop_in_parallel(msm_opqueue_index.size(), [&](size_t start, size_t end) {
            for (size_t i = start; i < end; i++) {
                //  for (size_t i = 0; i < msm_opqueue_index.size(); ++i) {
                const size_t opqueue_index = msm_opqueue_index[i];
                const auto& op = op_queue->raw_ops[opqueue_index];
                auto [msm_index, mul_index] = msm_mul_index[i];
                if (op.z1 != 0) {
                    ASSERT(msms_test.size() > msm_index);
                    ASSERT(msms_test[msm_index].size() > mul_index);
                    msms_test[msm_index][mul_index] = (ScalarMul{
                        .pc = 0,
                        .scalar = op.z1,
                        .base_point = op.base_point,
                        .wnaf_slices = compute_wnaf_slices(op.z1),
                        .wnaf_skew = (op.z1 & 1) == 0,
                        .precomputed_table = compute_precomputed_table(op.base_point),
                    });
                    mul_index++;
                }
                if (op.z2 != 0) {
                    ASSERT(msms_test.size() > msm_index);
                    ASSERT(msms_test[msm_index].size() > mul_index);
                    auto endo_point = AffineElement{ op.base_point.x * FF::cube_root_of_unity(), -op.base_point.y };
                    msms_test[msm_index][mul_index] = (ScalarMul{
                        .pc = 0,
                        .scalar = op.z2,
                        .base_point = endo_point,
                        .wnaf_slices = compute_wnaf_slices(op.z2),
                        .wnaf_skew = (op.z2 & 1) == 0,
                        .precomputed_table = compute_precomputed_table(endo_point),
                    });
                }
            }
        });

        // update pc. easier to do this serially but in theory could be optimised out
        // We start pc at `num_muls` and decrement for each mul processed.
        // This gives us two desired properties:
        // 1: the value of pc at the 1st row = number of muls (easy to check)
        // 2: the value of pc for the final mul = 1
        // The latter point is valuable as it means that we can add empty rows (where pc = 0) and still satisfy our
        // sumcheck relations that involve pc (if we did the other way around, starting at 1 and ending at num_muls,
        // we create a discontinuity in pc values between the last transcript row and the following empty row)
        uint32_t pc = num_muls;
        for (auto& msm : msms_test) {
            for (auto& mul : msm) {
                mul.pc = pc;
                pc--;
            }
        }

        ASSERT(pc == 0);
        return msms_test;
    }

    static std::vector<ScalarMul> get_flattened_scalar_muls(const std::vector<MSM>& msms)
    {
        std::vector<ScalarMul> result;
        for (const auto& msm : msms) {
            for (const auto& mul : msm) {
                result.push_back(mul);
            }
        }
        return result;
    }

    [[nodiscard]] size_t get_num_gates() const
    {
        // (issue #2218)
        return op_queue->get_num_rows();
    }

    [[nodiscard]] size_t get_circuit_subgroup_size(const size_t num_rows) const
    {

        const auto num_rows_log2 = static_cast<size_t>(numeric::get_msb64(num_rows));
        size_t num_rows_pow2 = 1UL << (num_rows_log2 + (1UL << num_rows_log2 == num_rows ? 0 : 1));
        return num_rows_pow2;
    }
};
} // namespace bb
