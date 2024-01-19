#pragma once

#include "./transition_widget.hpp"

namespace bb::plonk {
namespace widget {

/**
 * @brief Plookup Auxiliary Widget
 *
 * @details Evaluates polynomial identities associated with the following UltraPlonk custom gates:
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
 * N.B. The RAM consistency check identity is degree 3. To keep the overall quotient degree at <=5, only 2 selectors can
 * be used to select it.
 *
 * N.B.2 The q_c selector is used to store circuit-specific values in the RAM/ROM access gate
 *
 * @tparam Field
 * @tparam Getters
 * @tparam PolyContainer
 */
template <class Field, class Getters, typename PolyContainer> class PlookupAuxiliaryKernel {
  public:
    static constexpr bool use_quotient_mid = false;
    static constexpr size_t num_independent_relations = 4;
    // We state the challenges required for linear/nonlinear terms computation
    static constexpr uint8_t quotient_required_challenges = CHALLENGE_BIT_ALPHA;
    // We state the challenges required for updating kate opening scalars
    static constexpr uint8_t update_required_challenges = CHALLENGE_BIT_ALPHA;

  private:
    typedef containers::challenge_array<Field, num_independent_relations> challenge_array;

  public:
    inline static std::set<PolynomialIndex> const& get_required_polynomial_ids()
    {
        static const std::set<PolynomialIndex> required_polynomial_ids = {
            PolynomialIndex::Q_1, PolynomialIndex::Q_2, PolynomialIndex::Q_3,          PolynomialIndex::Q_4,
            PolynomialIndex::Q_M, PolynomialIndex::Q_C, PolynomialIndex::Q_ARITHMETIC, PolynomialIndex::Q_AUX,
            PolynomialIndex::W_1, PolynomialIndex::W_2, PolynomialIndex::W_3,          PolynomialIndex::W_4
        };
        return required_polynomial_ids;
    }

    inline static void accumulate_contribution(PolyContainer& polynomials,
                                               const challenge_array& challenges,
                                               Field& quotient,
                                               const size_t i = 0)
    {
        constexpr bb::fr LIMB_SIZE(uint256_t(1) << 68);
        constexpr bb::fr SUBLIMB_SHIFT(uint256_t(1) << 14);

        const Field& w_1 =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::W_1>(polynomials, i);
        const Field& w_2 =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::W_2>(polynomials, i);
        const Field& w_3 =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::W_3>(polynomials, i);
        const Field& w_4 =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::W_4>(polynomials, i);
        const Field& w_1_omega =
            Getters::template get_value<EvaluationType::SHIFTED, PolynomialIndex::W_1>(polynomials, i);
        const Field& w_2_omega =
            Getters::template get_value<EvaluationType::SHIFTED, PolynomialIndex::W_2>(polynomials, i);
        const Field& w_3_omega =
            Getters::template get_value<EvaluationType::SHIFTED, PolynomialIndex::W_3>(polynomials, i);
        const Field& w_4_omega =
            Getters::template get_value<EvaluationType::SHIFTED, PolynomialIndex::W_4>(polynomials, i);

        const Field& alpha_base = challenges.alpha_powers[0];
        const Field& alpha = challenges.elements[ChallengeIndex::ALPHA];
        const Field& eta = challenges.elements[ChallengeIndex::ETA];

        const Field& q_aux =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_AUX>(polynomials, i);
        const Field& q_1 =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_1>(polynomials, i);
        const Field& q_2 =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_2>(polynomials, i);
        const Field& q_3 =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_3>(polynomials, i);
        const Field& q_4 =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_4>(polynomials, i);
        const Field& q_m =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_M>(polynomials, i);
        const Field& q_c =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_C>(polynomials, i);
        const Field& q_arith =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_ARITHMETIC>(polynomials, i);

        /**
         * Non native field arithmetic gate 2
         *
         *             _                                                                               _
         *            /   _                   _                               _       14                \
         * q_2 . q_4 |   (w_1 . w_2) + (w_1 . w_2) + (w_1 . w_4 + w_2 . w_3 - w_3) . 2    - w_3 - w_4   |
         *            \_                                                                               _/
         *
         **/
        Field limb_subproduct = w_1 * w_2_omega + w_1_omega * w_2;
        Field non_native_field_gate_2 = (w_1 * w_4 + w_2 * w_3 - w_3_omega);
        non_native_field_gate_2 *= LIMB_SIZE;
        non_native_field_gate_2 -= w_4_omega;
        non_native_field_gate_2 += limb_subproduct;
        non_native_field_gate_2 *= q_4;

        limb_subproduct *= LIMB_SIZE;
        limb_subproduct += (w_1_omega * w_2_omega);
        Field non_native_field_gate_1 = limb_subproduct;
        non_native_field_gate_1 -= (w_3 + w_4);
        non_native_field_gate_1 *= q_3;

        Field non_native_field_gate_3 = limb_subproduct;
        non_native_field_gate_3 += w_4;
        non_native_field_gate_3 -= (w_3_omega + w_4_omega);
        non_native_field_gate_3 *= q_m;

        Field non_native_field_identity = non_native_field_gate_1 + non_native_field_gate_2 + non_native_field_gate_3;
        non_native_field_identity *= q_2;

        Field limb_accumulator_1 = w_2_omega;
        limb_accumulator_1 *= SUBLIMB_SHIFT;
        limb_accumulator_1 += w_1_omega;
        limb_accumulator_1 *= SUBLIMB_SHIFT;
        limb_accumulator_1 += w_3;
        limb_accumulator_1 *= SUBLIMB_SHIFT;
        limb_accumulator_1 += w_2;
        limb_accumulator_1 *= SUBLIMB_SHIFT;
        limb_accumulator_1 += w_1;
        limb_accumulator_1 -= w_4;
        limb_accumulator_1 *= q_4;

        Field limb_accumulator_2 = w_3_omega;
        limb_accumulator_2 *= SUBLIMB_SHIFT;
        limb_accumulator_2 += w_2_omega;
        limb_accumulator_2 *= SUBLIMB_SHIFT;
        limb_accumulator_2 += w_1_omega;
        limb_accumulator_2 *= SUBLIMB_SHIFT;
        limb_accumulator_2 += w_4;
        limb_accumulator_2 *= SUBLIMB_SHIFT;
        limb_accumulator_2 += w_3;
        limb_accumulator_2 -= w_4_omega;
        limb_accumulator_2 *= q_m;

        Field limb_accumulator_identity = limb_accumulator_1 + limb_accumulator_2;
        limb_accumulator_identity *= q_3;

        /**
         * MEMORY
         *
         * A RAM memory record contains a tuple of the following fields:
         *  * i: `index` of memory cell being accessed
         *  * t: `timestamp` of memory cell being accessed (used for RAM, set to 0 for ROM)
         *  * v: `value` of memory cell being accessed
         *  * a: `access` type of record. read: 0 = read, 1 = write
         *  * r: `record` of memory cell. record = access + index * eta + timestamp * eta^2 + value * eta^3
         *
         * A ROM memory record contains a tuple of the following fields:
         *  * i: `index` of memory cell being accessed
         *  * v: `value1` of memory cell being accessed (ROM tables can store up to 2 values per index)
         *  * v2:`value2` of memory cell being accessed (ROM tables can store up to 2 values per index)
         *  * r: `record` of memory cell. record = index * eta + value2 * eta^2 + value1 * eta^3
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
         *
         * A ROM/ROM access gate can be evaluated with the identity:
         *
         * qc + w1 \eta + w2 \eta^2 + w3 \eta^3 - w4 = 0
         *
         * For ROM gates, qc = 0
         */
        Field memory_record_check = w_3;
        memory_record_check *= eta;
        memory_record_check += w_2;
        memory_record_check *= eta;
        memory_record_check += w_1;
        memory_record_check *= eta;
        memory_record_check += q_c;
        Field partial_record_check = memory_record_check; // used in RAM consistency check
        memory_record_check = memory_record_check - w_4;

        /**
         * ROM Consistency Check
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
        Field index_delta = w_1_omega - w_1;
        Field record_delta = w_4_omega - w_4;

        Field index_is_monotonically_increasing = index_delta.sqr() - index_delta;

        Field adjacent_values_match_if_adjacent_indices_match = (Field(1) - index_delta) * record_delta;

        Field ROM_consistency_check_identity = adjacent_values_match_if_adjacent_indices_match;
        ROM_consistency_check_identity *= alpha;
        ROM_consistency_check_identity += index_is_monotonically_increasing;
        ROM_consistency_check_identity *= alpha;
        ROM_consistency_check_identity += memory_record_check;

        /**
         * RAM Consistency Check
         *
         * The 'access' type of the record is extracted with the expression `w_4 - partial_record_check`
         * (i.e. for an honest Prover `w1 * eta + w2 * eta^2 + w3 * eta^3 - w4 = access`.
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
        Field access_type = (w_4 - partial_record_check);     // will be 0 or 1 for honest Prover
        Field access_check = access_type.sqr() - access_type; // check value is 0 or 1

        // TODO: oof nasty compute here. If we sorted in reverse order we could re-use `partial_record_check`
        Field next_gate_access_type = w_3_omega;
        next_gate_access_type *= eta;
        next_gate_access_type += w_2_omega;
        next_gate_access_type *= eta;
        next_gate_access_type += w_1_omega;
        next_gate_access_type *= eta;
        next_gate_access_type = w_4_omega - next_gate_access_type;

        Field value_delta = w_3_omega - w_3;
        Field adjacent_values_match_if_adjacent_indices_match_and_next_access_is_a_read_operation =
            (Field(1) - index_delta) * value_delta * (Field(1) - next_gate_access_type);

        // We can't apply the RAM consistency check identity on the final entry in the sorted list (the wires in the
        // next gate would make the identity fail).
        // We need to validate that its 'access type' bool is correct. Can't do
        // with an arithmetic gate because of the `eta` factors. We need to check that the *next* gate's access type is
        // correct, to cover this edge case
        Field next_gate_access_type_is_boolean = next_gate_access_type.sqr() - next_gate_access_type;

        // Putting it all together...
        Field RAM_consistency_check_identity =
            adjacent_values_match_if_adjacent_indices_match_and_next_access_is_a_read_operation;
        RAM_consistency_check_identity *= alpha;
        RAM_consistency_check_identity += index_is_monotonically_increasing;
        RAM_consistency_check_identity *= alpha;
        RAM_consistency_check_identity += next_gate_access_type_is_boolean;
        RAM_consistency_check_identity *= alpha;
        RAM_consistency_check_identity += access_check;

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
        Field timestamp_delta = w_2_omega - w_2;
        Field RAM_timestamp_check_identity = (Field(1) - index_delta) * timestamp_delta - w_3;

        /**
         * The complete RAM/ROM memory identity
         *
         */
        Field memory_identity = ROM_consistency_check_identity * q_2;
        memory_identity += RAM_timestamp_check_identity * q_4;
        memory_identity += memory_record_check * q_m;
        memory_identity *= q_1;
        memory_identity += (RAM_consistency_check_identity * q_arith);

        Field auxiliary_identity = memory_identity + non_native_field_identity + limb_accumulator_identity;
        auxiliary_identity *= q_aux;
        auxiliary_identity *= alpha_base;

        quotient += (auxiliary_identity);
    }
};

} // namespace widget

template <typename Settings>
using ProverPlookupAuxiliaryWidget = widget::TransitionWidget<bb::fr, Settings, widget::PlookupAuxiliaryKernel>;

template <typename Field, typename Group, typename Transcript, typename Settings>
using VerifierPlookupAuxiliaryWidget =
    widget::GenericVerifierWidget<Field, Transcript, Settings, widget::PlookupAuxiliaryKernel>;

} // namespace bb::plonk