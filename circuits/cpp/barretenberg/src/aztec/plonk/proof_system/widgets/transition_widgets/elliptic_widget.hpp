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
        const Field& x_1 = Getters::template get_polynomial<false, PolynomialIndex::W_1>(polynomials, i);
        const Field& y_1 = Getters::template get_polynomial<false, PolynomialIndex::W_2>(polynomials, i);
        const Field& x_2 = Getters::template get_polynomial<false, PolynomialIndex::W_3>(polynomials, i);
        const Field& y_2 = Getters::template get_polynomial<false, PolynomialIndex::W_4>(polynomials, i);
        const Field& x_3 = Getters::template get_polynomial<true, PolynomialIndex::W_1>(polynomials, i);
        const Field& y_3 = Getters::template get_polynomial<true, PolynomialIndex::W_2>(polynomials, i);

        const Field lambda_numerator = y_2 - y_1;
        const Field lambda_denominator = x_2 - x_1;

        const Field T0 = (x_3 + x_2 + x_1) * lambda_denominator.sqr() - lambda_numerator.sqr();
        const Field T1 = (y_3 + y_1) * lambda_denominator - lambda_numerator * (x_1 - x_3);
        linear_terms[0] = T0 * challenges.alpha_powers[0] + T1 * challenges.alpha_powers[1];
    }

    inline static Field sum_linear_terms(PolyContainer& polynomials,
                                         const challenge_array&,
                                         coefficient_array& linear_terms,
                                         const size_t i = 0)
    {
        const Field& q_elliptic = Getters::template get_polynomial<false, PolynomialIndex::Q_ELLIPTIC>(polynomials, i);

        return linear_terms[0] * q_elliptic;
    }

    inline static void compute_non_linear_terms(PolyContainer& polynomials,
                                                const challenge_array& challenges,
                                                Field& quotient,
                                                const size_t i = 0)
    {
        const Field& x_1 = Getters::template get_polynomial<false, PolynomialIndex::W_1>(polynomials, i);
        const Field& y_1 = Getters::template get_polynomial<false, PolynomialIndex::W_2>(polynomials, i);
        const Field& x_3 = Getters::template get_polynomial<true, PolynomialIndex::W_1>(polynomials, i);
        const Field& y_3 = Getters::template get_polynomial<true, PolynomialIndex::W_2>(polynomials, i);
        const Field& q_elliptic_omega =
            Getters::template get_polynomial<true, PolynomialIndex::Q_ELLIPTIC>(polynomials, i);

        Field lambda_numerator = x_1.sqr();
        lambda_numerator += lambda_numerator;
        lambda_numerator += lambda_numerator;
        const Field lambda_denominator = y_1 + y_1;

        const Field T0 = (x_1 + x_1 + x_3) * lambda_denominator.sqr() - lambda_numerator.sqr();
        const Field T1 = (y_3 + y_1) * lambda_denominator - lambda_numerator * (x_1 - x_3);

        quotient += (T0 * challenges.alpha_powers[2] + T1 * challenges.alpha_powers[3]) * q_elliptic_omega;
    }

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