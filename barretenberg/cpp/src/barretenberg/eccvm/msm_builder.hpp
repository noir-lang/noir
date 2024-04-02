#pragma once

#include <cstddef>

#include "./eccvm_builder_types.hpp"
#include "barretenberg/stdlib_circuit_builders/op_queue/ecc_op_queue.hpp"

namespace bb {

class ECCVMMSMMBuilder {
  public:
    using CycleGroup = curve::BN254::Group;
    using FF = curve::Grumpkin::ScalarField;
    using Element = typename CycleGroup::element;
    using AffineElement = typename CycleGroup::affine_element;

    static constexpr size_t ADDITIONS_PER_ROW = bb::eccvm::ADDITIONS_PER_ROW;
    static constexpr size_t NUM_SCALAR_BITS = bb::eccvm::NUM_SCALAR_BITS;
    static constexpr size_t WNAF_SLICE_BITS = bb::eccvm::WNAF_SLICE_BITS;

    struct alignas(64) MSMState {
        uint32_t pc = 0;
        uint32_t msm_size = 0;
        uint32_t msm_count = 0;
        uint32_t msm_round = 0;
        bool msm_transition = false;
        bool q_add = false;
        bool q_double = false;
        bool q_skew = false;

        struct AddState {
            bool add = false;
            int slice = 0;
            AffineElement point{ 0, 0 };
            FF lambda = 0;
            FF collision_inverse = 0;
        };
        std::array<AddState, 4> add_state{ AddState{ false, 0, { 0, 0 }, 0, 0 },
                                           AddState{ false, 0, { 0, 0 }, 0, 0 },
                                           AddState{ false, 0, { 0, 0 }, 0, 0 },
                                           AddState{ false, 0, { 0, 0 }, 0, 0 } };
        FF accumulator_x = 0;
        FF accumulator_y = 0;
    };

    struct alignas(64) MSMRowTranscript {
        std::array<FF, 4> lambda_numerator;
        std::array<FF, 4> lambda_denominator;
        Element accumulator_in;
        Element accumulator_out;
    };

    struct alignas(64) AdditionTrace {
        Element p1;
        Element p2;
        Element p3;
        bool predicate;
        bool is_double;
    };

