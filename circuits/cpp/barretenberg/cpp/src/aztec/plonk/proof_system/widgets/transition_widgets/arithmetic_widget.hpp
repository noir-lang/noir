#pragma once

#include "./transition_widget.hpp"

namespace waffle {
namespace widget {

/**
 * @brief Core class implementing the arithmetic gate in Standard plonk
 *
 * @details ArithmethicKernel provides the logic that implements the standard arithmetic transition
 * q_m * w_1 * w_2 + q_1 * w_1 + q_2 * w_2 + q_3 * w_3 + q_c=0
 *
 * Uses only the alpha challenge
 *
 * @tparam Field The basic field in which the elements operates
 * @tparam Getters The class providing functions that access evaluations of polynomials at indices
 * @tparam PolyContainer Container for the polynomials or their simulation
 */
template <class Field, class Getters, typename PolyContainer> class ArithmeticKernel {
  public:
    static constexpr size_t num_independent_relations = 1;
    // We state the challenges required for linear/nonlinear terms computation
    static constexpr uint8_t quotient_required_challenges = CHALLENGE_BIT_ALPHA;
    // We state the challenges required for updating kate opening scalars
    static constexpr uint8_t update_required_challenges = CHALLENGE_BIT_ALPHA;

  private:
    // A structure with various challenges, even though only alpha is used here.
    typedef containers::challenge_array<Field, num_independent_relations> challenge_array;
    // Type for the linear terms of the transition
    typedef containers::coefficient_array<Field> coefficient_array;

  public:
    inline static std::set<PolynomialIndex> const& get_required_polynomial_ids()
    {
        static const std::set<PolynomialIndex> required_polynomial_ids = { PolynomialIndex::Q_1, PolynomialIndex::Q_2,
                                                                           PolynomialIndex::Q_3, PolynomialIndex::Q_M,
                                                                           PolynomialIndex::Q_C, PolynomialIndex::W_1,
                                                                           PolynomialIndex::W_2, PolynomialIndex::W_3 };
        return required_polynomial_ids;
    }

    /**
     * @brief Computes the linear terms.
     *
     * @details  Multiplies the values at the first and second wire, puts the product and all the wires into the linear
     * terms
     *
     * @param polynomials Polynomials from which the values of wires are obtained
     * @param linear_terms Container for results of computation
     * @param i Index at which the wire values are sampled.
     */
    inline static void compute_linear_terms(PolyContainer& polynomials,
                                            const challenge_array&,
                                            coefficient_array& linear_terms,
                                            const size_t i = 0)
    {
        const Field& w_1 =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::W_1>(polynomials, i);
        const Field& w_2 =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::W_2>(polynomials, i);
        const Field& w_3 =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::W_3>(polynomials, i);

        linear_terms[0] = w_1 * w_2;
        linear_terms[1] = w_1;
        linear_terms[2] = w_2;
        linear_terms[3] = w_3;
    }

    /**
     * @brief Not being used in arithmetic_widget because there are none
     *
     */
    inline static void compute_non_linear_terms(PolyContainer&, const challenge_array&, Field&, const size_t = 0) {}

    /**
     * @brief Scale and sum the linear terms for the final equation.
     *
     * @details Multiplies the linear terms by selector values and scale the whole sum by alpha before returning
     *
     * @param polynomials Container with polynomials or their simulation
     * @param challenges A structure with various challenges
     * @param linear_terms Precomuputed linear terms to be scaled and summed
     * @param i The index at which selector/witness values are sampled
     * @return Field Scaled sum of values
     */
    inline static Field sum_linear_terms(PolyContainer& polynomials,
                                         const challenge_array& challenges,
                                         coefficient_array& linear_terms,
                                         const size_t i = 0)
    {
        const Field& alpha = challenges.alpha_powers[0];
        const Field& q_1 =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_1>(polynomials, i);
        const Field& q_2 =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_2>(polynomials, i);
        const Field& q_3 =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_3>(polynomials, i);
        const Field& q_m =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_M>(polynomials, i);
        const Field& q_c =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_C>(polynomials, i);

        Field result = linear_terms[0] * q_m;
        result += (linear_terms[1] * q_1);
        result += (linear_terms[2] * q_2);
        result += (linear_terms[3] * q_3);
        result += q_c;
        result *= alpha;
        return result;
    }

    /**
     * @brief Compute the scaled values of openings
     *
     * @param linear_terms The original computed linear terms of the product and wires
     * @param scalars A map where we put the values
     * @param challenges Challenges where we get the alpha
     */
    inline static void update_kate_opening_scalars(coefficient_array& linear_terms,
                                                   std::map<std::string, Field>& scalars,
                                                   const challenge_array& challenges)
    {
        const Field& alpha = challenges.alpha_powers[0];
        scalars["Q_M"] += linear_terms[0] * alpha;
        scalars["Q_1"] += linear_terms[1] * alpha;
        scalars["Q_2"] += linear_terms[2] * alpha;
        scalars["Q_3"] += linear_terms[3] * alpha;
        scalars["Q_C"] += alpha;
    }
};

} // namespace widget

/**
 * @brief Standard plonk arithmetic widget for the prover. Provides standard plonk gate transition
 *
 * @details ArithmethicKernel provides the logic that implements the standard arithmetic transition
 * q_m * w_1 * w_2 + q_1 * w_1 + q_2 * w_2 + q_3 * w_3 + q_c=0
 * @tparam Settings
 */
template <typename Settings>
using ProverArithmeticWidget = widget::TransitionWidget<barretenberg::fr, Settings, widget::ArithmeticKernel>;

/**
 * @brief Standard plonk arithmetic widget for the verifier. Provides standard plonk gate transition
 *
 * @details ArithmethicKernel provides the logic that implements the standard arithmetic transition
 * q_m * w_1 * w_2 + q_1 * w_1 + q_2 * w_2 + q_3 * w_3 + q_c=0
 * @tparam Settings
 */
template <typename Field, typename Group, typename Transcript, typename Settings>
using VerifierArithmeticWidget = widget::GenericVerifierWidget<Field, Transcript, Settings, widget::ArithmeticKernel>;

} // namespace waffle