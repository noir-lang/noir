#pragma once

#include "./eccvm_builder_types.hpp"

namespace bb {

template <typename Flavor> class ECCVMTranscriptBuilder {
  public:
    using CycleGroup = typename Flavor::CycleGroup;
    using FF = typename Flavor::FF;
    using Element = typename CycleGroup::element;
    using AffineElement = typename CycleGroup::affine_element;

    struct TranscriptState {
        bool accumulator_empty = false;
        bool q_add = false;
        bool q_mul = false;
        bool q_eq = false;
        bool q_reset_accumulator = false;
        bool msm_transition = false;
        uint32_t pc = 0;
        uint32_t msm_count = 0;
        FF base_x = 0;
        FF base_y = 0;
        uint256_t z1 = 0;
        uint256_t z2 = 0;
        bool z1_zero = false;
        bool z2_zero = false;
        uint32_t opcode = 0;
        FF accumulator_x = 0;
        FF accumulator_y = 0;
        FF msm_output_x = 0;
        FF msm_output_y = 0;
        FF collision_check = 0;
    };
    struct VMState {
        uint32_t pc = 0;
        uint32_t count = 0;
        AffineElement accumulator = CycleGroup::affine_point_at_infinity;
        AffineElement msm_accumulator = CycleGroup::affine_point_at_infinity;
        bool is_accumulator_empty = true;
    };
    struct Opcode {
        bool add;
        bool mul;
        bool eq;
        bool reset;
        [[nodiscard]] uint32_t value() const
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
    static std::vector<TranscriptState> compute_transcript_state(
        const std::vector<bb::eccvm::VMOperation<CycleGroup>>& vm_operations, const uint32_t total_number_of_muls)
    {
        std::vector<TranscriptState> transcript_state;
        VMState state{
            .pc = total_number_of_muls,
            .count = 0,
            .accumulator = CycleGroup::affine_point_at_infinity,
            .msm_accumulator = CycleGroup::affine_point_at_infinity,
            .is_accumulator_empty = true,
        };
        VMState updated_state;

        // add an empty row. 1st row all zeroes because of our shiftable polynomials
        transcript_state.emplace_back(TranscriptState{});
        for (size_t i = 0; i < vm_operations.size(); ++i) {
            TranscriptState row;
            const bb::eccvm::VMOperation<CycleGroup>& entry = vm_operations[i];

            const bool is_mul = entry.mul;
            const bool z1_zero = (entry.mul) ? entry.z1 == 0 : true;
            const bool z2_zero = (entry.mul) ? entry.z2 == 0 : true;
            const uint32_t num_muls = is_mul ? (static_cast<uint32_t>(!z1_zero) + static_cast<uint32_t>(!z2_zero)) : 0;

            updated_state = state;

            if (entry.reset) {
                updated_state.is_accumulator_empty = true;
                updated_state.msm_accumulator = CycleGroup::affine_point_at_infinity;
            }
            updated_state.pc = state.pc - num_muls;

            bool last_row = i == (vm_operations.size() - 1);
            // msm transition = current row is doing a lookup to validate output = msm output
            // i.e. next row is not part of MSM and current row is part of MSM
            //   or next row is irrelevent and current row is a straight MUL
            bool next_not_msm = last_row ? true : !vm_operations[i + 1].mul;

            bool msm_transition = entry.mul && next_not_msm;

            // we reset the count in updated state if we are not accumulating and not doing an msm
            bool current_msm = entry.mul;
            bool current_ongoing_msm = entry.mul && !next_not_msm;
            updated_state.count = current_ongoing_msm ? state.count + num_muls : 0;

            if (current_msm) {
                const auto P = typename CycleGroup::element(entry.base_point);
                const auto R = typename CycleGroup::element(state.msm_accumulator);
                updated_state.msm_accumulator = R + P * entry.mul_scalar_full;
            }

            if (entry.mul && next_not_msm) {
                if (state.is_accumulator_empty) {
                    updated_state.accumulator = updated_state.msm_accumulator;
                } else {
                    const auto R = typename CycleGroup::element(state.accumulator);
                    updated_state.accumulator = R + updated_state.msm_accumulator;
                }
                updated_state.is_accumulator_empty = false;
            }

            bool add_accumulate = entry.add;
            if (add_accumulate) {
                if (state.is_accumulator_empty) {

                    updated_state.accumulator = entry.base_point;
                } else {
                    updated_state.accumulator = typename CycleGroup::element(state.accumulator) + entry.base_point;
                }
                updated_state.is_accumulator_empty = false;
            }
            row.accumulator_empty = state.is_accumulator_empty;
            row.q_add = entry.add;
            row.q_mul = entry.mul;
            row.q_eq = entry.eq;
            row.q_reset_accumulator = entry.reset;
            row.msm_transition = msm_transition;
            row.pc = state.pc;
            row.msm_count = state.count;
            row.base_x = (entry.add || entry.mul || entry.eq) ? entry.base_point.x : 0;
            row.base_y = (entry.add || entry.mul || entry.eq) ? entry.base_point.y : 0;
            row.z1 = (entry.mul) ? entry.z1 : 0;
            row.z2 = (entry.mul) ? entry.z2 : 0;
            row.z1_zero = z1_zero;
            row.z2_zero = z2_zero;
            row.opcode = Opcode{ .add = entry.add, .mul = entry.mul, .eq = entry.eq, .reset = entry.reset }.value();
            row.accumulator_x = (state.accumulator.is_point_at_infinity()) ? 0 : state.accumulator.x;
            row.accumulator_y = (state.accumulator.is_point_at_infinity()) ? 0 : state.accumulator.y;
            row.msm_output_x =
                msm_transition
                    ? (updated_state.msm_accumulator.is_point_at_infinity() ? 0 : updated_state.msm_accumulator.x)
                    : 0;
            row.msm_output_y =
                msm_transition
                    ? (updated_state.msm_accumulator.is_point_at_infinity() ? 0 : updated_state.msm_accumulator.y)
                    : 0;

            if (entry.mul && next_not_msm && !row.accumulator_empty) {
                ASSERT((row.msm_output_x != row.accumulator_x) &&
                       "eccvm: attempting msm. Result point x-coordinate matches accumulator x-coordinate.");
                state.msm_accumulator = CycleGroup::affine_point_at_infinity;
                row.collision_check = (row.msm_output_x - row.accumulator_x).invert();
            } else if (entry.add && !row.accumulator_empty) {
                ASSERT((row.base_x != row.accumulator_x) &&
                       "eccvm: attempting to add points with matching x-coordinates");
                row.collision_check = (row.base_x - row.accumulator_x).invert();
            }

            state = updated_state;

            if (entry.mul && next_not_msm) {
                state.msm_accumulator = CycleGroup::affine_point_at_infinity;
            }
            transcript_state.emplace_back(row);
        }

        TranscriptState final_row;
        final_row.pc = updated_state.pc;
        final_row.accumulator_x = (updated_state.accumulator.is_point_at_infinity()) ? 0 : updated_state.accumulator.x;
        final_row.accumulator_y = (updated_state.accumulator.is_point_at_infinity()) ? 0 : updated_state.accumulator.y;
        final_row.accumulator_empty = updated_state.is_accumulator_empty;

        transcript_state.push_back(final_row);
        return transcript_state;
    }
};
} // namespace bb
