#pragma once

#include "./transition_widget.hpp"

namespace waffle {
namespace widget {

template <class Field, class Getters, typename PolyContainer> class TurboArithmeticKernel {
  public:
    static constexpr size_t num_independent_relations = 2;
    // We state the challenges required for linear/nonlinear terms computation
    static constexpr uint8_t quotient_required_challenges = CHALLENGE_BIT_ALPHA;
    // We state the challenges required for updating kate opening scalars
    static constexpr uint8_t update_required_challenges = CHALLENGE_BIT_ALPHA;

  private:
    typedef containers::challenge_array<Field, num_independent_relations> challenge_array;
    typedef containers::coefficient_array<Field> coefficient_array;

  public:
    inline static void compute_linear_terms(PolyContainer& polynomials,
                                            const challenge_array& challenges,
                                            coefficient_array& linear_terms,
                                            const size_t i = 0)
    {
        constexpr barretenberg::fr minus_two(-2);
        const Field& alpha = challenges.elements[ChallengeIndex::ALPHA];
        const Field& w_1 = Getters::template get_polynomial<false, PolynomialIndex::W_1>(polynomials, i);
        const Field& w_2 = Getters::template get_polynomial<false, PolynomialIndex::W_2>(polynomials, i);
        const Field& w_3 = Getters::template get_polynomial<false, PolynomialIndex::W_3>(polynomials, i);
        const Field& w_4 = Getters::template get_polynomial<false, PolynomialIndex::W_4>(polynomials, i);

        const Field& q_arith =
            Getters::template get_polynomial<false, PolynomialIndex::Q_ARITHMETIC_SELECTOR>(polynomials, i);

        Field T0;
        Field T1;
        Field T2;
        Field T3;
        Field T4;
        Field T5;
        Field T6;

        T0 = q_arith * w_1 * w_2;
        T1 = q_arith * w_1;
        T2 = q_arith * w_2;
        T3 = q_arith * w_3;
        T4 = q_arith * w_4;

        // T5 imposes w_4 lies in {0, 1, 2}
        T5 = w_4.sqr();
        T5 -= w_4;
        T6 = w_4 + minus_two;
        T5 *= T6;
        T5 *= q_arith;
        T5 *= alpha;

        linear_terms[0] = T0;
        linear_terms[1] = T1;
        linear_terms[2] = T2;
        linear_terms[3] = T3;
        linear_terms[4] = T4;
        linear_terms[5] = T5;
        linear_terms[6] = q_arith;
    }

    inline static void compute_non_linear_terms(PolyContainer& polynomials,
                                                const challenge_array& challenges,
                                                Field& quotient,
                                                const size_t i = 0)
    {
        constexpr barretenberg::fr minus_seven(-7);

        const Field& w_3 = Getters::template get_polynomial<false, PolynomialIndex::W_3>(polynomials, i);
        const Field& w_4 = Getters::template get_polynomial<false, PolynomialIndex::W_4>(polynomials, i);
        const Field& q_arith =
            Getters::template get_polynomial<false, PolynomialIndex::Q_ARITHMETIC_SELECTOR>(polynomials, i);
        const Field& alpha_base = challenges.alpha_powers[0];

        Field T1;
        Field T2;
        Field T3;
        Field T4;
        Field T5;
        Field T6;

        /**
         * Quad extraction term. This term is only active when q_arith is set to 2.
         *
         * We evaluate ranges using the turbo_range_widget, which generates a sequence
         * of accumulating sums - each sum aggregates a base-4 value.
         *
         * We sometimes need to extract individual bits from our quads, the following
         * term will extract the high bit b from two accumulators, and add 6b into the
         * arithmetic identity.
         *
         * In more detail, a quad will be represented using accumulators stored in
         * wires w_3, w_4 via the formula
         *                 Δ = w_3 - 4.w_4.
         * We'd like to construct the high bit of Δ in each of the four possible cases
         * Δ = 0, 1, 2, 3. We could do this via Lagrange interpolation of the points
         * {(0, 0), (1, 0), (2, 1), (3, 1)} over the domain {0, 1, 2, 3}, leading to
         * the polynomial
         *
         *                     Δ(Δ-1)(Δ-3)  Δ(Δ-1)(Δ-2)
         *      l_2 + l_3   =  ---------- + ----------- .
         *                       2.1.-1        3.2.1
         *
         * Clearing the denominators, we find that
         *      9.Δ^2 - 2.Δ^3 - 7.Δ interpolates {(0, 0), (1, 0), (2, 6), (3, 6)},
         * so we instead treat 6 as the indicator that the high bit of Δ is 1.
         **/

        // T1 = q_arith^2 - q_arith.
        T1 = q_arith.sqr();
        T1 -= q_arith;

        // T2  = Δ
        T2 = w_4 + w_4;
        T2 += T2;
        T2 = w_3 - T2;

        // T3 = 2Δ^2
        T3 = T2.sqr();
        T3 += T3;

        // T4 = 9.Δ
        T4 = T2 + T2;
        T4 += T2;
        // // T5 = 6.Δ
        T5 = T4 + T4;
        T4 += T5;

        // T4 = 9.Δ - 2.Δ^2 - 7
        T4 -= T3;
        T4 += minus_seven;

        // T2 = 9.Δ^2 - 2.Δ^3 - 7.Δ
        T2 *= T4;

        // T1 = (q_arith^2 - q_arith).(9.Δ^2 - 2.Δ^3 - 7.Δ)
        // Note that, when q_arith = 2, q_arith^2 - q_arith = q_arith
        T1 *= T2;

        T1 *= alpha_base;

        quotient += T1;
    }

    inline static Field sum_linear_terms(PolyContainer& polynomials,
                                         const challenge_array& challenges,
                                         coefficient_array& linear_terms,
                                         const size_t i = 0)
    {
        const Field& alpha_base = challenges.alpha_powers[0];
        const Field& q_1 = Getters::template get_polynomial<false, PolynomialIndex::Q_1>(polynomials, i);
        const Field& q_2 = Getters::template get_polynomial<false, PolynomialIndex::Q_2>(polynomials, i);
        const Field& q_3 = Getters::template get_polynomial<false, PolynomialIndex::Q_3>(polynomials, i);
        const Field& q_4 = Getters::template get_polynomial<false, PolynomialIndex::Q_4>(polynomials, i);
        const Field& q_5 = Getters::template get_polynomial<false, PolynomialIndex::Q_5>(polynomials, i);
        const Field& q_m = Getters::template get_polynomial<false, PolynomialIndex::Q_M>(polynomials, i);
        const Field& q_c = Getters::template get_polynomial<false, PolynomialIndex::Q_C>(polynomials, i);

        Field result = linear_terms[0] * q_m;
        result += (linear_terms[1] * q_1);
        result += (linear_terms[2] * q_2);
        result += (linear_terms[3] * q_3);
        result += (linear_terms[4] * q_4);
        result += (linear_terms[5] * q_5);
        result += (linear_terms[6] * q_c);
        result *= alpha_base;
        return result;
    }

    inline static void update_kate_opening_scalars(coefficient_array& linear_terms,
                                                   std::map<std::string, Field>& scalars,
                                                   const challenge_array& challenges)
    {
        const Field& alpha = challenges.alpha_powers[0];
        const Field challenge_product = alpha;
        scalars["Q_M"] += linear_terms[0] * challenge_product;
        scalars["Q_1"] += linear_terms[1] * challenge_product;
        scalars["Q_2"] += linear_terms[2] * challenge_product;
        scalars["Q_3"] += linear_terms[3] * challenge_product;
        scalars["Q_4"] += linear_terms[4] * challenge_product;
        scalars["Q_5"] += linear_terms[5] * challenge_product;
        scalars["Q_C"] += linear_terms[6] * challenge_product;
    }
};

} // namespace widget

template <typename Settings>
using ProverTurboArithmeticWidget = widget::TransitionWidget<barretenberg::fr, Settings, widget::TurboArithmeticKernel>;

template <typename Field, typename Group, typename Transcript, typename Settings>
using VerifierTurboArithmeticWidget =
    widget::GenericVerifierWidget<Field, Transcript, Settings, widget::TurboArithmeticKernel>;

} // namespace waffle