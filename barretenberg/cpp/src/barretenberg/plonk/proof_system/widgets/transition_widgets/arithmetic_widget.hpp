#pragma once

#include "./transition_widget.hpp"

namespace bb::plonk {
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
    inline static void accumulate_contribution(PolyContainer& polynomials,
                                               const challenge_array& challenges,
                                               Field& quotient,
                                               const size_t i = 0)
    {
        const Field& w_1 =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::W_1>(polynomials, i);
        const Field& w_2 =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::W_2>(polynomials, i);
        const Field& w_3 =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::W_3>(polynomials, i);

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

        Field result = (w_1 * w_2) * q_m;
        result += w_1 * q_1;
        result += w_2 * q_2;
        result += w_3 * q_3;
        result += q_c;
        result *= alpha;
        quotient += result;
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
using ProverArithmeticWidget = widget::TransitionWidget<bb::fr, Settings, widget::ArithmeticKernel>;

/**
 * @brief Standard plonk arithmetic widget for the verifier. Provides standard plonk gate transition
 *
 * @details ArithmethicKernel provides the logic that implements the standard arithmetic transition
 * q_m * w_1 * w_2 + q_1 * w_1 + q_2 * w_2 + q_3 * w_3 + q_c=0
 * @tparam Settings
 */
template <typename Field, typename Group, typename Transcript, typename Settings>
using VerifierArithmeticWidget = widget::GenericVerifierWidget<Field, Transcript, Settings, widget::ArithmeticKernel>;

} // namespace bb::plonk