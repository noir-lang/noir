#pragma once

#include "./transition_widget.hpp"

namespace waffle {
namespace widget {

template <class Field, class Getters, typename PolyContainer> class EllipticKernel {
  public:
    static constexpr bool use_quotient_mid = false;
    static constexpr size_t num_independent_relations = 4;

  private:
    typedef containers::challenge_array<Field, num_independent_relations> challenge_array;
    typedef containers::coefficient_array<Field> coefficient_array;

  public:
    inline static void compute_linear_terms(PolyContainer& polynomials,
                                            const challenge_array& challenges,
                                            coefficient_array& linear_terms,
                                            const size_t i = 0)
    {
        const Field& x_1 = Getters::template get_polynomial<false, PolynomialIndex::W_2>(polynomials, i);
        const Field& y_1 = Getters::template get_polynomial<false, PolynomialIndex::W_3>(polynomials, i);
        const Field& x_2 = Getters::template get_polynomial<true, PolynomialIndex::W_1>(polynomials, i);
        const Field& y_2 = Getters::template get_polynomial<true, PolynomialIndex::W_4>(polynomials, i);
        const Field& x_3 = Getters::template get_polynomial<true, PolynomialIndex::W_2>(polynomials, i);
        const Field& y_3 = Getters::template get_polynomial<true, PolynomialIndex::W_3>(polynomials, i);

        const Field& q_beta = Getters::template get_polynomial<false, PolynomialIndex::Q_3>(polynomials, i);
        const Field& q_beta_sqr = Getters::template get_polynomial<false, PolynomialIndex::Q_4>(polynomials, i);
        const Field& q_sign = Getters::template get_polynomial<false, PolynomialIndex::Q_5>(polynomials, i);

        Field beta_term = -x_2 * x_1 * (x_3 + x_3 + x_1);
        Field beta_sqr_term = x_2.sqr();
        Field leftovers = beta_sqr_term;
        beta_sqr_term *= (x_3 - x_1);
        Field sign_term = y_2 * y_1;
        sign_term += sign_term;
        beta_term *= q_beta;
        beta_sqr_term *= q_beta_sqr;
        sign_term *= q_sign;
        leftovers *= x_2;
        leftovers += x_1.sqr() * (x_3 + x_1);
        leftovers -= (y_2.sqr() + y_1.sqr());

        Field x_identity = beta_term + beta_sqr_term + sign_term + leftovers;
        x_identity *= challenges.alpha_powers[0];

        beta_term = x_2 * (y_3 + y_1) * q_beta;
        sign_term = -y_2 * (x_1 - x_3) * q_sign;
        leftovers = -x_1 * (y_3 + y_1) + y_1 * (x_1 - x_3);

        Field y_identity = beta_term + sign_term + leftovers;
        y_identity *= challenges.alpha_powers[1];

        linear_terms[0] = x_identity + y_identity;
    }

    inline static Field sum_linear_terms(PolyContainer& polynomials,
                                         const challenge_array&,
                                         coefficient_array& linear_terms,
                                         const size_t i = 0)
    {
        const Field& q_elliptic = Getters::template get_polynomial<false, PolynomialIndex::Q_ELLIPTIC>(polynomials, i);
        return linear_terms[0] * q_elliptic;
    }

    inline static void compute_non_linear_terms(PolyContainer&, const challenge_array&, Field&, const size_t = 0) {}

    inline static void update_kate_opening_scalars(coefficient_array& linear_terms,
                                                   std::map<std::string, Field>& scalars,
                                                   const challenge_array& challenges)
    {
        const Field& linear_challenge = challenges.elements[ChallengeIndex::LINEAR_NU];
        scalars["Q_ELLIPTIC"] += linear_terms[0] * linear_challenge;
    }
};

} // namespace widget

template <typename Settings>
using ProverEllipticWidget = widget::TransitionWidget<barretenberg::fr, Settings, widget::EllipticKernel>;

template <typename Field, typename Group, typename Transcript, typename Settings>
using VerifierEllipticWidget = widget::GenericVerifierWidget<Field, Transcript, Settings, widget::EllipticKernel>;

} // namespace waffle