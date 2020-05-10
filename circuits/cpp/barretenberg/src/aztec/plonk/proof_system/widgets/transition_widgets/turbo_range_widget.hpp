#pragma once

#include "./transition_widget.hpp"

namespace waffle {
namespace widget {

template <class Field, class Getters, typename PolyContainer> class TurboRangeKernel {
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
        constexpr barretenberg::fr minus_two(-2);
        constexpr barretenberg::fr minus_three(-3);

        const Field& alpha_base = challenges[ChallengeIndex::ALPHA_BASE];
        const Field& alpha = challenges[ChallengeIndex::ALPHA];
        const Field& w_1 = Getters::template get_polynomial<false, PolynomialIndex::W_1>(polynomials, i);
        const Field& w_2 = Getters::template get_polynomial<false, PolynomialIndex::W_2>(polynomials, i);
        const Field& w_3 = Getters::template get_polynomial<false, PolynomialIndex::W_3>(polynomials, i);
        const Field& w_4 = Getters::template get_polynomial<false, PolynomialIndex::W_4>(polynomials, i);
        const Field& w_4_omega = Getters::template get_polynomial<true, PolynomialIndex::W_4>(polynomials, i);

        Field alpha_a = alpha_base;
        Field alpha_b = alpha_a * alpha;
        Field alpha_c = alpha_b * alpha;
        Field alpha_d = alpha_c * alpha;

        Field delta_1 = w_4 + w_4;
        delta_1 += delta_1;
        delta_1 = w_3 - delta_1;

        Field delta_2 = w_3 + w_3;
        delta_2 += delta_2;
        delta_2 = w_2 - delta_2;

        Field delta_3 = w_2 + w_2;
        delta_3 += delta_3;
        delta_3 = w_1 - delta_3;

        Field delta_4 = w_1 + w_1;
        delta_4 += delta_4;
        delta_4 = w_4_omega - delta_4;

        // D(D - 1)(D - 2)(D - 3).alpha
        Field T0 = delta_1.sqr();
        T0 -= delta_1;
        Field T1 = delta_1 + minus_two;
        T0 *= T1;
        T1 = delta_1 + minus_three;
        T0 *= T1;
        Field range_accumulator = T0 * alpha_a;

        T0 = delta_2.sqr();
        T0 -= delta_2;
        T1 = delta_2 + minus_two;
        T0 *= T1;
        T1 = delta_2 + minus_three;
        T0 *= T1;
        T0 *= alpha_b;
        range_accumulator += T0;

        T0 = delta_3.sqr();
        T0 -= delta_3;
        T1 = delta_3 + minus_two;
        T0 *= T1;
        T1 = delta_3 + minus_three;
        T0 *= T1;
        T0 *= alpha_c;
        range_accumulator += T0;

        T0 = delta_4.sqr();
        T0 -= delta_4;
        T1 = delta_4 + minus_two;
        T0 *= T1;
        T1 = delta_4 + minus_three;
        T0 *= T1;
        T0 *= alpha_d;
        range_accumulator += T0;

        linear_terms[0] = range_accumulator;
    }

    inline static void compute_non_linear_terms(PolyContainer&, const challenge_array&, Field&, const size_t = 0) {}

    inline static Field sum_linear_terms(PolyContainer& polynomials,
                                         const challenge_array&,
                                         coefficient_array& linear_terms,
                                         const size_t i = 0)
    {
        const Field& q_range =
            Getters::template get_polynomial<false, PolynomialIndex::Q_RANGE_SELECTOR>(polynomials, i);

        return linear_terms[0] * q_range;
    }

    inline static void update_kate_opening_scalars(coefficient_array& linear_terms,
                                                   std::map<std::string, Field>& scalars,
                                                   const challenge_array& challenges)
    {
        const Field& linear_challenge = challenges[ChallengeIndex::LINEAR_NU];
        scalars["Q_RANGE_SELECTOR"] += linear_terms[0] * linear_challenge;
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
using ProverTurboRangeWidget = widget::TransitionWidget<barretenberg::fr, Settings, widget::TurboRangeKernel>;

template <typename Field, typename Group, typename Transcript, typename Settings>
using VerifierTurboRangeWidget = widget::GenericVerifierWidget<Field, Transcript, Settings, widget::TurboRangeKernel>;

} // namespace waffle