#pragma once

#include "./transition_widget.hpp"

namespace bb::plonk {
namespace widget {

/**
 * @brief Core class implementing elliptic curve point addition. It is enhanced to handle the case where one of the
 * points is automatically scaled by the endomorphism constant β or negated
 *
 *
 * TODO(#429): based on the ultra honk relation consistency test, the below expressions differ
 * slightly from what is actually implemented. (Mostly sign errors; some incorrect terms)
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

  public:
    inline static std::set<PolynomialIndex> const& get_required_polynomial_ids()
    {
        static const std::set<PolynomialIndex> required_polynomial_ids = {
            PolynomialIndex::Q_1, PolynomialIndex::Q_3,        PolynomialIndex::Q_4,
            PolynomialIndex::Q_M, PolynomialIndex::Q_ELLIPTIC, PolynomialIndex::W_1,
            PolynomialIndex::W_2, PolynomialIndex::W_3,        PolynomialIndex::W_4
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
    inline static void accumulate_contribution(PolyContainer& polynomials,
                                               const challenge_array& challenges,
                                               Field& quotient,
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
        const Field& q_elliptic =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_ELLIPTIC>(polynomials, i);

        // sign
        const Field& q_sign =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_1>(polynomials, i);

        // ecc add gate is active when q_elliptic = 1 and q_m = 0
        // ecc double gate is active when q_elliptic = 1 and q_m = 1
        const Field& q_is_double =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_M>(polynomials, i);

        Field x_diff = x_2 - x_1;
        Field y1_sqr = y_1.sqr();
        Field y2_sqr = y_2.sqr();
        Field y1y2 = y_1 * y_2 * q_sign;
        Field x_identity_add = (x_3 + x_2 + x_1) * x_diff.sqr() - y1_sqr - y2_sqr + y1y2 + y1y2;
        Field y_identity_add = (y_3 + y_1) * x_diff + (x_3 - x_1) * (y_2 * q_sign - y_1);

        // x-coordinate identity
        // (x3 + 2x1)(4y^2) - (9x^4) = 0
        // This is degree 4...but
        // we can use x^3 = y^2 - b
        // (x3 + 2x1)(4y ^ 2) - (9x(y ^ 2 - b)) is degree 3
        const Field x_pow_4 = (y_1 * y_1 - grumpkin::g1::curve_b) * x_1;
        Field x_identity_double = (x_3 + x_1 + x_1) * (y_1 + y_1) * (y_1 + y_1) - x_pow_4 * Field(9);

        // Y identity: (x1 - x3)(3x^2) - (2y1)(y1 + y3) = 0
        const Field x_pow_2 = (x_1 * x_1);
        Field y_identity_double = x_pow_2 * (x_1 - x_3) * 3 - (y_1 + y_1) * (y_1 + y_3);

        auto x_identity =
            (q_is_double * (x_identity_double - x_identity_add) + x_identity_add) * challenges.alpha_powers[0];
        auto y_identity =
            (q_is_double * (y_identity_double - y_identity_add) + y_identity_add) * challenges.alpha_powers[1];
        Field identity = x_identity + y_identity;

        quotient += identity * q_elliptic;
    }
};

} // namespace widget

template <typename Settings>
using ProverEllipticWidget = widget::TransitionWidget<bb::fr, Settings, widget::EllipticKernel>;

template <typename Field, typename Group, typename Transcript, typename Settings>
using VerifierEllipticWidget = widget::GenericVerifierWidget<Field, Transcript, Settings, widget::EllipticKernel>;

} // namespace bb::plonk