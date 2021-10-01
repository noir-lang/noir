#pragma once

#include "./transition_widget.hpp"

namespace waffle {
namespace widget {

template <class Field, class Getters, typename PolyContainer> class ArithmeticKernel {
  public:
    static constexpr bool use_quotient_mid = false;
    static constexpr size_t num_independent_relations = 1;
    // We state the challenges required for linear/nonlinear terms computation
    static constexpr uint8_t quotient_required_challenges = CHALLENGE_BIT_ALPHA;
    // We state the challenges required for updating kate opening scalars
    static constexpr uint8_t update_required_challenges = CHALLENGE_BIT_ALPHA;

  private:
    typedef containers::challenge_array<Field, num_independent_relations> challenge_array;
    typedef containers::coefficient_array<Field> coefficient_array;

  public:
    inline static void compute_linear_terms(PolyContainer& polynomials,
                                            const challenge_array&,
                                            coefficient_array& linear_terms,
                                            const size_t i = 0)
    {
        const Field& w_1 = Getters::template get_polynomial<false, PolynomialIndex::W_1>(polynomials, i);
        const Field& w_2 = Getters::template get_polynomial<false, PolynomialIndex::W_2>(polynomials, i);
        const Field& w_3 = Getters::template get_polynomial<false, PolynomialIndex::W_3>(polynomials, i);

        linear_terms[0] = w_1 * w_2;
        linear_terms[1] = w_1;
        linear_terms[2] = w_2;
        linear_terms[3] = w_3;
    }

    inline static void compute_non_linear_terms(PolyContainer&, const challenge_array&, Field&, const size_t = 0) {}

    inline static Field sum_linear_terms(PolyContainer& polynomials,
                                         const challenge_array& challenges,
                                         coefficient_array& linear_terms,
                                         const size_t i = 0)
    {
        const Field& alpha = challenges.alpha_powers[0];
        const Field& q_1 = Getters::template get_polynomial<false, PolynomialIndex::Q_1>(polynomials, i);
        const Field& q_2 = Getters::template get_polynomial<false, PolynomialIndex::Q_2>(polynomials, i);
        const Field& q_3 = Getters::template get_polynomial<false, PolynomialIndex::Q_3>(polynomials, i);
        const Field& q_m = Getters::template get_polynomial<false, PolynomialIndex::Q_M>(polynomials, i);
        const Field& q_c = Getters::template get_polynomial<false, PolynomialIndex::Q_C>(polynomials, i);

        Field result = linear_terms[0] * q_m;
        result += (linear_terms[1] * q_1);
        result += (linear_terms[2] * q_2);
        result += (linear_terms[3] * q_3);
        result += q_c;
        result *= alpha;
        return result;
    }

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

template <typename Settings>
using ProverArithmeticWidget = widget::TransitionWidget<barretenberg::fr, Settings, widget::ArithmeticKernel>;

template <typename Field, typename Group, typename Transcript, typename Settings>
using VerifierArithmeticWidget = widget::GenericVerifierWidget<Field, Transcript, Settings, widget::ArithmeticKernel>;

} // namespace waffle