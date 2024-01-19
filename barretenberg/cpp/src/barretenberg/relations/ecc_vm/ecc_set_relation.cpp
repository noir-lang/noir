#include "barretenberg/flavor/ecc_vm.hpp"
#include "barretenberg/flavor/relation_definitions.hpp"
#include "ecc_msm_relation.hpp"

namespace bb::honk::sumcheck {

/**
 * @brief Performs list-equivalence checks for the ECCVM
 *
 * @details ECCVMSetRelationImpl validates the correctness of the inputs/outputs of the three main algorithms evaluated
 * by the ECCVM.
 *
 * First term: tuple of (pc, round, wnaf_slice), computed when slicing scalar multipliers into slices,
 *             as part of ECCVMWnafRelation
 * Input source: ECCVMWnafRelation
 * Output source: ECCVMMSMRelation
 *
 *
 * Second term: tuple of (point-counter, P.x, P.y, scalar-multiplier), used in ECCVMWnafRelation and
 *              ECCVMPointTableRelation
 * Input source: ECCVMPointTableRelation
 * Output source: ECCVMMSMRelation
 *
 * Third term: tuple of (point-counter, P.x, P.y, msm-size) from ECCVMMSMRelation
 * Input source: ECCVMMSMRelation
 * Output source: ECCVMTranscriptRelation
 *
 * @tparam FF
 * @tparam AccumulatorTypes
 * @param in
 * @param relation_params
 * @param index
 * @return ECCVMSetRelationImpl<FF>::template Accumulator<AccumulatorTypes>
 */
template <typename FF>
template <typename Accumulator, typename AllEntities, typename Parameters>
Accumulator ECCVMSetRelationImpl<FF>::compute_permutation_numerator(const AllEntities& in, const Parameters& params)
{
    using View = typename Accumulator::View;

    const auto& precompute_round = View(in.precompute_round);
    const auto precompute_round2 = precompute_round + precompute_round;
    const auto precompute_round4 = precompute_round2 + precompute_round2;

    const auto& gamma = params.gamma;
    const auto& beta = params.beta;
    const auto& beta_sqr = params.beta_sqr;
    const auto& beta_cube = params.beta_cube;
    const auto& precompute_pc = View(in.precompute_pc);
    const auto& precompute_select = View(in.precompute_select);

    /**
     * @brief First term: tuple of (pc, round, wnaf_slice), computed when slicing scalar multipliers into slices,
     *        as part of ECCVMWnafRelation.
     *        If precompute_select = 1, tuple entry = (wnaf-slice + point-counter * beta + msm-round * beta_sqr).
     *                       There are 4 tuple entries per row.
     */
    Accumulator numerator(1); // degree-0
    {
        const auto& s0 = View(in.precompute_s1hi);
        const auto& s1 = View(in.precompute_s1lo);

        auto wnaf_slice = s0 + s0;
        wnaf_slice += wnaf_slice;
        wnaf_slice += s1;

        // TODO(@zac-williamson #2226) optimize
        const auto wnaf_slice_input0 = wnaf_slice + gamma + precompute_pc * beta + precompute_round4 * beta_sqr;
        numerator *= wnaf_slice_input0; // degree-1
    }
    {
        const auto& s0 = View(in.precompute_s2hi);
        const auto& s1 = View(in.precompute_s2lo);

        auto wnaf_slice = s0 + s0;
        wnaf_slice += wnaf_slice;
        wnaf_slice += s1;

        // TODO(@zac-williamson #2226) optimize
        const auto wnaf_slice_input1 = wnaf_slice + gamma + precompute_pc * beta + (precompute_round4 + 1) * beta_sqr;
        numerator *= wnaf_slice_input1; // degree-2
    }
    {
        const auto& s0 = View(in.precompute_s3hi);
        const auto& s1 = View(in.precompute_s3lo);

        auto wnaf_slice = s0 + s0;
        wnaf_slice += wnaf_slice;
        wnaf_slice += s1;

        // TODO(@zac-williamson #2226) optimize
        const auto wnaf_slice_input2 = wnaf_slice + gamma + precompute_pc * beta + (precompute_round4 + 2) * beta_sqr;
        numerator *= wnaf_slice_input2; // degree-3
    }
    {
        const auto& s0 = View(in.precompute_s4hi);
        const auto& s1 = View(in.precompute_s4lo);

        auto wnaf_slice = s0 + s0;
        wnaf_slice += wnaf_slice;
        wnaf_slice += s1;
        // TODO(@zac-williamson #2226) optimize
        const auto wnaf_slice_input3 = wnaf_slice + gamma + precompute_pc * beta + (precompute_round4 + 3) * beta_sqr;
        numerator *= wnaf_slice_input3; // degree-4
    }
    {
        // skew product if relevant
        const auto& skew = View(in.precompute_skew);
        const auto& precompute_point_transition = View(in.precompute_point_transition);
        const auto skew_input =
            precompute_point_transition * (skew + gamma + precompute_pc * beta + (precompute_round4 + 4) * beta_sqr) +
            (-precompute_point_transition + 1);
        numerator *= skew_input; // degree-5
    }
    {
        const auto& eccvm_set_permutation_delta = params.eccvm_set_permutation_delta;
        numerator *= precompute_select * (-eccvm_set_permutation_delta + 1) + eccvm_set_permutation_delta; // degree-7
    }

    /**
     * @brief Second term: tuple of (point-counter, P.x, P.y, scalar-multiplier), used in ECCVMWnafRelation and
     * ECCVMPointTableRelation. ECCVMWnafRelation validates the sum of the wnaf slices associated with point-counter
     * equals scalar-multiplier. ECCVMPointTableRelation computes a table of muliples of [P]: { -15[P], -13[P], ...,
     * 15[P] }. We need to validate that scalar-multiplier and [P] = (P.x, P.y) come from MUL opcodes in the transcript
     * columns.
     */
    {
        const auto& table_x = View(in.precompute_tx);
        const auto& table_y = View(in.precompute_ty);

        const auto& precompute_skew = View(in.precompute_skew);
        static constexpr FF negative_inverse_seven = FF(-7).invert();
        auto adjusted_skew = precompute_skew * negative_inverse_seven;

        const auto& wnaf_scalar_sum = View(in.precompute_scalar_sum);
        const auto w0 = convert_to_wnaf<Accumulator>(View(in.precompute_s1hi), View(in.precompute_s1lo));
        const auto w1 = convert_to_wnaf<Accumulator>(View(in.precompute_s2hi), View(in.precompute_s2lo));
        const auto w2 = convert_to_wnaf<Accumulator>(View(in.precompute_s3hi), View(in.precompute_s3lo));
        const auto w3 = convert_to_wnaf<Accumulator>(View(in.precompute_s4hi), View(in.precompute_s4lo));

        auto row_slice = w0;
        row_slice += row_slice;
        row_slice += row_slice;
        row_slice += row_slice;
        row_slice += row_slice;
        row_slice += w1;
        row_slice += row_slice;
        row_slice += row_slice;
        row_slice += row_slice;
        row_slice += row_slice;
        row_slice += w2;
        row_slice += row_slice;
        row_slice += row_slice;
        row_slice += row_slice;
        row_slice += row_slice;
        row_slice += w3;

        auto scalar_sum_full = wnaf_scalar_sum + wnaf_scalar_sum;
        scalar_sum_full += scalar_sum_full;
        scalar_sum_full += scalar_sum_full;
        scalar_sum_full += scalar_sum_full;
        scalar_sum_full += scalar_sum_full;
        scalar_sum_full += scalar_sum_full;
        scalar_sum_full += scalar_sum_full;
        scalar_sum_full += scalar_sum_full;
        scalar_sum_full += scalar_sum_full;
        scalar_sum_full += scalar_sum_full;
        scalar_sum_full += scalar_sum_full;
        scalar_sum_full += scalar_sum_full;
        scalar_sum_full += scalar_sum_full;
        scalar_sum_full += scalar_sum_full;
        scalar_sum_full += scalar_sum_full;
        scalar_sum_full += scalar_sum_full;
        scalar_sum_full += row_slice + adjusted_skew;

        auto precompute_point_transition = View(in.precompute_point_transition);

        auto point_table_init_read =
            (precompute_pc + table_x * beta + table_y * beta_sqr + scalar_sum_full * beta_cube);
        point_table_init_read =
            precompute_point_transition * (point_table_init_read + gamma) + (-precompute_point_transition + 1);

        numerator *= point_table_init_read; // degree-9
    }
    /**
     * @brief Third term: tuple of (point-counter, P.x, P.y, msm-size) from ECCVMMSMRelation.
     *        (P.x, P.y) is the output of a multi-scalar-multiplication evaluated in ECCVMMSMRelation.
     *        We need to validate that the same values (P.x, P.y) are present in the Transcript columns and describe a
     *        multi-scalar multiplication of size `msm-size`, starting at `point-counter`.
     *
     *        If msm_transition_shift = 1, this indicates the current row is the last row of a multiscalar
     * multiplication evaluation. The output of the MSM will be present on `(msm_accumulator_x_shift,
     * msm_accumulator_y_shift)`. The values of `msm_accumulator_x_shift, msm_accumulator_y_shift, msm_pc,
     * msm_size_of_msm` must match up with equivalent values `transcript_msm_output_x, transcript_msm_output_y,
     * transcript_pc, transcript_msm_count` present in the Transcript columns
     */
    {
        const auto& lagrange_first = View(in.lagrange_first);
        const auto& partial_msm_transition_shift = View(in.msm_transition_shift);
        const auto msm_transition_shift = (-lagrange_first + 1) * partial_msm_transition_shift;
        const auto& msm_pc_shift = View(in.msm_pc_shift);

        const auto& msm_x_shift = View(in.msm_accumulator_x_shift);
        const auto& msm_y_shift = View(in.msm_accumulator_y_shift);
        const auto& msm_size = View(in.msm_size_of_msm);

        // msm_transition = 1 when a row BEGINS a new msm
        //
        // row msm tx  acc.x acc.y pc  msm_size
        // i   0       no    no    no  yes
        // i+1 1       yes   yes   yes no
        //
        // at row i we are at the final row of the current msm
        // at row i the value of `msm_size` = size of current msm
        // at row i + 1 we have the final accumulated value of the msm computation
        // at row i + 1 we have updated `pc` to be `(pc at start of msm) + msm_count`
        // at row i + 1 q_msm_transtiion = 1

        auto msm_result_write = msm_pc_shift + msm_x_shift * beta + msm_y_shift * beta_sqr + msm_size * beta_cube;

        // msm_result_write = degree 2
        msm_result_write = msm_transition_shift * (msm_result_write + gamma) + (-msm_transition_shift + 1);
        numerator *= msm_result_write; // degree-11
    }
    return numerator;
}

template <typename FF>
template <typename Accumulator, typename AllEntities, typename Parameters>
Accumulator ECCVMSetRelationImpl<FF>::compute_permutation_denominator(const AllEntities& in, const Parameters& params)
{
    using View = typename Accumulator::View;

    // TODO(@zac-williamson). The degree of this contribution is 17! makes overall relation degree 19.
    // Can optimise by refining the algebra, once we have a stable base to iterate off of.
    const auto& gamma = params.gamma;
    const auto& beta = params.beta;
    const auto& beta_sqr = params.beta_sqr;
    const auto& beta_cube = params.beta_cube;
    const auto& msm_pc = View(in.msm_pc);
    const auto& msm_count = View(in.msm_count);
    const auto& msm_round = View(in.msm_round);

    /**
     * @brief First term: tuple of (pc, round, wnaf_slice), used to determine which points we extract from lookup tables
     * when evaluaing MSMs in ECCVMMsmRelation.
     * These values must be equivalent to the values computed in the 1st term of `compute_permutation_numerator`
     */
    Accumulator denominator(1); // degree-0
    {
        const auto& add1 = View(in.msm_add1);
        const auto& msm_slice1 = View(in.msm_slice1);

        auto wnaf_slice_output1 =
            add1 * (msm_slice1 + gamma + (msm_pc - msm_count) * beta + msm_round * beta_sqr) + (-add1 + 1);
        denominator *= wnaf_slice_output1; // degree-2
    }
    {
        const auto& add2 = View(in.msm_add2);
        const auto& msm_slice2 = View(in.msm_slice2);

        auto wnaf_slice_output2 =
            add2 * (msm_slice2 + gamma + (msm_pc - msm_count - 1) * beta + msm_round * beta_sqr) + (-add2 + 1);
        denominator *= wnaf_slice_output2; // degree-4
    }
    {
        const auto& add3 = View(in.msm_add3);
        const auto& msm_slice3 = View(in.msm_slice3);

        auto wnaf_slice_output3 =
            add3 * (msm_slice3 + gamma + (msm_pc - msm_count - 2) * beta + msm_round * beta_sqr) + (-add3 + 1);
        denominator *= wnaf_slice_output3; // degree-6
    }
    {
        const auto& add4 = View(in.msm_add4);
        const auto& msm_slice4 = View(in.msm_slice4);
        auto wnaf_slice_output4 =
            add4 * (msm_slice4 + gamma + (msm_pc - msm_count - 3) * beta + msm_round * beta_sqr) + (-add4 + 1);
        denominator *= wnaf_slice_output4; // degree-8
    }

    /**
     * @brief Second term: tuple of (transcript_pc, transcript_Px, transcript_Py, z1) OR (transcript_pc, \lambda *
     * transcript_Px, -transcript_Py, z2) for each scalar multiplication in ECCVMTranscriptRelation columns. (the latter
     * term uses the curve endomorphism: \lambda = cube root of unity). These values must be equivalent to the second
     * term values in `compute_permutation_numerator`
     */
    {
        const auto& transcript_pc = View(in.transcript_pc);

        auto transcript_Px = View(in.transcript_Px);
        auto transcript_Py = View(in.transcript_Py);
        auto z1 = View(in.transcript_z1);
        auto z2 = View(in.transcript_z2);
        auto z1_zero = View(in.transcript_z1zero);
        auto z2_zero = View(in.transcript_z2zero);
        auto transcript_mul = View(in.transcript_mul);

        auto lookup_first = (-z1_zero + 1);
        auto lookup_second = (-z2_zero + 1);
        FF endomorphism_base_field_shift = FF::cube_root_of_unity();

        auto transcript_input1 = transcript_pc + transcript_Px * beta + transcript_Py * beta_sqr + z1 * beta_cube;
        auto transcript_input2 = (transcript_pc - 1) + transcript_Px * endomorphism_base_field_shift * beta -
                                 transcript_Py * beta_sqr + z2 * beta_cube;

        // | q_mul | z2_zero | z1_zero | lookup                 |
        // | ----- | ------- | ------- | ---------------------- |
        // | 0     | -       | -       | 1                      |
        // | 1     | 0       | 1       | X + gamma              |
        // | 1     | 1       | 0       | Y + gamma              |
        // | 1     | 1       | 1       | (X + gamma)(Y + gamma) |
        transcript_input1 = (transcript_input1 + gamma) * lookup_first + (-lookup_first + 1);
        transcript_input2 = (transcript_input2 + gamma) * lookup_second + (-lookup_second + 1);
        // point_table_init_write = degree 2

        auto point_table_init_write = transcript_mul * transcript_input1 * transcript_input2 + (-transcript_mul + 1);
        denominator *= point_table_init_write; // degree-13

        // auto point_table_init_write_1 = transcript_mul * transcript_input1 + (-transcript_mul + 1);
        // denominator *= point_table_init_write_1; // degree-11

        // auto point_table_init_write_2 = transcript_mul * transcript_input2 + (-transcript_mul + 1);
        // denominator *= point_table_init_write_2; // degree-14
    }
    /**
     * @brief Third term: tuple of (point-counter, P.x, P.y, msm-size) from ECCVMTranscriptRelation.
     *        (P.x, P.y) is the *claimed* output of a multi-scalar-multiplication evaluated in ECCVMMSMRelation.
     *        We need to validate that the msm output produced in ECCVMMSMRelation is equivalent to the output present
     * in `transcript_msm_output_x, transcript_msm_output_y`, for a given multi-scalar multiplication starting at
     * `transcript_pc` and has size `transcript_msm_count`
     */
    {
        auto transcript_pc_shift = View(in.transcript_pc_shift);
        auto transcript_msm_x = View(in.transcript_msm_x);
        auto transcript_msm_y = View(in.transcript_msm_y);
        auto transcript_msm_transition = View(in.transcript_msm_transition);
        auto transcript_msm_count = View(in.transcript_msm_count);
        auto z1_zero = View(in.transcript_z1zero);
        auto z2_zero = View(in.transcript_z2zero);
        auto transcript_mul = View(in.transcript_mul);

        auto full_msm_count = transcript_msm_count + transcript_mul * ((-z1_zero + 1) + (-z2_zero + 1));
        //      auto count_test = transcript_msm_count
        // msm_result_read = degree 2
        auto msm_result_read =
            transcript_pc_shift + transcript_msm_x * beta + transcript_msm_y * beta_sqr + full_msm_count * beta_cube;

        msm_result_read = transcript_msm_transition * (msm_result_read + gamma) + (-transcript_msm_transition + 1);
        denominator *= msm_result_read; // degree-17
    }
    return denominator;
}

/**
 * @brief Expression for the StandardArithmetic gate.
 * @dbetails The relation is defined as C(in(X)...) =
 *    (q_m * w_r * w_l) + (q_l * w_l) + (q_r * w_r) + (q_o * w_o) + q_c
 *
 * @param evals transformed to `evals + C(in(X)...)*scaling_factor`
 * @param in an std::array containing the fully extended Accumulator edges.
 * @param parameters contains beta, gamma, and public_input_delta, ....
 * @param scaling_factor optional term to scale the evaluation before adding to evals.
 */
template <typename FF>
template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
void ECCVMSetRelationImpl<FF>::accumulate(ContainerOverSubrelations& accumulator,
                                          const AllEntities& in,
                                          const Parameters& params,
                                          const FF& scaling_factor)
{
    using Accumulator = typename std::tuple_element_t<0, ContainerOverSubrelations>;
    using View = typename Accumulator::View;

    // degree-11
    Accumulator numerator_evaluation = compute_permutation_numerator<Accumulator>(in, params);

    // degree-17
    Accumulator denominator_evaluation = compute_permutation_denominator<Accumulator>(in, params);

    const auto& lagrange_first = View(in.lagrange_first);
    const auto& lagrange_last = View(in.lagrange_last);

    const auto& z_perm = View(in.z_perm);
    const auto& z_perm_shift = View(in.z_perm_shift);

    // degree-18
    std::get<0>(accumulator) +=
        ((z_perm + lagrange_first) * numerator_evaluation - (z_perm_shift + lagrange_last) * denominator_evaluation) *
        scaling_factor;

    // Contribution (2)
    std::get<1>(accumulator) += (lagrange_last * z_perm_shift) * scaling_factor;
}

template class ECCVMSetRelationImpl<grumpkin::fr>;
DEFINE_SUMCHECK_RELATION_CLASS(ECCVMSetRelationImpl, flavor::ECCVM);
DEFINE_SUMCHECK_PERMUTATION_CLASS(ECCVMSetRelationImpl, flavor::ECCVM);

} // namespace bb::honk::sumcheck
