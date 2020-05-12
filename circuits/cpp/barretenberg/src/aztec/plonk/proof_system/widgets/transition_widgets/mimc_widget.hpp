#pragma once

#include "./transition_widget.hpp"

namespace waffle {
namespace widget {

template <class Field, class Getters, typename PolyContainer> class MiMCKernel {
  public:
    static constexpr bool use_quotient_mid = false;
    static constexpr size_t num_independent_relations = 2;

  private:
    typedef containers::challenge_array<Field, num_independent_relations> challenge_array;
    typedef containers::coefficient_array<Field> coefficient_array;

  public:
    inline static void compute_linear_terms(PolyContainer& polynomials,
                                            const challenge_array& challenges,
                                            coefficient_array& linear_terms,
                                            const size_t i = 0)
    {
        const Field& w_1 = Getters::template get_polynomial<false, PolynomialIndex::W_1>(polynomials, i);
        const Field& w_2 = Getters::template get_polynomial<false, PolynomialIndex::W_2>(polynomials, i);
        const Field& w_3 = Getters::template get_polynomial<false, PolynomialIndex::W_3>(polynomials, i);
        const Field& w_3_omega = Getters::template get_polynomial<true, PolynomialIndex::W_3>(polynomials, i);
        const Field& q_mimc_coefficient =
            Getters::template get_polynomial<false, PolynomialIndex::Q_MIMC_COEFFICIENT>(polynomials, i);

        const Field T0 = w_1 + w_3 + q_mimc_coefficient;
        const Field T1 = (T0.sqr() * T0) - w_2;
        const Field T2 = (w_2.sqr() * T0 - w_3_omega);
        const Field T3 = T1 * challenges.alpha_powers[0] + T2 * challenges.alpha_powers[1];

        linear_terms[0] = T3;
    }

    inline static void compute_non_linear_terms(PolyContainer&, const challenge_array&, Field&, const size_t = 0) {}

    inline static Field sum_linear_terms(PolyContainer& polynomials,
                                         const challenge_array&,
                                         coefficient_array& linear_terms,
                                         const size_t i = 0)
    {
        const Field& q_mimc_selector =
            Getters::template get_polynomial<false, PolynomialIndex::Q_MIMC_SELECTOR>(polynomials, i);

        return linear_terms[0] * q_mimc_selector;
    }

    inline static void update_kate_opening_scalars(coefficient_array& linear_terms,
                                                   std::map<std::string, Field>& scalars,
                                                   const challenge_array& challenges)
    {
        const Field& linear_challenge = challenges.elements[ChallengeIndex::LINEAR_NU];
        scalars["Q_MIMC_SELECTOR"] += linear_terms[0] * linear_challenge;
    }
};

} // namespace widget

template <typename Settings>
using ProverMiMCWidget = widget::TransitionWidget<barretenberg::fr, Settings, widget::MiMCKernel>;

template <typename Field, typename Group, typename Transcript, typename Settings>
using VerifierMiMCWidget = widget::GenericVerifierWidget<Field, Transcript, Settings, widget::MiMCKernel>;

} // namespace waffle