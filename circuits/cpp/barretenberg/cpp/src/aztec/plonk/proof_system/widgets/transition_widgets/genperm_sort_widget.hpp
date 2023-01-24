#pragma once

#include "./transition_widget.hpp"

namespace waffle {
namespace widget {

template <class Field, class Getters, typename PolyContainer> class GenPermSortKernel {
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
        static const std::set<PolynomialIndex> required_polynomial_ids = { PolynomialIndex::Q_SORT_SELECTOR,
                                                                           PolynomialIndex::W_1,
                                                                           PolynomialIndex::W_2,
                                                                           PolynomialIndex::W_3,
                                                                           PolynomialIndex::W_4,
                                                                           PolynomialIndex::Z };
        return required_polynomial_ids;
    }

    inline static void compute_linear_terms(PolyContainer& polynomials,
                                            const challenge_array& challenges,
                                            coefficient_array& linear_terms,
                                            const size_t i = 0)
    {
        constexpr barretenberg::fr minus_two(-2);
        constexpr barretenberg::fr minus_three(-3);

        const Field& alpha_base = challenges.alpha_powers[0];
        const Field& alpha = challenges.elements[ChallengeIndex::ALPHA];
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

        Field alpha_a = alpha_base;
        Field alpha_b = alpha_a * alpha;
        Field alpha_c = alpha_b * alpha;
        Field alpha_d = alpha_c * alpha;

        Field delta_1 = w_2 - w_1;
        Field delta_2 = w_3 - w_2;
        Field delta_3 = w_4 - w_3;
        Field delta_4 = w_1_omega - w_4;

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
        const Field& q_sort =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_SORT_SELECTOR>(polynomials, i);

        return linear_terms[0] * q_sort;
    }

    inline static void update_kate_opening_scalars(coefficient_array& linear_terms,
                                                   std::map<std::string, Field>& scalars,
                                                   const challenge_array&)
    {
        scalars["Q_SORT_SELECTOR"] += linear_terms[0];
    }
};

} // namespace widget

template <typename Settings>
using ProverGenPermSortWidget = widget::TransitionWidget<barretenberg::fr, Settings, widget::GenPermSortKernel>;

template <typename Field, typename Group, typename Transcript, typename Settings>
using VerifierGenPermSortWidget = widget::GenericVerifierWidget<Field, Transcript, Settings, widget::GenPermSortKernel>;

} // namespace waffle