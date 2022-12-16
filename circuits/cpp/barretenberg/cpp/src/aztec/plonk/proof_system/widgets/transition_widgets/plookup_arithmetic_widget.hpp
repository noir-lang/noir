#pragma once

#include "./transition_widget.hpp"

namespace waffle {
namespace widget {

/**
 * @brief Core class implementing the arithmetic gate in Turbo plonk
 *
 * @details ArithmethicKernel provides the logic that can implement one of several transitions. The whole formula
 * without alpha scaling is:
 *
 * q_arith * ( ( (-1/2) * (q_arith - 3) * q_m * w_1 * w_2 + q_1 * w_1 + q_2 * w_2 + q_3 * w_3 + q_4 * w_4 + q_c ) +
 * (q_arith - 1)*( α² * (q_arith - 2) * (w_1 + w_4 - w_1_omega + q_m) + w_4_omega) ) = 0
 *
 * This formula results in several cases depending on q_arith:
 * 1. q_arith == 0: Arithmetic gate is completely disabled
 *
 * 2. q_arith == 1: Everything in the minigate on the right is disabled. The equation is just a standard plonk equation
 * with extra wires: q_m * w_1 * w_2 + q_1 * w_1 + q_2 * w_2 + q_3 * w_3 + q_4 * w_4 + q_c = 0
 *
 * 3. q_arith == 2: The (w_1 + w_4 - ...) term is disabled. THe equation is:
 * (1/2) * q_m * w_1 * w_2 + q_1 * w_1 + q_2 * w_2 + q_3 * w_3 + q_4 * w_4 + q_c + w_4_omega = 0
 * It allows defining w_4 at next index (w_4_omega) in terms of current wire values
 *
 * 4. q_arith == 3: The product of w_1 and w_2 is disabled, but a mini addition gate is enabled. α² allows us to split
 * the equation into two:
 *
 * q_1 * w_1 + q_2 * w_2 + q_3 * w_3 + q_4 * w_4 + q_c + 2 * w_4_omega = 0
 *
 * w_1 + w_4 - w_1_omega + q_m = 0  (we are reusing q_m here)
 *
 * 5. q_arith > 3: The product of w_1 and w_2 is scaled by (q_arith - 3), while the w_4_omega term is scaled by (q_arith
 * - 1). The equation can be split into two:
 *
 * (q_arith - 3)* q_m * w_1 * w_ 2 + q_1 * w_1 + q_2 * w_2 + q_3 * w_3 + q_4 * w_4 + q_c + (q_arith - 1) * w_4_omega = 0
 *
 * w_1 + w_4 - w_1_omega + q_m = 0
 *
 * The problem that q_m is used both in both equations can be dealt with by appropriately changing selector values at
 * the next gate. Then we can treat (q_arith - 1) as a simulated q_6 selector and scale q_m to handle (q_arith - 3) at
 * product.
 *
 * Uses only the alpha challenge
 *
 * @tparam Field The basic field in which the elements operates
 * @tparam Getters The class providing functions that access evaluations of polynomials at indices
 * @tparam PolyContainer Container for the polynomials or their simulation
 */
template <class Field, class Getters, typename PolyContainer> class PlookupArithmeticKernel {
  public:
    static constexpr bool use_quotient_mid = false;
    static constexpr size_t num_independent_relations = 2;
    // We state the challenges required for linear/nonlinear terms computation
    static constexpr uint8_t quotient_required_challenges = CHALLENGE_BIT_ALPHA;
    // We state the challenges required for updating kate opening scalars
    static constexpr uint8_t update_required_challenges = CHALLENGE_BIT_ALPHA;

  private:
    // A structure with various challenges, even though only alpha is used here.
    typedef containers::challenge_array<Field, num_independent_relations> challenge_array;
    // Type for the linear terms of the transition (not actually used here)
    typedef containers::coefficient_array<Field> coefficient_array;

  public:
    inline static std::set<PolynomialIndex> const& get_required_polynomial_ids()
    {
        static const std::set<PolynomialIndex> required_polynomial_ids = {
            PolynomialIndex::Q_1, PolynomialIndex::Q_2, PolynomialIndex::Q_3, PolynomialIndex::Q_4,
            PolynomialIndex::Q_5, PolynomialIndex::Q_M, PolynomialIndex::Q_C, PolynomialIndex::Q_ARITHMETIC,
            PolynomialIndex::W_1, PolynomialIndex::W_2, PolynomialIndex::W_3, PolynomialIndex::W_4
        };
        return required_polynomial_ids;
    }

    /**
     * @brief Stub for computing linear terms. Not used in plookup artihmetic gate
     *
     */
    inline static void compute_linear_terms(PolyContainer&,
                                            const challenge_array&,
                                            coefficient_array&,
                                            const size_t = 0)
    {}
    /**
     * @brief Computes the full identity for the arithmetic gate in plookup to be added to the quotient. All the logic
     * is explained in class description
     *
     * @param polynomials Container for polynomials or their simpulation
     * @param challenges Challenge array (we only need powers of alpha here)
     * @param quotient Quotient reference to add the result to
     * @param i Gate index
     */
    inline static void compute_non_linear_terms(PolyContainer& polynomials,
                                                const challenge_array& challenges,
                                                Field& quotient,
                                                const size_t i = 0)
    {
        // For subgroup element i, this term evaluates to W_4(i \omega) * 2 iff Q_ARITH(i \omega) = 2
        const Field& q_arith =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_ARITHMETIC>(polynomials, i);
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
        const Field& w_4_omega =
            Getters::template get_value<EvaluationType::SHIFTED, PolynomialIndex::W_4>(polynomials, i);
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

        const Field& alpha_base = challenges.alpha_powers[0];
        const Field& alpha = challenges.alpha_powers[1];

        // basic arithmetic gate identity
        // (w_1 . w_2 . q_m) + (w_1 . q_1) + (w_2 . q_2) + (w_3 . q_3) + (w_4 . q_4) + q_c = 0
        // q_m is turned off if q_arith == 3
        Field arithmetic_gate_identity = w_2;
        arithmetic_gate_identity *= q_m;
        arithmetic_gate_identity *= (q_arith - 3);

        // TODO: if we multiply all q_m values by `-1/2` we can remove the need for this extra multiplication
        if constexpr (std::is_same<barretenberg::fr, Field>::value) {
            static constexpr barretenberg::fr neg_half = barretenberg::fr(-2).invert();
            arithmetic_gate_identity *= neg_half;
        } else {
            static const Field neg_half = Field(-2).invert();
            arithmetic_gate_identity *= neg_half;
        }
        arithmetic_gate_identity += q_1;
        arithmetic_gate_identity *= w_1;
        arithmetic_gate_identity += (w_2 * q_2);
        arithmetic_gate_identity += (w_3 * q_3);
        arithmetic_gate_identity += (w_4 * q_4);
        arithmetic_gate_identity += q_c;

        // if q_arith == 2 OR q_arith == 3 we add the 4th wire of the NEXT gate into the arithmetic identity
        // N.B. if q_arith > 2, this wire value will be scaled by (q_arith - 1) relative to the other gate wires!
        const Field next_wire_in_arithmetic_gate_identity = w_4_omega;

        // if q_arith == 3 we evaluate an additional mini addition gate (on top of the regular one), where:
        //   w_1 + w_4 - w_1_omega + q_m = 0
        // we use this gate to save an addition gate when adding or subtracting non-native field elements
        Field extra_small_addition_gate_identity = (w_1 + w_4 - w_1_omega + q_m);
        extra_small_addition_gate_identity *= alpha;
        extra_small_addition_gate_identity *= (q_arith - 2);

        Field identity = extra_small_addition_gate_identity + next_wire_in_arithmetic_gate_identity;
        identity *= (q_arith - 1);
        identity += arithmetic_gate_identity;
        identity *= q_arith;
        identity *= alpha_base;

        quotient += identity;
    }

    inline static Field sum_linear_terms(PolyContainer&, const challenge_array&, coefficient_array&, const size_t = 0)
    {
        return Field(0);
    }

    /**
     * @brief Stub for updating opening scalars, since not using linear terms
     *
     */
    inline static void update_kate_opening_scalars(coefficient_array&,
                                                   std::map<std::string, Field>&,
                                                   const challenge_array&)
    {}
};

} // namespace widget

/**
 * @brief Ultra plonk arithmetic widget for the prover. It's quite complex, so for details better look at the kernel
 * class description
 * @tparam Settings
 */
template <typename Settings>
using ProverPlookupArithmeticWidget =
    widget::TransitionWidget<barretenberg::fr, Settings, widget::PlookupArithmeticKernel>;

/**
 * @brief Ultra plonk arithmetic widget for the verifier. It's quite complex, so for details better look at the kernel
 * class description
 * @tparam Settings
 */
template <typename Field, typename Group, typename Transcript, typename Settings>
using VerifierPlookupArithmeticWidget =
    widget::GenericVerifierWidget<Field, Transcript, Settings, widget::PlookupArithmeticKernel>;

} // namespace waffle