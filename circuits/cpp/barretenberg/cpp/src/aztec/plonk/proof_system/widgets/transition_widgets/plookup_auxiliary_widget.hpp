#pragma once

#include "./transition_widget.hpp"

namespace waffle {
namespace widget {

/**
 * Plookup Auxiliary Widget
 *
 * Evaluates polynomial identities associated with the following UltraPlonk custom gates:
 *
 * RAM/ROM read-write consistency check
 * RAM timestamp difference consistency check
 * RAM/ROM index difference consistency check
 * Bigfield product evaluation (3 in total)
 * Bigfield limb accumulation (2 in total)
 *
 **/
template <class Field, class Getters, typename PolyContainer> class PlookupAuxiliaryKernel {
  public:
    static constexpr bool use_quotient_mid = false;
    static constexpr size_t num_independent_relations = 3;
    // We state the challenges required for linear/nonlinear terms computation
    static constexpr uint8_t quotient_required_challenges = CHALLENGE_BIT_ALPHA;
    // We state the challenges required for updating kate opening scalars
    static constexpr uint8_t update_required_challenges = CHALLENGE_BIT_ALPHA;

  private:
    typedef containers::challenge_array<Field, num_independent_relations> challenge_array;
    typedef containers::coefficient_array<Field> coefficient_array;

  public:
    inline static std::set<PolynomialIndex> const& get_required_polynomial_ids()
    {
        static const std::set<PolynomialIndex> required_polynomial_ids = { PolynomialIndex::Q_1, PolynomialIndex::Q_2,
                                                                           PolynomialIndex::Q_3, PolynomialIndex::Q_4,
                                                                           PolynomialIndex::Q_M, PolynomialIndex::Q_AUX,
                                                                           PolynomialIndex::W_1, PolynomialIndex::W_2,
                                                                           PolynomialIndex::W_3, PolynomialIndex::W_4 };
        return required_polynomial_ids;
    }

    inline static void compute_linear_terms(PolyContainer&,
                                            const challenge_array&,
                                            coefficient_array&,
                                            const size_t = 0)
    {}

    inline static void compute_non_linear_terms(PolyContainer& polynomials,
                                                const challenge_array& challenges,
                                                Field& quotient,
                                                const size_t i = 0)
    {
        constexpr barretenberg::fr LIMB_SIZE(uint256_t(1) << 68);
        constexpr barretenberg::fr SUBLIMB_SHIFT(uint256_t(1) << 14);

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

        limb_subproduct = w_1_omega - w_1;

        /**
         * MEMORY
         *
         * A memory record contains a tuple of the following fields:
         *  * i: `index` of memory cell being accessed
         *  * t: `tiemstamp` of memory cell being accessed (used for RAM, set to 0 for ROM)
         *  * v: `value` of memory cell being accessed
         *  * r: `record` of memory cell. record = index + timestamp * eta + value * eta^2
         *
         * A record gate is structured such that each of the above 4 fields maps to wires 1, 2, 3, 4
         **/

        /**
         * RAM SORTED LIST CHECK
         *
         * Validate the following at gate `j`:
         *
         *  1. (i_{j+1} - i_j)^2 - (i_{j+1} - i_j) = 0 (index increases by 0 or 1)
         *  2. (r = i + t * eta + v * eta^2)
         *
         * Used for sorted RAM records iff gate `j` is a RAM read and gate `j + 1` is a RAM write
         **/
        Field memory_sorted_list_check = limb_subproduct.sqr() - limb_subproduct;

        /**
         * ROM CONSISTENT SORTED LIST CHECK
         *
         * Validate the following at gate `j`:
         *
         *  1. (i_{j+1} - i_j)^2 - (i_{j+1} - i_j) = 0 (index increases by 0 or 1)
         *  2. (r = i + t * eta + v * eta^2)
         *  3. (1 - (i_{j+1} - i_j)) * (r_{j+1} - r_j) (if index does not change, neither does
         *value)
         *
         * Used for sorted ROM records
         **/
        // TODO: MAKE this work for ram records iff gate `j` is a RAM write and gate `j + 1` is a
        // *RAM read
        // for RAM we want to compare across the value field (column 3)
        // but for ROM, if we compare across the record field (column 4) we can lookup 2 ROM values per gate
        Field memory_consistent_sorted_list_check = Field(1) - limb_subproduct; // 1 - (w_1_omega - w_1)
        // Field memory_consistent_sorted_list_RAM_check = memory_consistent_sorted_list_check * (w_3_omega - w_3);
        memory_consistent_sorted_list_check *= (w_4_omega - w_4); // (1 - (w_1_omega - w_1)) * (w_4_omega - w_4)

        /**
         * RAM TIMESTAMP CHECK
         *
         * Validate the following at gate `j`:
         *
         *  1. \delta = (1 - ((i_{j+1} - i_j)^2 - (i_{j+1} - i_j))(t_{j+1} - t_j)
         *
         * i.e. If index does not change, `\delta` = timestamp difference, eles `\delta` = 0
         *
         * Used for RAM records to validate consistency between read/writes into the same cell.
         * Timestamp check is performed in a separate gate to the checks against the sorted list (with copy constraints
         *to map between the two gates)
         **/
        Field memory_timestamp_check = memory_consistent_sorted_list_check - w_2;

        Field memory_record_check = w_3;
        memory_record_check *= eta;
        memory_record_check += w_2;
        memory_record_check *= eta;
        memory_record_check += w_1;
        memory_record_check -= w_4;

        memory_sorted_list_check *= alpha;
        memory_sorted_list_check += memory_record_check;

        memory_consistent_sorted_list_check += (memory_sorted_list_check * alpha);

        Field memory_identity = memory_consistent_sorted_list_check * q_2;
        memory_identity += memory_sorted_list_check * q_3;
        memory_identity += memory_timestamp_check * q_4;
        memory_identity += memory_record_check * q_m;
        memory_identity *= q_1;

        Field auxiliary_identity = memory_identity + non_native_field_identity + limb_accumulator_identity;
        auxiliary_identity *= q_aux;
        auxiliary_identity *= alpha_base;

        quotient += (auxiliary_identity);
    }

    inline static Field sum_linear_terms(PolyContainer&, const challenge_array&, coefficient_array&, const size_t = 0)
    {
        return Field(0);
    }

    inline static void update_kate_opening_scalars(coefficient_array&,
                                                   std::map<std::string, Field>&,
                                                   const challenge_array&)
    {}
};

} // namespace widget

template <typename Settings>
using ProverPlookupAuxiliaryWidget =
    widget::TransitionWidget<barretenberg::fr, Settings, widget::PlookupAuxiliaryKernel>;

template <typename Field, typename Group, typename Transcript, typename Settings>
using VerifierPlookupAuxiliaryWidget =
    widget::GenericVerifierWidget<Field, Transcript, Settings, widget::PlookupAuxiliaryKernel>;

} // namespace waffle