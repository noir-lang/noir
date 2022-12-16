#pragma once

#include "./transition_widget.hpp"

namespace waffle {
namespace widget {

/**
 * @brief Core class implementing elliptic curve point addition. It is enhanced to handle the case where one of the
 * points is automatically scaled by the endomorphism constant β or negated
 *
 *
 * @details The basic equation for the elliptic curve in short weierstrass form is y^2 == x^3 + a * x + b.
 *
 * The addition formulas are:
 *    λ = (y_2 - y_1) / (x_2 - x_1)
 *    x_3 = λ^2 - x_2 - x_1 = (y_2 - y_1)^2 / (x_2 - x_1)^2 - x_2 - x_1 = ((y_2 - y_1)^2 - (x_2 - x_1) * (x_2^2 -
 * x_1^2)) / (x_2 - x_1)^2
 *
 * If we assume that the points being added are distinct and not invereses of each other (so their x coordinates
 * differ), then we can rephrase this equality:
 *    x_3 * (x_2 - x_1)^2 = ((y_2 - y_1)^2 - (x_2 - x_1) * (x_2^2 - x_1^2))
 * Let's say we want to apply the endomorphism to the (x_2, y_2) point at the same time and maybe change the sign of
 * y_2:
 *
 *    (x_2, y_2) = (β * x_2', sign * y_2')
 *    x_3 * (β * x_2' - x_1)^2 = ((sign * y_2' - y_1)^2 - (β * x_2' - x_1) * ((β * x_2')^2 - x_1^2))
 *
 * Let's open the brackets and group the terms by β, β^2, sign:
 *
 *  x_2'^2 * x_3 * β^2 - 2 * β * x_1 * x_2' * x_3 - x_1^2 * x_3 = sign^2 * y_2'^2 - 2 * sign * y_1 * y_2  + y_1^2 - β^3
 * * x_2'^3 + β * x_1^2 * x_2' + β^2 * x_1 * x_2'^2 - x_1^3
 *
 *  β^3 = 1
 *  sign^2 = 1 (at least we always expect sign to be set to 1 or -1)
 *
 *  sign * (-2 * y_1 * y_2) + β * (2 * x_1 * x_2' * x_3 +x_1^2 * x_2') + β^2 * (x_1 * x_2'^2 - x_2'^2 * x_3) + (x_1^2 *
 * x_3 + y_2'^2 + y_1^2 - x_2'^3 - x_1^3) = 0
 *  This is the equation computed in x_identity and scaled by α
 *
 *  Now let's deal with the y equation:
 *    y_3 = λ * (x_3 - x_1) + y_1 = (y_2 - y_1) * (x_3 - x_1) / (x_2 - x_1) + y_1 = ((y_2 - y_1) * (x_3 - x_1) + y_1 *
 * (x_2 - x_1)) / (x_2 - x_1)
 *
 *    (x_2 - x_1) * y_3 = (y_2 - y_1) * (x_3 - x_1) + y_1 * (x_2 - x_1)
 *
 * Let's substitute  (x_2, y_2) = (β * x_2', sign * y_2'):
 *
 *    β * x_2' * y_3 - x_1 * y_3 - sign * y_2' * x_3  + y_1 * x_3 + sign * y_2' * x_1 - y_1 * x_1 - β * y_1 * x_2' + x_1
 * * y_1 = 0
 *
 * Let's group:
 *
 *    sign * (-y_2' * x_3 + y_2' * x_1) + β * (x_2' * x_3 + y_1 * x_2') + (-x_1 * y_3 + y_1 * x_3 - x_1 * y_1 +
 * x_1 * y_1) = 0
 *
 *
 *
 * @tparam Field Field being used for elements
 * @tparam Getters  A class that implements retrieval methods for PolyContainer
 * @tparam PolyContainer Container with polynomials or their simulation
 */
template <class Field, class Getters, typename PolyContainer> class EllipticKernel {
  public:
    static constexpr size_t num_independent_relations = 4;
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
        static const std::set<PolynomialIndex> required_polynomial_ids = {
            PolynomialIndex::Q_1, PolynomialIndex::Q_3, PolynomialIndex::Q_4, PolynomialIndex::Q_ELLIPTIC,
            PolynomialIndex::W_1, PolynomialIndex::W_2, PolynomialIndex::W_3, PolynomialIndex::W_4
        };
        return required_polynomial_ids;
    }

