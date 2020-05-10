#pragma once

#include "./transition_widget.hpp"

namespace waffle {
namespace widget {

template <class Field, class Getters, typename PolyContainer> class TurboLogicKernel {
  private:
    typedef containers::challenge_array<Field> challenge_array;
    typedef containers::coefficient_array<Field> coefficient_array;

  public:
    static constexpr bool use_quotient_mid = false;

    inline static void compute_linear_terms(PolyContainer& polynomials,
                                            const challenge_array& challenges,
                                            coefficient_array& linear_terms,
                                            const size_t i = 0)
    {
        constexpr barretenberg::fr six(6);
        constexpr barretenberg::fr eighty_one(81);
        constexpr barretenberg::fr eighty_three(83);

        const Field& alpha_base = challenges[ChallengeIndex::ALPHA_BASE];
        const Field& alpha = challenges[ChallengeIndex::ALPHA];
        const Field& w_1 = Getters::template get_polynomial<false, PolynomialIndex::W_1>(polynomials, i);
        const Field& w_2 = Getters::template get_polynomial<false, PolynomialIndex::W_2>(polynomials, i);
        const Field& w_3 = Getters::template get_polynomial<false, PolynomialIndex::W_3>(polynomials, i);
        const Field& w_4 = Getters::template get_polynomial<false, PolynomialIndex::W_4>(polynomials, i);
        const Field& w_1_omega = Getters::template get_polynomial<true, PolynomialIndex::W_1>(polynomials, i);
        const Field& w_2_omega = Getters::template get_polynomial<true, PolynomialIndex::W_2>(polynomials, i);
        const Field& w_4_omega = Getters::template get_polynomial<true, PolynomialIndex::W_4>(polynomials, i);

        const Field& q_c = Getters::template get_polynomial<false, PolynomialIndex::Q_C>(polynomials, i);

        Field delta_sum;
        Field delta_squared_sum;
        Field T0;
        Field T1;
        Field T2;
        Field T3;
        Field T4;
        Field identity;

        T0 = w_1 + w_1;
        T0 += T0;
        T0 = w_1_omega - T0;

        // T1 = b
        T1 = w_2 + w_2;
        T1 += T1;
        T1 = w_2_omega - T1;

        // delta_sum = a + b
        delta_sum = T0 + T1;

        // T2 = a^2, T3 = b^2
        T2 = T0.sqr();
        T3 = T1.sqr();

        delta_squared_sum = T2 + T3;

        // identity = a^2 + b^2 + 2ab
        identity = delta_sum.sqr();
        // identity = 2ab
        identity -= delta_squared_sum;

        // identity = 2(ab - w)
        T4 = w_3 + w_3;
        identity -= T4;
        identity *= alpha;

        // T4 = 4w
        T4 += T4;

        // T2 = a^2 - a
        T2 -= T0;

        // T0 = a^2 - 5a + 6
        T0 += T0;
        T0 += T0;
        T0 = T2 - T0;
        T0 += six;

        // identity = (identity + a(a - 1)(a - 2)(a - 3)) * alpha
        T0 *= T2;
        identity += T0;
        identity *= alpha;

        // T3 = b^2 - b
        T3 -= T1;

        // T1 = b^2 - 5b + 6
        T1 += T1;
        T1 += T1;
        T1 = T3 - T1;
        T1 += six;

        // identity = (identity + b(b - 1)(b - 2)(b - 3)) * alpha
        T1 *= T3;
        identity += T1;
        identity *= alpha;

        // T0 = 3(a + b)
        T0 = delta_sum + delta_sum;
        T0 += delta_sum;

        // T1 = 9(a + b)
        T1 = T0 + T0;
        T1 += T0;

        // delta_sum = 18(a + b)
        delta_sum = T1 + T1;

        // T1 = 81(a + b)
        T2 = delta_sum + delta_sum;
        T2 += T2;
        T1 += T2;

        // delta_squared_sum = 18(a^2 + b^2)
        T2 = delta_squared_sum + delta_squared_sum;
        T2 += delta_squared_sum;
        delta_squared_sum = T2 + T2;
        delta_squared_sum += T2;
        delta_squared_sum += delta_squared_sum;

        // delta_sum = w(4w - 18(a + b) + 81)
        delta_sum = T4 - delta_sum;
        delta_sum += eighty_one;
        delta_sum *= w_3;

        // T1 = 18(a^2 + b^2) - 81(a + b) + 83
        T1 = delta_squared_sum - T1;
        T1 += eighty_three;

        // delta_sum = w ( w ( 4w - 18(a + b) + 81) + 18(a^2 + b^2) - 81(a + b) + 83)
        delta_sum += T1;
        delta_sum *= w_3;

        // T2 = 3c
        T2 = w_4 + w_4;
        T2 += T2;
        T2 = w_4_omega - T2;
        T3 = T2 + T2;
        T2 += T3;

        // T3 = 9c
        T3 = T2 + T2;
        T3 += T2;

        // T3 = q_c * (9c - 3(a + b))
        T3 -= T0;
        T3 *= q_c;

        // T2 = 3c + 3(a + b) - 2 * delta_sum
        T2 += T0;
        delta_sum += delta_sum;
        T2 -= delta_sum;

        // T2 = T2 + T3
        T2 += T3;

        // identity = q_logic * alpha_base * (identity + T2)
        identity += T2;
        identity *= alpha_base;

        linear_terms[0] = identity;
    }

    inline static void compute_non_linear_terms(PolyContainer&, const challenge_array&, Field&, const size_t = 0) {}

    inline static Field sum_linear_terms(PolyContainer& polynomials,
                                         const challenge_array&,
                                         coefficient_array& linear_terms,
                                         const size_t i = 0)
    {
        const Field& q_logic =
            Getters::template get_polynomial<false, PolynomialIndex::Q_LOGIC_SELECTOR>(polynomials, i);

        return linear_terms[0] * q_logic;
    }

    inline static void update_kate_opening_scalars(coefficient_array& linear_terms,
                                                   std::map<std::string, Field>& scalars,
                                                   const challenge_array& challenges)
    {
        const Field& linear_challenge = challenges[ChallengeIndex::LINEAR_NU];
        scalars["Q_LOGIC_SELECTOR"] += linear_terms[0] * linear_challenge;
    }

    inline static Field update_alpha(const Field& alpha_base, const Field& alpha)
    {
        return alpha_base * alpha.sqr().sqr();
    }

    static void compute_round_commitments(
        proving_key*, program_witness*, transcript::StandardTranscript&, const size_t, work_queue&){};
};

} // namespace widget

template <typename Settings>
using ProverTurboLogicWidget = widget::TransitionWidget<barretenberg::fr, Settings, widget::TurboLogicKernel>;

template <typename Field, typename Group, typename Transcript, typename Settings>
using VerifierTurboLogicWidget = widget::GenericVerifierWidget<Field, Transcript, Settings, widget::TurboLogicKernel>;

} // namespace waffle