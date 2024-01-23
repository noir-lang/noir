#pragma once

#include "./eccvm_builder_types.hpp"
#include "./msm_builder.hpp"
#include "./precomputed_tables_builder.hpp"
#include "./transcript_builder.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/flavor/ecc_vm.hpp"
#include "barretenberg/honk/proof_system/logderivative_library.hpp"
#include "barretenberg/honk/proof_system/permutation_library.hpp"
#include "barretenberg/proof_system/op_queue/ecc_op_queue.hpp"
#include "barretenberg/relations/relation_parameters.hpp"

namespace bb {

template <typename Flavor> class ECCVMCircuitBuilder {
  public:
    using CycleGroup = typename Flavor::CycleGroup;
    using FF = typename Flavor::FF;
    using Polynomial = typename Flavor::Polynomial;

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

    static constexpr size_t NUM_POLYNOMIALS = Flavor::NUM_ALL_ENTITIES;
    static constexpr size_t NUM_WIRES = Flavor::NUM_WIRES;

    using MSM = bb::eccvm::MSM<CycleGroup>;
    using VMOperation = bb::eccvm::VMOperation<CycleGroup>;
    std::shared_ptr<ECCOpQueue> op_queue;
    using ScalarMul = bb::eccvm::ScalarMul<CycleGroup>;
    using ProverPolynomials = typename Flavor::ProverPolynomials;

    ECCVMCircuitBuilder()
        : op_queue(std::make_shared<ECCOpQueue>()){};

    ECCVMCircuitBuilder(std::shared_ptr<ECCOpQueue>& op_queue)
        : op_queue(op_queue){};

    [[nodiscard]] uint32_t get_number_of_muls() const
    {
        uint32_t num_muls = 0;
        for (auto& op : op_queue->raw_ops) {
            if (op.mul) {
                if (op.z1 != 0) {
                    num_muls++;
                }
                if (op.z2 != 0) {
                    num_muls++;
                }
            }
        }
        return num_muls;
    }

    std::vector<MSM> get_msms() const
    {
        const uint32_t num_muls = get_number_of_muls();
        /**
         * For input point [P], return { -15[P], -13[P], ..., -[P], [P], ..., 13[P], 15[P] }
         */
        const auto compute_precomputed_table = [](const AffineElement& base_point) {
            const auto d2 = Element(base_point).dbl();
            std::array<AffineElement, POINT_TABLE_SIZE> table;
            table[POINT_TABLE_SIZE / 2] = base_point;
            for (size_t i = 1; i < POINT_TABLE_SIZE / 2; ++i) {
                table[i + POINT_TABLE_SIZE / 2] = Element(table[i + POINT_TABLE_SIZE / 2 - 1]) + d2;
            }
            for (size_t i = 0; i < POINT_TABLE_SIZE / 2; ++i) {
                table[i] = -table[POINT_TABLE_SIZE - 1 - i];
            }
            return table;
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
        std::vector<MSM> msms;
        std::vector<ScalarMul> active_msm;

        // We start pc at `num_muls` and decrement for each mul processed.
        // This gives us two desired properties:
        // 1: the value of pc at the 1st row = number of muls (easy to check)
        // 2: the value of pc for the final mul = 1
        // The latter point is valuable as it means that we can add empty rows (where pc = 0) and still satisfy our
        // sumcheck relations that involve pc (if we did the other way around, starting at 1 and ending at num_muls,
        // we create a discontinuity in pc values between the last transcript row and the following empty row)
        uint32_t pc = num_muls;

        const auto process_mul = [&active_msm, &pc, &compute_wnaf_slices, &compute_precomputed_table](
                                     const auto& scalar, const auto& base_point) {
            if (scalar != 0) {
                active_msm.push_back(ScalarMul{
                    .pc = pc,
                    .scalar = scalar,
                    .base_point = base_point,
                    .wnaf_slices = compute_wnaf_slices(scalar),
                    .wnaf_skew = (scalar & 1) == 0,
                    .precomputed_table = compute_precomputed_table(base_point),
                });
                pc--;
            }
        };

        for (auto& op : op_queue->raw_ops) {
            if (op.mul) {
                process_mul(op.z1, op.base_point);
                process_mul(op.z2, AffineElement{ op.base_point.x * FF::cube_root_of_unity(), -op.base_point.y });

            } else {
                if (!active_msm.empty()) {
                    msms.push_back(active_msm);
                    active_msm = {};
                }
            }
        }
        if (!active_msm.empty()) {
            msms.push_back(active_msm);
        }

        ASSERT(pc == 0);
        return msms;
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

    void add_accumulate(const AffineElement& to_add)
    {
        op_queue->raw_ops.emplace_back(VMOperation{
            .add = true,
            .mul = false,
            .eq = false,
            .reset = false,
            .base_point = to_add,
            .z1 = 0,
            .z2 = 0,
            .mul_scalar_full = 0,
        });
    }

    void mul_accumulate(const AffineElement& to_mul, const CycleScalar& scalar)
    {
        CycleScalar z1 = 0;
        CycleScalar z2 = 0;
        auto converted = scalar.from_montgomery_form();
        CycleScalar::split_into_endomorphism_scalars(converted, z1, z2);
        z1 = z1.to_montgomery_form();
        z2 = z2.to_montgomery_form();
        op_queue->raw_ops.emplace_back(VMOperation{
            .add = false,
            .mul = true,
            .eq = false,
            .reset = false,
            .base_point = to_mul,
            .z1 = z1,
            .z2 = z2,
            .mul_scalar_full = scalar,
        });
    }

    void eq_and_reset(const AffineElement& expected)
    {
        op_queue->raw_ops.emplace_back(VMOperation{
            .add = false,
            .mul = false,
            .eq = true,
            .reset = true,
            .base_point = expected,
            .z1 = 0,
            .z2 = 0,
            .mul_scalar_full = 0,
        });
    }

    void empty_row()
    {
        op_queue->raw_ops.emplace_back(VMOperation{
            .add = false,
            .mul = false,
            .eq = false,
            .reset = false,
            .base_point = CycleGroup::affine_point_at_infinity,
            .z1 = 0,
            .z2 = 0,
            .mul_scalar_full = 0,
        });
    }

    /**
     * @brief Compute the ECCVM flavor polynomial data required to generate an ECCVM Proof
     *
     * @details RawPolynomial member polynomials that this fn must populate described below
     *          For full details see `flavor/ecc_vm.hpp`
     *
     *          lagrange_first: lagrange_first[0] = 1, 0 elsewhere
     *          lagrange_second: lagrange_second[1] = 1, 0 elsewhere
     *          lagrange_last: lagrange_last[lagrange_last.size() - 1] = 1, 0 elsewhere
     *          transcript_add/mul/eq/reset_accumulator: boolean selectors that toggle add/mul/eq/reset opcodes
     *          transcript_collision_check: used to ensure any point being added into eccvm accumulator does not trigger
     * incomplete addition rules
     *          transcript_msm_transition: is current transcript row the final `mul` opcode of a multiscalar
     multiplication?
     *          transcript_pc: point counter for transcript columns
     *          transcript_msm_count: counts number of muls processed in an ongoing multiscalar multiplication
     *          transcript_Px: input transcript point, x-coordinate
     *          transcript_Py: input transcriot point, y-coordinate
     *          transcript_op: input transcript opcode value
     *          transcript_z1: input transcript scalar multiplier (low component, 128 bits max)
     *          transcript_z2: input transcript scalar multipplier (high component, 128 bits max)
     * N.B. scalar multiplier = transcript_z1 + \lambda * transcript_z2. \lambda = cube root of unity in scalar field
     *          transcript_z1zero: if 1, transcript_z1 must equal 0
     *          transcript_z2zero: if 1, transcript_z2 must equal 0
     *          transcript_accumulator_x: x-coordinate of eccvm accumulator register
     *          transcript_accumulator_y: y-coordinate of eccvm accumulator register
     *          transcript_msm_x: x-coordinate of MSM output
     *          transcript_msm_y: y-coordinate of MSM output
     *          transcript_accumulator_empty: if 1, transcript_accumulator = point at infinity
     *          precompute_pc: point counter for Straus precomputation columns
     *          precompute_select: if 1, evaluate Straus precomputation algorithm at current row
     *          precompute_point_transition: 1 if current row operating on a different point to previous row
     *          precompute_round: round counter for Straus precomputation algorithm
     *          precompute_scalar_sum: accumulating sum of Straus scalar slices
     *          precompute_s1hi/lo: 2-bit hi/lo components of a Straus 4-bit scalar slice
     *          precompute_s2hilo/precompute_s3hi/loprecompute_s4hi/lo: same as above but for a total of 4 Straus 4-bit
     scalar slices
     *          precompute_skew: Straus WNAF skew parameter for a single scalar multiplier
     *          precompute_tx: x-coordinate of point accumulator used to generate Straus lookup table for an input point
     (from transcript)
     *          precompute_tx: x-coordinate of point accumulator used to generate Straus lookup table for an input point
     (from transcript)
     *          precompute_dx: x-coordinate of D = 2 * input point we are evaluating Straus over
     *          precompute_dy: y-coordinate of D
     *          msm_pc: point counter for Straus MSM columns
     *          msm_transition: 1 if current row evaluates different MSM to previous row
     *          msm_add: 1 if we are adding points in Straus MSM algorithm at current row
     *          msm_double: 1 if we are doubling accumulator in Straus MSM algorithm at current row
     *          msm_skew: 1 if we are adding skew points in Straus MSM algorithm at current row
     *          msm_size_of_msm: size of multiscalar multiplication current row is a part of
     *          msm_round: describes which round of the Straus MSM algorithm the current row represents
     *          msm_count: number of points processed for the round indicated by `msm_round`
     *          msm_x1: x-coordinate of potential point in Straus MSM round
     *          msm_y1: y-coordinate of potential point in Straus MSM round
     *          msm_x2: x-coordinate of potential point in Straus MSM round
     *          msm_y2: y-coordinate of potential point in Straus MSM round
     *          msm_x3: x-coordinate of potential point in Straus MSM round
     *          msm_y3: y-coordinate of potential point in Straus MSM round
     *          msm_x4: x-coordinate of potential point in Straus MSM round
     *          msm_y4: y-coordinate of potential point in Straus MSM round
     *          msm_add1: are we adding msm_x1/msm_y1 into accumulator at current round?
     *          msm_add2: are we adding msm_x2/msm_y2 into accumulator at current round?
     *          msm_add3: are we adding msm_x3/msm_y3 into accumulator at current round?
     *          msm_add4: are we adding msm_x4/msm_y4 into accumulator at current round?
     *          msm_lambda1: temp variable used for ecc point addition algorithm if msm_add1 = 1
     *          msm_lambda2: temp variable used for ecc point addition algorithm if msm_add2 = 1
     *          msm_lambda3: temp variable used for ecc point addition algorithm if msm_add3 = 1
     *          msm_lambda4: temp variable used for ecc point addition algorithm if msm_add4 = 1
     *          msm_collision_x1: used to ensure incomplete ecc addition exceptions not triggered if msm_add1 = 1
     *          msm_collision_x2: used to ensure incomplete ecc addition exceptions not triggered if msm_add2 = 1
     *          msm_collision_x3: used to ensure incomplete ecc addition exceptions not triggered if msm_add3 = 1
     *          msm_collision_x4: used to ensure incomplete ecc addition exceptions not triggered if msm_add4 = 1
     *          lookup_read_counts_0: stores number of times a point has been read from a Straus precomputation table
     (reads come from msm_x/y1, msm_x/y2)
     *          lookup_read_counts_1: stores number of times a point has been read from a Straus precomputation table
     (reads come from msm_x/y3, msm_x/y4)
     * @return ProverPolynomials
     */
    ProverPolynomials compute_polynomials()
    {
        const auto msms = get_msms();
        const auto flattened_muls = get_flattened_scalar_muls(msms);

        std::array<std::vector<size_t>, 2> point_table_read_counts;
        const auto transcript_state =
            ECCVMTranscriptBuilder<Flavor>::compute_transcript_state(op_queue->raw_ops, get_number_of_muls());
        const auto precompute_table_state =
            ECCVMPrecomputedTablesBuilder<Flavor>::compute_precompute_state(flattened_muls);
        const auto msm_state =
            ECCVMMSMMBuilder<Flavor>::compute_msm_state(msms, point_table_read_counts, get_number_of_muls());

        const size_t msm_size = msm_state.size();
        const size_t transcript_size = transcript_state.size();
        const size_t precompute_table_size = precompute_table_state.size();

        const size_t num_rows = std::max(precompute_table_size, std::max(msm_size, transcript_size));

        const auto num_rows_log2 = static_cast<size_t>(numeric::get_msb64(num_rows));
        size_t num_rows_pow2 = 1UL << (num_rows_log2 + (1UL << num_rows_log2 == num_rows ? 0 : 1));

        ProverPolynomials polys;
        for (auto& poly : polys.get_all()) {
            poly = Polynomial(num_rows_pow2);
        }

        polys.lagrange_first[0] = 1;
        polys.lagrange_second[1] = 1;
        polys.lagrange_last[polys.lagrange_last.size() - 1] = 1;

        for (size_t i = 0; i < point_table_read_counts[0].size(); ++i) {
            // Explanation of off-by-one offset
            // When computing the WNAF slice for a point at point counter value `pc` and a round index `round`, the row
            // number that computes the slice can be derived. This row number is then mapped to the index of
            // `lookup_read_counts`. We do this mapping in `ecc_msm_relation`. We are off-by-one because we add an empty
            // row at the start of the WNAF columns that is not accounted for (index of lookup_read_counts maps to the
            // row in our WNAF columns that computes a slice for a given value of pc and round)
            polys.lookup_read_counts_0[i + 1] = point_table_read_counts[0][i];
            polys.lookup_read_counts_1[i + 1] = point_table_read_counts[1][i];
        }
        for (size_t i = 0; i < transcript_state.size(); ++i) {
            polys.transcript_accumulator_empty[i] = transcript_state[i].accumulator_empty;
            polys.transcript_add[i] = transcript_state[i].q_add;
            polys.transcript_mul[i] = transcript_state[i].q_mul;
            polys.transcript_eq[i] = transcript_state[i].q_eq;
            polys.transcript_reset_accumulator[i] = transcript_state[i].q_reset_accumulator;
            polys.transcript_msm_transition[i] = transcript_state[i].msm_transition;
            polys.transcript_pc[i] = transcript_state[i].pc;
            polys.transcript_msm_count[i] = transcript_state[i].msm_count;
            polys.transcript_Px[i] = transcript_state[i].base_x;
            polys.transcript_Py[i] = transcript_state[i].base_y;
            polys.transcript_z1[i] = transcript_state[i].z1;
            polys.transcript_z2[i] = transcript_state[i].z2;
            polys.transcript_z1zero[i] = transcript_state[i].z1_zero;
            polys.transcript_z2zero[i] = transcript_state[i].z2_zero;
            polys.transcript_op[i] = transcript_state[i].opcode;
            polys.transcript_accumulator_x[i] = transcript_state[i].accumulator_x;
            polys.transcript_accumulator_y[i] = transcript_state[i].accumulator_y;
            polys.transcript_msm_x[i] = transcript_state[i].msm_output_x;
            polys.transcript_msm_y[i] = transcript_state[i].msm_output_y;
            polys.transcript_collision_check[i] = transcript_state[i].collision_check;
        }

        // TODO(@zac-williamson) if final opcode resets accumulator, all subsequent "is_accumulator_empty" row values
        // must be 1. Ideally we find a way to tweak this so that empty rows that do nothing have column values that are
        // all zero (issue #2217)
        if (transcript_state[transcript_state.size() - 1].accumulator_empty == 1) {
            for (size_t i = transcript_state.size(); i < num_rows_pow2; ++i) {
                polys.transcript_accumulator_empty[i] = 1;
            }
        }
        for (size_t i = 0; i < precompute_table_state.size(); ++i) {
            // first row is always an empty row (to accommodate shifted polynomials which must have 0 as 1st
            // coefficient). All other rows in the precompute_table_state represent active wnaf gates (i.e.
            // precompute_select = 1)
            polys.precompute_select[i] = (i != 0) ? 1 : 0;
            polys.precompute_pc[i] = precompute_table_state[i].pc;
            polys.precompute_point_transition[i] = static_cast<uint64_t>(precompute_table_state[i].point_transition);
            polys.precompute_round[i] = precompute_table_state[i].round;
            polys.precompute_scalar_sum[i] = precompute_table_state[i].scalar_sum;

            polys.precompute_s1hi[i] = precompute_table_state[i].s1;
            polys.precompute_s1lo[i] = precompute_table_state[i].s2;
            polys.precompute_s2hi[i] = precompute_table_state[i].s3;
            polys.precompute_s2lo[i] = precompute_table_state[i].s4;
            polys.precompute_s3hi[i] = precompute_table_state[i].s5;
            polys.precompute_s3lo[i] = precompute_table_state[i].s6;
            polys.precompute_s4hi[i] = precompute_table_state[i].s7;
            polys.precompute_s4lo[i] = precompute_table_state[i].s8;
            // If skew is active (i.e. we need to subtract a base point from the msm result),
            // write `7` into rows.precompute_skew. `7`, in binary representation, equals `-1` when converted into WNAF
            // form
            polys.precompute_skew[i] = precompute_table_state[i].skew ? 7 : 0;

            polys.precompute_dx[i] = precompute_table_state[i].precompute_double.x;
            polys.precompute_dy[i] = precompute_table_state[i].precompute_double.y;
            polys.precompute_tx[i] = precompute_table_state[i].precompute_accumulator.x;
            polys.precompute_ty[i] = precompute_table_state[i].precompute_accumulator.y;
        }

        for (size_t i = 0; i < msm_state.size(); ++i) {
            polys.msm_transition[i] = static_cast<int>(msm_state[i].msm_transition);
            polys.msm_add[i] = static_cast<int>(msm_state[i].q_add);
            polys.msm_double[i] = static_cast<int>(msm_state[i].q_double);
            polys.msm_skew[i] = static_cast<int>(msm_state[i].q_skew);
            polys.msm_accumulator_x[i] = msm_state[i].accumulator_x;
            polys.msm_accumulator_y[i] = msm_state[i].accumulator_y;
            polys.msm_pc[i] = msm_state[i].pc;
            polys.msm_size_of_msm[i] = msm_state[i].msm_size;
            polys.msm_count[i] = msm_state[i].msm_count;
            polys.msm_round[i] = msm_state[i].msm_round;
            polys.msm_add1[i] = static_cast<int>(msm_state[i].add_state[0].add);
            polys.msm_add2[i] = static_cast<int>(msm_state[i].add_state[1].add);
            polys.msm_add3[i] = static_cast<int>(msm_state[i].add_state[2].add);
            polys.msm_add4[i] = static_cast<int>(msm_state[i].add_state[3].add);
            polys.msm_x1[i] = msm_state[i].add_state[0].point.x;
            polys.msm_y1[i] = msm_state[i].add_state[0].point.y;
            polys.msm_x2[i] = msm_state[i].add_state[1].point.x;
            polys.msm_y2[i] = msm_state[i].add_state[1].point.y;
            polys.msm_x3[i] = msm_state[i].add_state[2].point.x;
            polys.msm_y3[i] = msm_state[i].add_state[2].point.y;
            polys.msm_x4[i] = msm_state[i].add_state[3].point.x;
            polys.msm_y4[i] = msm_state[i].add_state[3].point.y;
            polys.msm_collision_x1[i] = msm_state[i].add_state[0].collision_inverse;
            polys.msm_collision_x2[i] = msm_state[i].add_state[1].collision_inverse;
            polys.msm_collision_x3[i] = msm_state[i].add_state[2].collision_inverse;
            polys.msm_collision_x4[i] = msm_state[i].add_state[3].collision_inverse;
            polys.msm_lambda1[i] = msm_state[i].add_state[0].lambda;
            polys.msm_lambda2[i] = msm_state[i].add_state[1].lambda;
            polys.msm_lambda3[i] = msm_state[i].add_state[2].lambda;
            polys.msm_lambda4[i] = msm_state[i].add_state[3].lambda;
            polys.msm_slice1[i] = msm_state[i].add_state[0].slice;
            polys.msm_slice2[i] = msm_state[i].add_state[1].slice;
            polys.msm_slice3[i] = msm_state[i].add_state[2].slice;
            polys.msm_slice4[i] = msm_state[i].add_state[3].slice;
        }

        polys.transcript_mul_shift = Polynomial(polys.transcript_mul.shifted());
        polys.transcript_msm_count_shift = Polynomial(polys.transcript_msm_count.shifted());
        polys.transcript_accumulator_x_shift = Polynomial(polys.transcript_accumulator_x.shifted());
        polys.transcript_accumulator_y_shift = Polynomial(polys.transcript_accumulator_y.shifted());
        polys.precompute_scalar_sum_shift = Polynomial(polys.precompute_scalar_sum.shifted());
        polys.precompute_s1hi_shift = Polynomial(polys.precompute_s1hi.shifted());
        polys.precompute_dx_shift = Polynomial(polys.precompute_dx.shifted());
        polys.precompute_dy_shift = Polynomial(polys.precompute_dy.shifted());
        polys.precompute_tx_shift = Polynomial(polys.precompute_tx.shifted());
        polys.precompute_ty_shift = Polynomial(polys.precompute_ty.shifted());
        polys.msm_transition_shift = Polynomial(polys.msm_transition.shifted());
        polys.msm_add_shift = Polynomial(polys.msm_add.shifted());
        polys.msm_double_shift = Polynomial(polys.msm_double.shifted());
        polys.msm_skew_shift = Polynomial(polys.msm_skew.shifted());
        polys.msm_accumulator_x_shift = Polynomial(polys.msm_accumulator_x.shifted());
        polys.msm_accumulator_y_shift = Polynomial(polys.msm_accumulator_y.shifted());
        polys.msm_count_shift = Polynomial(polys.msm_count.shifted());
        polys.msm_round_shift = Polynomial(polys.msm_round.shifted());
        polys.msm_add1_shift = Polynomial(polys.msm_add1.shifted());
        polys.msm_pc_shift = Polynomial(polys.msm_pc.shifted());
        polys.precompute_pc_shift = Polynomial(polys.precompute_pc.shifted());
        polys.transcript_pc_shift = Polynomial(polys.transcript_pc.shifted());
        polys.precompute_round_shift = Polynomial(polys.precompute_round.shifted());
        polys.transcript_accumulator_empty_shift = Polynomial(polys.transcript_accumulator_empty.shifted());
        polys.precompute_select_shift = Polynomial(polys.precompute_select.shifted());
        return polys;
    }

    bool check_circuit()
    {
        const FF gamma = FF::random_element();
        const FF beta = FF::random_element();
        const FF beta_sqr = beta.sqr();
        const FF beta_cube = beta_sqr * beta;
        auto eccvm_set_permutation_delta =
            gamma * (gamma + beta_sqr) * (gamma + beta_sqr + beta_sqr) * (gamma + beta_sqr + beta_sqr + beta_sqr);
        eccvm_set_permutation_delta = eccvm_set_permutation_delta.invert();
        bb::RelationParameters<typename Flavor::FF> params{
            .eta = 0,
            .beta = beta,
            .gamma = gamma,
            .public_input_delta = 0,
            .lookup_grand_product_delta = 0,
            .beta_sqr = beta_sqr,
            .beta_cube = beta_cube,
            .eccvm_set_permutation_delta = eccvm_set_permutation_delta,
        };

        auto polynomials = compute_polynomials();
        const size_t num_rows = polynomials.get_polynomial_size();
        bb::honk::logderivative_library::compute_logderivative_inverse<Flavor, honk::sumcheck::ECCVMLookupRelation<FF>>(
            polynomials, params, num_rows);

        honk::permutation_library::compute_permutation_grand_product<Flavor, honk::sumcheck::ECCVMSetRelation<FF>>(
            num_rows, polynomials, params);

        polynomials.z_perm_shift = Polynomial(polynomials.z_perm.shifted());

        const auto evaluate_relation = [&]<typename Relation>(const std::string& relation_name) {
            typename Relation::SumcheckArrayOfValuesOverSubrelations result;
            for (auto& r : result) {
                r = 0;
            }
            constexpr size_t NUM_SUBRELATIONS = result.size();

            for (size_t i = 0; i < num_rows; ++i) {
                Relation::accumulate(result, polynomials.get_row(i), params, 1);

                bool x = true;
                for (size_t j = 0; j < NUM_SUBRELATIONS; ++j) {
                    if (result[j] != 0) {
                        info("Relation ", relation_name, ", subrelation index ", j, " failed at row ", i);
                        x = false;
                    }
                }
                if (!x) {
                    return false;
                }
            }
            return true;
        };

        bool result = true;
        result = result && evaluate_relation.template operator()<honk::sumcheck::ECCVMTranscriptRelation<FF>>(
                               "ECCVMTranscriptRelation");
        result = result && evaluate_relation.template operator()<honk::sumcheck::ECCVMPointTableRelation<FF>>(
                               "ECCVMPointTableRelation");
        result =
            result && evaluate_relation.template operator()<honk::sumcheck::ECCVMWnafRelation<FF>>("ECCVMWnafRelation");
        result =
            result && evaluate_relation.template operator()<honk::sumcheck::ECCVMMSMRelation<FF>>("ECCVMMSMRelation");
        result =
            result && evaluate_relation.template operator()<honk::sumcheck::ECCVMSetRelation<FF>>("ECCVMSetRelation");

        using LookupRelation = honk::sumcheck::ECCVMLookupRelation<FF>;
        typename honk::sumcheck::ECCVMLookupRelation<typename Flavor::FF>::SumcheckArrayOfValuesOverSubrelations
            lookup_result;
        for (auto& r : lookup_result) {
            r = 0;
        }
        for (size_t i = 0; i < num_rows; ++i) {
            LookupRelation::accumulate(lookup_result, polynomials.get_row(i), params, 1);
        }
        for (auto r : lookup_result) {
            if (r != 0) {
                info("Relation ECCVMLookupRelation failed.");
                return false;
            }
        }
        return result;
    }

    [[nodiscard]] size_t get_num_gates() const
    {
        // TODO(@zac-williamson) once we have a stable base to work off of, optimize this method!
        // (issue #2218)
        const auto msms = get_msms();
        const auto flattened_muls = get_flattened_scalar_muls(msms);

        std::array<std::vector<size_t>, 2> point_table_read_counts;
        const auto transcript_state =
            ECCVMTranscriptBuilder<Flavor>::compute_transcript_state(op_queue->raw_ops, get_number_of_muls());
        const auto precompute_table_state =
            ECCVMPrecomputedTablesBuilder<Flavor>::compute_precompute_state(flattened_muls);
        const auto msm_state =
            ECCVMMSMMBuilder<Flavor>::compute_msm_state(msms, point_table_read_counts, get_number_of_muls());

        const size_t msm_size = msm_state.size();
        const size_t transcript_size = transcript_state.size();
        const size_t precompute_table_size = precompute_table_state.size();

        const size_t num_rows = std::max(precompute_table_size, std::max(msm_size, transcript_size));
        return num_rows;
    }

    [[nodiscard]] size_t get_circuit_subgroup_size(const size_t num_rows) const
    {

        const auto num_rows_log2 = static_cast<size_t>(numeric::get_msb64(num_rows));
        size_t num_rows_pow2 = 1UL << (num_rows_log2 + (1UL << num_rows_log2 == num_rows ? 0 : 1));
        return num_rows_pow2;
    }
};
} // namespace bb