    /**
     * @brief Computes the single linear term for elliptic point addition
     *
     * @param polynomials Polynomial container or simulator
     * @param challenges Challenge array
     * @param linear_terms Output array
     * @param i Gate index
     */
    inline static void compute_linear_terms(PolyContainer& polynomials,
                                            const challenge_array& challenges,
                                            coefficient_array& linear_terms,
                                            const size_t i = 0)
    {
        const Field& x_1 =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::W_2>(polynomials, i);
        const Field& y_1 =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::W_3>(polynomials, i);
        const Field& x_2 = Getters::template get_value<EvaluationType::SHIFTED, PolynomialIndex::W_1>(polynomials, i);
        const Field& y_2 = Getters::template get_value<EvaluationType::SHIFTED, PolynomialIndex::W_4>(polynomials, i);
        const Field& x_3 = Getters::template get_value<EvaluationType::SHIFTED, PolynomialIndex::W_2>(polynomials, i);
        const Field& y_3 = Getters::template get_value<EvaluationType::SHIFTED, PolynomialIndex::W_3>(polynomials, i);

        // Endomorphism coefficient for when we add and multiply by beta at the same time
        const Field& q_beta =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_3>(polynomials, i);
        // Square of endomorphism coefficient
        const Field& q_beta_sqr =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_4>(polynomials, i);
        // sign
        const Field& q_sign =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_1>(polynomials, i);

        // TODO: Can this be implemented more efficiently?
        // It seems that Zac wanted to group the elements by selectors to use several linear terms initially,
        // but in the end we are using one, so there is no reason why we can't optimize computation in another way

        Field beta_term = -x_2 * x_1 * (x_3 + x_3 + x_1); // -x_1 * x_2 * (2 * x_3 + x_1)
        Field beta_sqr_term = x_2.sqr();                  // x_2^2
        Field leftovers = beta_sqr_term;                  // x_2^2
        beta_sqr_term *= (x_3 - x_1);                     // x_2^2 * (x_3 - x_1)
        Field sign_term = y_2 * y_1;                      // y_1 * y_2
        sign_term += sign_term;                           // 2 * y_1 * y_2
        beta_term *= q_beta;                              // -β * x_1 * x_2 * (2 * x_3 + x_1)
        beta_sqr_term *= q_beta_sqr;                      // β^2 * x_2^2 * (x_3 - x_1)
        sign_term *= q_sign;                              // 2 * y_1 * y_2 * sign
        leftovers *= x_2;                                 // x_2^3
        leftovers += x_1.sqr() * (x_3 + x_1);             // x_2^3 + x_1 * (x_3 + x_1)
        leftovers -= (y_2.sqr() + y_1.sqr());             // x_2^3 + x_1 * (x_3 + x_1) - y_2^2 - y_1^2

        // Can be found in class description
        Field x_identity = beta_term + beta_sqr_term + sign_term + leftovers;
        x_identity *= challenges.alpha_powers[0];

        beta_term = x_2 * (y_3 + y_1) * q_beta;  // β * x_2 * (y_3 + y_1)
        sign_term = -y_2 * (x_1 - x_3) * q_sign; // - signt * y_2 * (x_1 - x_3)
        // TODO: remove extra additions if we decide to stay with this implementation
        leftovers = -x_1 * (y_3 + y_1) + y_1 * (x_1 - x_3); // -x_1 * y_3 - x_1 * y_1 + y_1 * x_1 - y_1 * x_3

        Field y_identity = beta_term + sign_term + leftovers;
        y_identity *= challenges.alpha_powers[1];

        linear_terms[0] = x_identity + y_identity;
    }

    /**
     * @brief Return the linear term multiplied by elliptic curve addition selector value at gate
     *
     * @param polynomials Polynomial container or simulator
     * @param linear_terms Array of linear terms
     * @param i Gate index
     * @return Field
     */
    inline static Field sum_linear_terms(PolyContainer& polynomials,
                                         const challenge_array&,
                                         coefficient_array& linear_terms,
                                         const size_t i = 0)
    {
        const Field& q_elliptic =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_ELLIPTIC>(polynomials, i);
        return linear_terms[0] * q_elliptic;
    }

    inline static void compute_non_linear_terms(PolyContainer&, const challenge_array&, Field&, const size_t = 0) {}

    /**
     * @brief Update opening scalars with the linear term from elliptic gate
     *
     * @param linear_terms Contains input scalar
     * @param scalars Output map for updates
     */
    inline static void update_kate_opening_scalars(coefficient_array& linear_terms,
                                                   std::map<std::string, Field>& scalars,
                                                   const challenge_array&)
    {
        scalars["Q_ELLIPTIC"] += linear_terms[0];
    }
};

} // namespace widget

template <typename Settings>
using ProverEllipticWidget = widget::TransitionWidget<barretenberg::fr, Settings, widget::EllipticKernel>;

template <typename Field, typename Group, typename Transcript, typename Settings>
using VerifierEllipticWidget = widget::GenericVerifierWidget<Field, Transcript, Settings, widget::EllipticKernel>;

} // namespace waffle