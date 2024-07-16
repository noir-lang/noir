#pragma once
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/relations/relation_types.hpp"

namespace bb {

template <typename FF_> class AuxiliaryRelationImpl {
  public:
    using FF = FF_;
    /*
     * TODO(https://github.com/AztecProtocol/barretenberg/issues/757): Investigate optimizations.
     * It seems that we could have:
     *     static constexpr std::array<size_t, 6> SUBRELATION_PARTIAL_LENGTHS{
     *     5 // auxiliary sub-relation;
     *     6 // ROM consistency sub-relation 1
     *     6 // ROM consistency sub-relation 2
     *     6 // RAM consistency sub-relation 1
     *     5 // RAM consistency sub-relation 2
     *     5 // RAM consistency sub-relation 3
     * };
     *
     * and
     *
     *     static constexpr std::array<size_t, 6> TOTAL_LENGTH_ADJUSTMENTS{
     *     6, // auxiliary sub-relation
     *     0, // ROM consistency sub-relation 1
     *     0, // ROM consistency sub-relation 2
     *     3, // RAM consistency sub-relation 1
     *     0, // RAM consistency sub-relation 2
     *     1  // RAM consistency sub-relation 3
     * };
     */

    static constexpr std::array<size_t, 6> SUBRELATION_PARTIAL_LENGTHS{
        6, // auxiliary sub-relation;
        6, // ROM consistency sub-relation 1
        6, // ROM consistency sub-relation 2
        6, // RAM consistency sub-relation 1
        6, // RAM consistency sub-relation 2
        6  // RAM consistency sub-relation 3
    };
    /**
     * @brief For ZK-Flavors: The degrees of subrelations considered as polynomials only in witness polynomials,
     * i.e. all selectors and public polynomials are treated as constants.
     *
     */
    static constexpr std::array<size_t, 6> SUBRELATION_WITNESS_DEGREES{
        2, // auxiliary sub-relation;
        2, // ROM consistency sub-relation 1: adjacent values match if adjacent indices match and next access is a read
           // operation
        2, // ROM consistency sub-relation 2: index is monotonously increasing
        3, // RAM consistency sub-relation 1: adjacent values match if adjacent indices match and next access is a read
           // operation
        2, // RAM consistency sub-relation 2: index is monotonously increasing
        2  // RAM consistency sub-relation 3: next gate access type is boolean
    };

    static constexpr std::array<size_t, 6> TOTAL_LENGTH_ADJUSTMENTS{
        1, // auxiliary sub-relation
        1, // ROM consistency sub-relation 1
        1, // ROM consistency sub-relation 2
        1, // RAM consistency sub-relation 1
        1, // RAM consistency sub-relation 2
        1  // RAM consistency sub-relation 3
    };

    /**
     * @brief Returns true if the contribution from all subrelations for the provided inputs is identically zero
     *
     */
    template <typename AllEntities> inline static bool skip(const AllEntities& in) { return in.q_aux.is_zero(); }

    /**
     * @brief Expression for the generalized permutation sort gate.
     * @details The following explanation is reproduced from the Plonk analog 'plookup_auxiliary_widget':
     * Adds contributions for identities associated with several custom gates:
     *  * RAM/ROM read-write consistency check
     *  * RAM timestamp difference consistency check
     *  * RAM/ROM index difference consistency check
     *  * Bigfield product evaluation (3 in total)
     *  * Bigfield limb accumulation (2 in total)
     *
     * Multiple selectors are used to 'switch' aux gates on/off according to the following pattern:
     *
     * | gate type                    | q_aux | q_1 | q_2 | q_3 | q_4 | q_m | q_c | q_arith |
     * | ---------------------------- | ----- | --- | --- | --- | --- | --- | --- | ------  |
     * | Bigfield Limb Accumulation 1 | 1     | 0   | 0   | 1   | 1   | 0   | --- | 0       |
     * | Bigfield Limb Accumulation 2 | 1     | 0   | 0   | 1   | 0   | 1   | --- | 0       |
     * | Bigfield Product 1           | 1     | 0   | 1   | 1   | 0   | 0   | --- | 0       |
     * | Bigfield Product 2           | 1     | 0   | 1   | 0   | 1   | 0   | --- | 0       |
     * | Bigfield Product 3           | 1     | 0   | 1   | 0   | 0   | 1   | --- | 0       |
     * | RAM/ROM access gate          | 1     | 1   | 0   | 0   | 0   | 1   | --- | 0       |
     * | RAM timestamp check          | 1     | 1   | 0   | 0   | 1   | 0   | --- | 0       |
     * | ROM consistency check        | 1     | 1   | 1   | 0   | 0   | 0   | --- | 0       |
     * | RAM consistency check        | 1     | 0   | 0   | 0   | 0   | 0   | 0   | 1       |
     *
     * N.B. The RAM consistency check identity is degree 3. To keep the overall quotient degree at <=5, only 2 selectors
     * can be used to select it.
     *
     * N.B.2 The q_c selector is used to store circuit-specific values in the RAM/ROM access gate
     *
     * @param evals transformed to `evals + C(in(X)...)*scaling_factor`
     * @param in an std::array containing the Totaly extended Univariate edges.
     * @param parameters contains beta, gamma, and public_input_delta, ....
     * @param scaling_factor optional term to scale the evaluation before adding to evals.
     */
    template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
    inline static void accumulate(ContainerOverSubrelations& accumulators,
                                  const AllEntities& in,
                                  const Parameters& params,
                                  const FF& scaling_factor)
    {
        BB_OP_COUNT_TIME_NAME("Auxiliary::accumulate");
        // declare the accumulator of the maximum length, in non-ZK Flavors, they are of the same length,
        // whereas in ZK Flavors, the accumulator corresponding to RAM consistency sub-relation 1 is the longest
        using Accumulator = typename std::tuple_element_t<3, ContainerOverSubrelations>;
        using View = typename Accumulator::View;
        // allows to re-use the values accumulated by accumulators of the sizes smaller or equal to
        // the size of Accumulator declared above
        using ShortView = typename std::tuple_element_t<0, ContainerOverSubrelations>::View;
        using ParameterView = GetParameterView<Parameters, View>;

        const auto& eta = ParameterView(params.eta);
        const auto& eta_two = ParameterView(params.eta_two);
        const auto& eta_three = ParameterView(params.eta_three);

        auto w_1 = View(in.w_l);
        auto w_2 = View(in.w_r);
        auto w_3 = View(in.w_o);
        auto w_4 = View(in.w_4);
        auto w_1_shift = View(in.w_l_shift);
        auto w_2_shift = View(in.w_r_shift);
        auto w_3_shift = View(in.w_o_shift);
        auto w_4_shift = View(in.w_4_shift);

        auto q_1 = View(in.q_l);
        auto q_2 = View(in.q_r);
        auto q_3 = View(in.q_o);
        auto q_4 = View(in.q_4);
        auto q_m = View(in.q_m);
        auto q_c = View(in.q_c);
        auto q_arith = View(in.q_arith);
        auto q_aux = View(in.q_aux);

        const FF LIMB_SIZE(uint256_t(1) << 68);
        const FF SUBLIMB_SHIFT(uint256_t(1) << 14);

        /**
         * Non native field arithmetic gate 2
         * deg 4
         *
         *             _                                                                               _
         *            /   _                   _                               _       14                \
         * q_2 . q_4 |   (w_1 . w_2) + (w_1 . w_2) + (w_1 . w_4 + w_2 . w_3 - w_3) . 2    - w_3 - w_4   |
         *            \_                                                                               _/
         *
         **/
        auto limb_subproduct = w_1 * w_2_shift + w_1_shift * w_2;
        auto non_native_field_gate_2 = (w_1 * w_4 + w_2 * w_3 - w_3_shift);
        non_native_field_gate_2 *= LIMB_SIZE;
        non_native_field_gate_2 -= w_4_shift;
        non_native_field_gate_2 += limb_subproduct;
        non_native_field_gate_2 *= q_4;

        limb_subproduct *= LIMB_SIZE;
        limb_subproduct += (w_1_shift * w_2_shift);
        auto non_native_field_gate_1 = limb_subproduct;
        non_native_field_gate_1 -= (w_3 + w_4);
        non_native_field_gate_1 *= q_3;

        auto non_native_field_gate_3 = limb_subproduct;
        non_native_field_gate_3 += w_4;
        non_native_field_gate_3 -= (w_3_shift + w_4_shift);
        non_native_field_gate_3 *= q_m;

        auto non_native_field_identity = non_native_field_gate_1 + non_native_field_gate_2 + non_native_field_gate_3;
        non_native_field_identity *= q_2;

        // ((((w2' * 2^14 + w1') * 2^14 + w3) * 2^14 + w2) * 2^14 + w1 - w4) * qm
        // deg 2
        auto limb_accumulator_1 = w_2_shift * SUBLIMB_SHIFT;
        limb_accumulator_1 += w_1_shift;
        limb_accumulator_1 *= SUBLIMB_SHIFT;
        limb_accumulator_1 += w_3;
        limb_accumulator_1 *= SUBLIMB_SHIFT;
        limb_accumulator_1 += w_2;
        limb_accumulator_1 *= SUBLIMB_SHIFT;
        limb_accumulator_1 += w_1;
        limb_accumulator_1 -= w_4;
        limb_accumulator_1 *= q_4;

        // ((((w3' * 2^14 + w2') * 2^14 + w1') * 2^14 + w4) * 2^14 + w3 - w4') * qm
        // deg 2
        auto limb_accumulator_2 = w_3_shift * SUBLIMB_SHIFT;
        limb_accumulator_2 += w_2_shift;
        limb_accumulator_2 *= SUBLIMB_SHIFT;
        limb_accumulator_2 += w_1_shift;
        limb_accumulator_2 *= SUBLIMB_SHIFT;
        limb_accumulator_2 += w_4;
        limb_accumulator_2 *= SUBLIMB_SHIFT;
        limb_accumulator_2 += w_3;
        limb_accumulator_2 -= w_4_shift;
        limb_accumulator_2 *= q_m;

        auto limb_accumulator_identity = limb_accumulator_1 + limb_accumulator_2;
        limb_accumulator_identity *= q_3; //  deg 3

        /**
         * MEMORY
         *
         * A RAM memory record contains a tuple of the following fields:
         *  * i: `index` of memory cell being accessed
         *  * t: `timestamp` of memory cell being accessed (used for RAM, set to 0 for ROM)
         *  * v: `value` of memory cell being accessed
         *  * a: `access` type of record. read: 0 = read, 1 = write
         *  * r: `record` of memory cell. record = access + index * eta + timestamp * η₂ + value * η₃
         *
         * A ROM memory record contains a tuple of the following fields:
         *  * i: `index` of memory cell being accessed
         *  * v: `value1` of memory cell being accessed (ROM tables can store up to 2 values per index)
         *  * v2:`value2` of memory cell being accessed (ROM tables can store up to 2 values per index)
         *  * r: `record` of memory cell. record = index * eta + value2 * η₂ + value1 * η₃
         *
         *  When performing a read/write access, the values of i, t, v, v2, a, r are stored in the following wires +
         * selectors, depending on whether the gate is a RAM read/write or a ROM read
         *
         *  | gate type | i  | v2/t  |  v | a  | r  |
         *  | --------- | -- | ----- | -- | -- | -- |
         *  | ROM       | w1 | w2    | w3 | -- | w4 |
         *  | RAM       | w1 | w2    | w3 | qc | w4 |
         *
         * (for accesses where `index` is a circuit constant, it is assumed the circuit will apply a copy constraint on
         * `w2` to fix its value)
         *
         **/

        /**
         * Memory Record Check
         * Partial degree: 1
         * Total degree: 2
         *
         * A ROM/ROM access gate can be evaluated with the identity:
         *
         * qc + w1 \eta + w2 η₂ + w3 η₃ - w4 = 0
         *
         * For ROM gates, qc = 0
         */
        auto memory_record_check = w_3 * eta_three;
        memory_record_check += w_2 * eta_two;
        memory_record_check += w_1 * eta;
        memory_record_check += q_c;
        auto partial_record_check = memory_record_check; // used in RAM consistency check; deg 1 or 2
        memory_record_check = memory_record_check - w_4;

        /**
         * ROM Consistency Check
         * Partial degree: 1
         * Total degree: 4
         *
         * For every ROM read, a set equivalence check is applied between the record witnesses, and a second set of
         * records that are sorted.
         *
         * We apply the following checks for the sorted records:
         *
         * 1. w1, w2, w3 correctly map to 'index', 'v1, 'v2' for a given record value at w4
         * 2. index values for adjacent records are monotonically increasing
         * 3. if, at gate i, index_i == index_{i + 1}, then value1_i == value1_{i + 1} and value2_i == value2_{i + 1}
         *
         */
        auto index_delta = w_1_shift - w_1;
        auto record_delta = w_4_shift - w_4;

        auto index_is_monotonically_increasing = index_delta.sqr() - index_delta; // deg 2

        auto adjacent_values_match_if_adjacent_indices_match = (-index_delta + FF(1)) * record_delta; // deg 2

        auto q_aux_by_scaling = q_aux * scaling_factor;
        auto q_one_by_two = q_1 * q_2;
        auto q_one_by_two_by_aux_by_scaling = q_one_by_two * q_aux_by_scaling;

        std::get<1>(accumulators) +=
            ShortView(adjacent_values_match_if_adjacent_indices_match * q_one_by_two_by_aux_by_scaling); // deg 5
        std::get<2>(accumulators) +=
            ShortView(index_is_monotonically_increasing * q_one_by_two_by_aux_by_scaling); // deg 5
        auto ROM_consistency_check_identity = memory_record_check * q_one_by_two;          // deg 3 or 4

        /**
         * RAM Consistency Check
         *
         * The 'access' type of the record is extracted with the expression `w_4 - partial_record_check`
         * (i.e. for an honest Prover `w1 * η + w2 * η₂ + w3 * η₃ - w4 = access`.
         * This is validated by requiring `access` to be boolean
         *
         * For two adjacent entries in the sorted list if _both_
         *  A) index values match
         *  B) adjacent access value is 0 (i.e. next gate is a READ)
         * then
         *  C) both values must match.
         * The gate boolean check is
         * (A && B) => C  === !(A && B) || C ===  !A || !B || C
         *
         * N.B. it is the responsibility of the circuit writer to ensure that every RAM cell is initialized
         * with a WRITE operation.
         */
        auto access_type = (w_4 - partial_record_check);             // will be 0 or 1 for honest Prover; deg 1 or 2
        auto access_check = access_type * access_type - access_type; // check value is 0 or 1; deg 2 or 4

        // TODO(https://github.com/AztecProtocol/barretenberg/issues/757): If we sorted in
        // reverse order we could re-use `partial_record_check`  1 -  (w3' * eta_three + w2' * eta_two + w1' *
        // eta) deg 1 or 2
        auto next_gate_access_type = w_3_shift * eta_three;
        next_gate_access_type += w_2_shift * eta_two;
        next_gate_access_type += w_1_shift * eta;
        next_gate_access_type = w_4_shift - next_gate_access_type;

        auto value_delta = w_3_shift - w_3;
        auto adjacent_values_match_if_adjacent_indices_match_and_next_access_is_a_read_operation =
            (-index_delta + FF(1)) * value_delta * (-next_gate_access_type + FF(1)); // deg 3 or 4

        // We can't apply the RAM consistency check identity on the final entry in the sorted list (the wires in the
        // next gate would make the identity fail).  We need to validate that its 'access type' bool is correct. Can't
        // do  with an arithmetic gate because of the  `eta` factors. We need to check that the *next* gate's access
        // type is  correct, to cover this edge case
        // deg 2 or 4
        auto next_gate_access_type_is_boolean = next_gate_access_type.sqr() - next_gate_access_type;

        auto q_arith_by_aux_and_scaling = q_arith * q_aux_by_scaling;
        // Putting it all together...
        std::get<3>(accumulators) +=
            adjacent_values_match_if_adjacent_indices_match_and_next_access_is_a_read_operation *
            q_arith_by_aux_and_scaling; // deg 5 or 6
        std::get<4>(accumulators) += ShortView(index_is_monotonically_increasing * q_arith_by_aux_and_scaling); // deg 4
        std::get<5>(accumulators) +=
            ShortView(next_gate_access_type_is_boolean * q_arith_by_aux_and_scaling); // deg 4 or 6

        auto RAM_consistency_check_identity = access_check * (q_arith); // deg 3 or 5

        /**
         * RAM Timestamp Consistency Check
         *
         * | w1 | w2 | w3 | w4 |
         * | index | timestamp | timestamp_check | -- |
         *
         * Let delta_index = index_{i + 1} - index_{i}
         *
         * Iff delta_index == 0, timestamp_check = timestamp_{i + 1} - timestamp_i
         * Else timestamp_check = 0
         */
        auto timestamp_delta = w_2_shift - w_2;
        auto RAM_timestamp_check_identity = (-index_delta + FF(1)) * timestamp_delta - w_3; // deg 3

        /**
         * The complete RAM/ROM memory identity
         * Partial degree:
         */
        auto memory_identity = ROM_consistency_check_identity;         // deg 3 or 4
        memory_identity += RAM_timestamp_check_identity * (q_4 * q_1); // deg 4
        memory_identity += memory_record_check * (q_m * q_1);          // deg 3 or 4
        memory_identity += RAM_consistency_check_identity;             // deg 3 or 5

        // (deg 3 or 5) + (deg 4) + (deg 3)
        auto auxiliary_identity = memory_identity + non_native_field_identity + limb_accumulator_identity;
        auxiliary_identity *= q_aux_by_scaling; // deg 5 or 6
        std::get<0>(accumulators) += ShortView(auxiliary_identity);
    };
};

template <typename FF> using AuxiliaryRelation = Relation<AuxiliaryRelationImpl<FF>>;
} // namespace bb