    /**
     * @brief Computes the row values for the Straus MSM columns of the ECCVM.
     *
     * For a detailed description of the Straus algorithm and its relation to the ECCVM, please see
     * https://hackmd.io/@aztec-network/rJ5xhuCsn
     *
     * @param msms
     * @param point_table_read_counts
     * @param total_number_of_muls
     * @return std::vector<MSMState>
     */
    static std::vector<MSMState> compute_msm_state(const std::vector<bb::eccvm::MSM<CycleGroup>>& msms,
                                                   std::array<std::vector<size_t>, 2>& point_table_read_counts,
                                                   const uint32_t total_number_of_muls,
                                                   const size_t num_msm_rows)
    {
        // N.B. the following comments refer to a "point lookup table" frequently.
        // To perform a scalar multiplicaiton of a point [P] by a scalar x, we compute multiples of [P] and store in a
        // table: specifically: -15[P], -13[P], ..., -3[P], -[P], [P], 3[P], ..., 15[P] when we define our point lookup
        // table, we have 2 write columns and 4 read columns when we perform a read on a given row, we need to increment
        // the read count on the respective write column by 1 we can define the following struture: 1st write column =
        // positive 2nd write column = negative the row number is a function of pc and slice value row = pc_delta *
        // rows_per_point_table + some function of the slice value pc_delta = total_number_of_muls - pc
        // std::vector<std::array<size_t, > point_table_read_counts;
        const size_t table_rows = static_cast<size_t>(total_number_of_muls) * 8;
        point_table_read_counts[0].reserve(table_rows);
        point_table_read_counts[1].reserve(table_rows);
        for (size_t i = 0; i < table_rows; ++i) {
            point_table_read_counts[0].emplace_back(0);
            point_table_read_counts[1].emplace_back(0);
        }
        const auto update_read_counts = [&](const size_t pc, const int slice) {
            // When we compute our wnaf/point tables, we start with the point with the largest pc value.
            // i.e. if we are reading a slice for point with a point counter value `pc`,
            // its position in the wnaf/point table (relative to other points) will be `total_number_of_muls - pc`
            const size_t pc_delta = total_number_of_muls - pc;
            const size_t pc_offset = pc_delta * 8;
            bool slice_negative = slice < 0;
            const int slice_row = (slice + 15) / 2;

            const size_t column_index = slice_negative ? 1 : 0;

            /**
             * When computing `point_table_read_counts`, we need the *table index* that a given point belongs to.
             * the slice value is in *compressed* windowed-non-adjacent-form format:
             * A non-compressed WNAF slice is in the range: `-15, -13, ..., 15`
             * In compressed form, tney become `0, ..., 15`
             * The *point table* format is the following:
             * (for positive point table) T[0] = P, T[1] = PT, ..., T[7] = 15P
             * (for negative point table) T[0] = -P, T[1] = -3P, ..., T[15] = -15P
             * i.e. if the slice value is negative, we can use the compressed WNAF directly as the table index
             *      if the slice value is positive, we must take `15 - compressedWNAF` to get the table index
             */
            if (slice_negative) {
                point_table_read_counts[column_index][pc_offset + static_cast<size_t>(slice_row)]++;
            } else {
                point_table_read_counts[column_index][pc_offset + 15 - static_cast<size_t>(slice_row)]++;
            }
        };

        // compute which row index each multiscalar multiplication will start at.
        // also compute the program counter index that each multiscalar multiplication will start at.
        // we use this information to populate the MSM row data across multiple threads
        std::vector<size_t> msm_row_indices;
        std::vector<size_t> pc_indices;
        msm_row_indices.reserve(msms.size() + 1);
        pc_indices.reserve(msms.size() + 1);

        msm_row_indices.push_back(1);
        pc_indices.push_back(total_number_of_muls);
        for (const auto& msm : msms) {
            const size_t rows = ECCOpQueue::get_msm_row_count_for_single_msm(msm.size());
            msm_row_indices.push_back(msm_row_indices.back() + rows);
            pc_indices.push_back(pc_indices.back() - msm.size());
        }

        static constexpr size_t num_rounds = NUM_SCALAR_BITS / WNAF_SLICE_BITS;
        std::vector<MSMState> msm_state(num_msm_rows);
        // start with empty row (shiftable polynomials must have 0 as first coefficient)
        msm_state[0] = (MSMState{});

        // compute "read counts" so that we can determine the number of times entries in our log-derivative lookup
        // tables are called.
        // Note: this part is single-threaded. THe amount of compute is low, however, so this is likely not a big
        // concern.
        for (size_t i = 0; i < msms.size(); ++i) {

            for (size_t j = 0; j < num_rounds; ++j) {
                uint32_t pc = static_cast<uint32_t>(pc_indices[i]);
                const auto& msm = msms[i];
                const size_t msm_size = msm.size();
                const size_t rows_per_round =
                    (msm_size / ADDITIONS_PER_ROW) + (msm_size % ADDITIONS_PER_ROW != 0 ? 1 : 0);

                for (size_t k = 0; k < rows_per_round; ++k) {
                    const size_t points_per_row =
                        (k + 1) * ADDITIONS_PER_ROW > msm_size ? msm_size % ADDITIONS_PER_ROW : ADDITIONS_PER_ROW;
                    const size_t idx = k * ADDITIONS_PER_ROW;
                    for (size_t m = 0; m < ADDITIONS_PER_ROW; ++m) {
                        bool add = points_per_row > m;
                        if (add) {
                            int slice = add ? msm[idx + m].wnaf_slices[j] : 0;
                            update_read_counts(pc - idx - m, slice);
                        }
                    }
                }

                if (j == num_rounds - 1) {
                    for (size_t k = 0; k < rows_per_round; ++k) {
                        const size_t points_per_row =
                            (k + 1) * ADDITIONS_PER_ROW > msm_size ? msm_size % ADDITIONS_PER_ROW : ADDITIONS_PER_ROW;
                        const size_t idx = k * ADDITIONS_PER_ROW;
                        for (size_t m = 0; m < 4; ++m) {
                            bool add = points_per_row > m;

                            if (add) {
                                update_read_counts(pc - idx - m, msm[idx + m].wnaf_skew ? -1 : -15);
                            }
                        }
                    }
                }
            }
        }

        // The execution trace data for the MSM columns requires knowledge of intermediate values from *affine* point
        // addition. The naive solution to compute this data requires 2 field inversions per in-circuit group addition
        // evaluation. This is bad! To avoid this, we split the witness computation algorithm into 3 steps. Step 1:
        // compute the execution trace group operations in *projective* coordinates Step 2: use batch inversion trick to
        // convert all point traces into affine coordinates Step 3: populate the full execution trace, including the
        // intermediate values from affine group operations This section sets up the data structures we need to store
        // all intermediate ECC operations in projective form
        const size_t num_point_adds_and_doubles = (num_msm_rows - 2) * 4;
        const size_t num_accumulators = num_msm_rows - 1;
        const size_t num_points_in_trace = (num_point_adds_and_doubles * 3) + num_accumulators;
        // We create 1 vector to store the entire point trace. We split into multiple containers using std::span
        // (we want 1 vector object to more efficiently batch normalize points)
        std::vector<Element> point_trace(num_points_in_trace);
        // the point traces record group operations. Either p1 + p2 = p3, or p1.dbl() = p3
        std::span<Element> p1_trace(&point_trace[0], num_point_adds_and_doubles);
        std::span<Element> p2_trace(&point_trace[num_point_adds_and_doubles], num_point_adds_and_doubles);
        std::span<Element> p3_trace(&point_trace[num_point_adds_and_doubles * 2], num_point_adds_and_doubles);
        // operation_trace records whether an entry in the p1/p2/p3 trace represents a point addition or doubling
        std::vector<bool> operation_trace(num_point_adds_and_doubles);
        // accumulator_trace tracks the value of the ECCVM accumulator for each row
        std::span<Element> accumulator_trace(&point_trace[num_point_adds_and_doubles * 3], num_accumulators);

        // we start the accumulator at the point at infinity
        accumulator_trace[0] = (CycleGroup::affine_point_at_infinity);

        // populate point trace data, and the components of the MSM execution trace that do not relate to affine point
        // operations
        run_loop_in_parallel(msms.size(), [&](size_t start, size_t end) {
            for (size_t i = start; i < end; i++) {
                Element accumulator = CycleGroup::affine_point_at_infinity;
                const auto& msm = msms[i];
                size_t msm_row_index = msm_row_indices[i];
                const size_t msm_size = msm.size();
                const size_t rows_per_round =
                    (msm_size / ADDITIONS_PER_ROW) + (msm_size % ADDITIONS_PER_ROW != 0 ? 1 : 0);
                size_t trace_index = (msm_row_indices[i] - 1) * 4;

                for (size_t j = 0; j < num_rounds; ++j) {
                    const uint32_t pc = static_cast<uint32_t>(pc_indices[i]);

                    for (size_t k = 0; k < rows_per_round; ++k) {
                        const size_t points_per_row =
                            (k + 1) * ADDITIONS_PER_ROW > msm_size ? msm_size % ADDITIONS_PER_ROW : ADDITIONS_PER_ROW;
                        auto& row = msm_state[msm_row_index];
                        const size_t idx = k * ADDITIONS_PER_ROW;
                        row.msm_transition = (j == 0) && (k == 0);
                        for (size_t m = 0; m < ADDITIONS_PER_ROW; ++m) {

                            auto& add_state = row.add_state[m];
                            add_state.add = points_per_row > m;
                            int slice = add_state.add ? msm[idx + m].wnaf_slices[j] : 0;
                            // In the MSM columns in the ECCVM circuit, we can add up to 4 points per row.
                            // if `row.add_state[m].add = 1`, this indicates that we want to add the `m`'th point in
                            // the MSM columns into the MSM accumulator `add_state.slice` = A 4-bit WNAF slice of
                            // the scalar multiplier associated with the point we are adding (the specific slice
                            // chosen depends on the value of msm_round) (WNAF = windowed-non-adjacent-form. Value
                            // range is `-15, -13,
                            // ..., 15`) If `add_state.add = 1`, we want `add_state.slice` to be the *compressed*
                            // form of the WNAF slice value. (compressed = no gaps in the value range. i.e. -15,
                            // -13, ..., 15 maps to 0, ... , 15)
                            add_state.slice = add_state.add ? (slice + 15) / 2 : 0;
                            add_state.point = add_state.add
                                                  ? msm[idx + m].precomputed_table[static_cast<size_t>(add_state.slice)]
                                                  : AffineElement{ 0, 0 };

                            // predicate logic:
                            // add_predicate should normally equal add_state.add
                            // However! if j == 0 AND k == 0 AND m == 0 this implies we are examing the 1st point
                            // addition of a new MSM In this case, we do NOT add the 1st point into the accumulator,
                            // instead we SET the accumulator to equal the 1st point. add_predicate is used to
                            // determine whether we add the output of a point addition into the accumulator,
                            // therefore if j == 0 AND k == 0 AND m == 0, add_predicate = 0 even if add_state.add =
                            // true
                            bool add_predicate = (m == 0 ? (j != 0 || k != 0) : add_state.add);

                            Element p1 = (m == 0) ? Element(add_state.point) : accumulator;
                            Element p2 = (m == 0) ? accumulator : Element(add_state.point);

                            accumulator = add_predicate ? (accumulator + add_state.point) : Element(p1);
                            p1_trace[trace_index] = p1;
                            p2_trace[trace_index] = p2;
                            p3_trace[trace_index] = accumulator;
                            operation_trace[trace_index] = false;
                            trace_index++;
                        }
                        accumulator_trace[msm_row_index] = accumulator;
                        row.q_add = true;
                        row.q_double = false;
                        row.q_skew = false;
                        row.msm_round = static_cast<uint32_t>(j);
                        row.msm_size = static_cast<uint32_t>(msm_size);
                        row.msm_count = static_cast<uint32_t>(idx);
                        row.pc = pc;
                        msm_row_index++;
                    }
                    // doubling
                    if (j < num_rounds - 1) {
                        auto& row = msm_state[msm_row_index];
                        row.msm_transition = false;
                        row.msm_round = static_cast<uint32_t>(j + 1);
                        row.msm_size = static_cast<uint32_t>(msm_size);
                        row.msm_count = static_cast<uint32_t>(0);
                        row.q_add = false;
                        row.q_double = true;
                        row.q_skew = false;
                        for (size_t m = 0; m < 4; ++m) {

                            auto& add_state = row.add_state[m];
                            add_state.add = false;
                            add_state.slice = 0;
                            add_state.point = { 0, 0 };
                            add_state.collision_inverse = 0;

                            p1_trace[trace_index] = accumulator;
                            p2_trace[trace_index] = accumulator;
                            accumulator = accumulator.dbl();
                            p3_trace[trace_index] = accumulator;
                            operation_trace[trace_index] = true;
                            trace_index++;
                        }
                        accumulator_trace[msm_row_index] = accumulator;
                        msm_row_index++;
                    } else {
                        for (size_t k = 0; k < rows_per_round; ++k) {
                            auto& row = msm_state[msm_row_index];

                            const size_t points_per_row = (k + 1) * ADDITIONS_PER_ROW > msm_size
                                                              ? msm_size % ADDITIONS_PER_ROW
                                                              : ADDITIONS_PER_ROW;
                            const size_t idx = k * ADDITIONS_PER_ROW;
                            row.msm_transition = false;

                            Element acc_expected = accumulator;

                            for (size_t m = 0; m < 4; ++m) {
                                auto& add_state = row.add_state[m];
                                add_state.add = points_per_row > m;
                                add_state.slice = add_state.add ? msm[idx + m].wnaf_skew ? 7 : 0 : 0;

                                add_state.point =
                                    add_state.add ? msm[idx + m].precomputed_table[static_cast<size_t>(add_state.slice)]
                                                  : AffineElement{ 0, 0 };
                                bool add_predicate = add_state.add ? msm[idx + m].wnaf_skew : false;
                                auto p1 = accumulator;
                                accumulator = add_predicate ? accumulator + add_state.point : accumulator;
                                p1_trace[trace_index] = p1;
                                p2_trace[trace_index] = add_state.point;
                                p3_trace[trace_index] = accumulator;
                                operation_trace[trace_index] = false;
                                trace_index++;
                            }
                            row.q_add = false;
                            row.q_double = false;
                            row.q_skew = true;
                            row.msm_round = static_cast<uint32_t>(j + 1);
                            row.msm_size = static_cast<uint32_t>(msm_size);
                            row.msm_count = static_cast<uint32_t>(idx);
                            row.pc = pc;
                            accumulator_trace[msm_row_index] = accumulator;
                            msm_row_index++;
                        }
                    }
                }
            }
        });

        // Normalize the points in the point trace
        run_loop_in_parallel(point_trace.size(), [&](size_t start, size_t end) {
            Element::batch_normalize(&point_trace[start], end - start);
        });

        // inverse_trace is used to compute the value of the `collision_inverse` column in the ECCVM.
        std::vector<FF> inverse_trace(num_point_adds_and_doubles);
        run_loop_in_parallel(num_point_adds_and_doubles, [&](size_t start, size_t end) {
            for (size_t i = start; i < end; ++i) {
                if (operation_trace[i]) {
                    inverse_trace[i] = (p1_trace[i].y + p1_trace[i].y);
                } else {
                    inverse_trace[i] = (p2_trace[i].x - p1_trace[i].x);
                }
            }
            FF::batch_invert(&inverse_trace[start], end - start);
        });

        // complete the computation of the ECCVM execution trace, by adding the affine intermediate point data
        // i.e. row.accumulator_x, row.accumulator_y, row.add_state[0...3].collision_inverse,
        // row.add_state[0...3].lambda
        run_loop_in_parallel(msms.size(), [&](size_t start, size_t end) {
            for (size_t i = start; i < end; i++) {
                const auto& msm = msms[i];
                size_t trace_index = ((msm_row_indices[i] - 1) * ADDITIONS_PER_ROW);
                size_t msm_row_index = msm_row_indices[i];
                // 1st MSM row will have accumulator equal to the previous MSM output
                // (or point at infinity for 1st MSM)
                size_t accumulator_index = msm_row_indices[i] - 1;
                const size_t msm_size = msm.size();
                const size_t rows_per_round =
                    (msm_size / ADDITIONS_PER_ROW) + (msm_size % ADDITIONS_PER_ROW != 0 ? 1 : 0);

                for (size_t j = 0; j < num_rounds; ++j) {
                    for (size_t k = 0; k < rows_per_round; ++k) {
                        auto& row = msm_state[msm_row_index];
                        const Element& normalized_accumulator = accumulator_trace[accumulator_index];
                        const FF& acc_x = normalized_accumulator.is_point_at_infinity() ? 0 : normalized_accumulator.x;
                        const FF& acc_y = normalized_accumulator.is_point_at_infinity() ? 0 : normalized_accumulator.y;
                        row.accumulator_x = acc_x;
                        row.accumulator_y = acc_y;

                        for (size_t m = 0; m < ADDITIONS_PER_ROW; ++m) {
                            auto& add_state = row.add_state[m];
                            bool add_predicate = (m == 0 ? (j != 0 || k != 0) : add_state.add);

                            const auto& inverse = inverse_trace[trace_index];
                            const auto& p1 = p1_trace[trace_index];
                            const auto& p2 = p2_trace[trace_index];
                            add_state.collision_inverse = add_predicate ? inverse : 0;
                            add_state.lambda = add_predicate ? (p2.y - p1.y) * inverse : 0;
                            trace_index++;
                        }
                        accumulator_index++;
                        msm_row_index++;
                    }

                    if (j < num_rounds - 1) {
                        MSMState& row = msm_state[msm_row_index];
                        const Element& normalized_accumulator = accumulator_trace[accumulator_index];
                        const FF& acc_x = normalized_accumulator.is_point_at_infinity() ? 0 : normalized_accumulator.x;
                        const FF& acc_y = normalized_accumulator.is_point_at_infinity() ? 0 : normalized_accumulator.y;
                        row.accumulator_x = acc_x;
                        row.accumulator_y = acc_y;

                        for (size_t m = 0; m < 4; ++m) {
                            auto& add_state = row.add_state[m];
                            add_state.collision_inverse = 0;
                            const FF& dx = p1_trace[trace_index].x;
                            const FF& inverse = inverse_trace[trace_index];
                            add_state.lambda = ((dx + dx + dx) * dx) * inverse;
                            trace_index++;
                        }
                        accumulator_index++;
                        msm_row_index++;
                    } else {
                        for (size_t k = 0; k < rows_per_round; ++k) {
                            MSMState& row = msm_state[msm_row_index];
                            const Element& normalized_accumulator = accumulator_trace[accumulator_index];

                            const size_t idx = k * ADDITIONS_PER_ROW;

                            const FF& acc_x =
                                normalized_accumulator.is_point_at_infinity() ? 0 : normalized_accumulator.x;
                            const FF& acc_y =
                                normalized_accumulator.is_point_at_infinity() ? 0 : normalized_accumulator.y;
                            row.accumulator_x = acc_x;
                            row.accumulator_y = acc_y;

                            for (size_t m = 0; m < ADDITIONS_PER_ROW; ++m) {
                                auto& add_state = row.add_state[m];
                                bool add_predicate = add_state.add ? msm[idx + m].wnaf_skew : false;

                                const auto& inverse = inverse_trace[trace_index];
                                const auto& p1 = p1_trace[trace_index];
                                const auto& p2 = p2_trace[trace_index];
                                add_state.collision_inverse = add_predicate ? inverse : 0;
                                add_state.lambda = add_predicate ? (p2.y - p1.y) * inverse : 0;
                                trace_index++;
                            }
                            accumulator_index++;
                            msm_row_index++;
                        }
                    }
                }
            }
        });

        // populate the final row in the MSM execution trace.
        // we always require 1 extra row at the end of the trace, because the accumulator x/y coordinates for row `i`
        // are present at row `i+1`
        Element final_accumulator(accumulator_trace.back());
        MSMState& final_row = msm_state.back();
        final_row.pc = static_cast<uint32_t>(pc_indices.back());
        final_row.msm_transition = true;
        final_row.accumulator_x = final_accumulator.is_point_at_infinity() ? 0 : final_accumulator.x;
        final_row.accumulator_y = final_accumulator.is_point_at_infinity() ? 0 : final_accumulator.y;
        final_row.msm_size = 0;
        final_row.msm_count = 0;
        final_row.q_add = false;
        final_row.q_double = false;
        final_row.q_skew = false;
        final_row.add_state = { typename MSMState::AddState{ false, 0, AffineElement{ 0, 0 }, 0, 0 },
                                typename MSMState::AddState{ false, 0, AffineElement{ 0, 0 }, 0, 0 },
                                typename MSMState::AddState{ false, 0, AffineElement{ 0, 0 }, 0, 0 },
                                typename MSMState::AddState{ false, 0, AffineElement{ 0, 0 }, 0, 0 } };

        return msm_state;
    }
};
} // namespace bb
