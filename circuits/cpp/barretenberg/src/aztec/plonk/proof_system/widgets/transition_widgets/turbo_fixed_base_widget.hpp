#pragma once

#include "./transition_widget.hpp"

namespace waffle {
namespace widget {

template <class Field, class Getters, typename PolyContainer> class TurboFixedBaseKernel {
  public:
    static constexpr bool use_quotient_mid = false;
    static constexpr size_t num_independent_relations = 7;

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
        const Field& w_4 = Getters::template get_polynomial<false, PolynomialIndex::W_4>(polynomials, i);
        const Field& w_1_omega = Getters::template get_polynomial<true, PolynomialIndex::W_1>(polynomials, i);
        const Field& w_3_omega = Getters::template get_polynomial<true, PolynomialIndex::W_3>(polynomials, i);
        const Field& w_4_omega = Getters::template get_polynomial<true, PolynomialIndex::W_4>(polynomials, i);
        const Field& q_c = Getters::template get_polynomial<false, PolynomialIndex::Q_C>(polynomials, i);
        const Field& q_ecc_1 =
            Getters::template get_polynomial<false, PolynomialIndex::Q_FIXED_BASE_SELECTOR>(polynomials, i);

        Field delta = w_4_omega - (w_4 + w_4 + w_4 + w_4);

        Field delta_squared = delta.sqr();

        Field q_1_multiplicand = delta_squared * q_ecc_1 * challenges.alpha_powers[1];

        Field q_2_multiplicand = challenges.alpha_powers[1] * q_ecc_1;

        Field q_3_multiplicand = (w_1_omega - w_1) * delta * w_3_omega * challenges.alpha_powers[3] * q_ecc_1;
        Field T1 = delta * w_3_omega * w_2 * challenges.alpha_powers[2];
        q_3_multiplicand = q_3_multiplicand + (T1 + T1) * q_ecc_1;

        Field q_4_multiplicand = w_3 * q_ecc_1 * q_c * challenges.alpha_powers[5];

        Field q_5_multiplicand = (Field(1) - w_4) * q_ecc_1 * q_c * challenges.alpha_powers[5];

        Field q_m_multiplicand = w_3 * q_ecc_1 * q_c * challenges.alpha_powers[6];

        linear_terms[0] = q_m_multiplicand;
        linear_terms[1] = q_1_multiplicand;
        linear_terms[2] = q_2_multiplicand;
        linear_terms[3] = q_3_multiplicand;
        linear_terms[4] = q_4_multiplicand;
        linear_terms[5] = q_5_multiplicand;
    }

    inline static Field sum_linear_terms(PolyContainer& polynomials,
                                         const challenge_array&,
                                         coefficient_array& linear_terms,
                                         const size_t i = 0)
    {
        const Field& q_1 = Getters::template get_polynomial<false, PolynomialIndex::Q_1>(polynomials, i);
        const Field& q_2 = Getters::template get_polynomial<false, PolynomialIndex::Q_2>(polynomials, i);
        const Field& q_3 = Getters::template get_polynomial<false, PolynomialIndex::Q_3>(polynomials, i);
        const Field& q_4 = Getters::template get_polynomial<false, PolynomialIndex::Q_4>(polynomials, i);
        const Field& q_5 = Getters::template get_polynomial<false, PolynomialIndex::Q_5>(polynomials, i);
        const Field& q_m = Getters::template get_polynomial<false, PolynomialIndex::Q_M>(polynomials, i);

        Field result = linear_terms[0] * q_m;
        result += (linear_terms[1] * q_1);
        result += (linear_terms[2] * q_2);
        result += (linear_terms[3] * q_3);
        result += (linear_terms[4] * q_4);
        result += (linear_terms[5] * q_5);
        return result;
    }

    inline static void compute_non_linear_terms(PolyContainer& polynomials,
                                                const challenge_array& challenges,
                                                Field& quotient,
                                                const size_t i = 0)
    {
        constexpr barretenberg::fr grumpkin_curve_b(-17);

        const Field& w_1 = Getters::template get_polynomial<false, PolynomialIndex::W_1>(polynomials, i);
        const Field& w_2 = Getters::template get_polynomial<false, PolynomialIndex::W_2>(polynomials, i);
        const Field& w_3 = Getters::template get_polynomial<false, PolynomialIndex::W_3>(polynomials, i);
        const Field& w_4 = Getters::template get_polynomial<false, PolynomialIndex::W_4>(polynomials, i);
        const Field& w_1_omega = Getters::template get_polynomial<true, PolynomialIndex::W_1>(polynomials, i);
        const Field& w_2_omega = Getters::template get_polynomial<true, PolynomialIndex::W_2>(polynomials, i);
        const Field& w_3_omega = Getters::template get_polynomial<true, PolynomialIndex::W_3>(polynomials, i);
        const Field& w_4_omega = Getters::template get_polynomial<true, PolynomialIndex::W_4>(polynomials, i);
        const Field& q_c = Getters::template get_polynomial<false, PolynomialIndex::Q_C>(polynomials, i);
        const Field& q_ecc_1 =
            Getters::template get_polynomial<false, PolynomialIndex::Q_FIXED_BASE_SELECTOR>(polynomials, i);

        Field delta = w_4_omega - (w_4 + w_4 + w_4 + w_4);
        const Field three = Field(3);
        Field T1 = (delta + Field(1));
        Field T2 = (delta + three);
        Field T3 = (delta - Field(1));
        Field T4 = (delta - three);

        Field accumulator_identity = T1 * T2 * T3 * T4 * challenges.alpha_powers[0];

        Field x_alpha_identity = -(w_3_omega * challenges.alpha_powers[1]);

        Field T0 = w_1_omega + w_1 + w_3_omega;
        T1 = (w_3_omega - w_1).sqr();
        T0 = T0 * T1;

        T1 = w_3_omega.sqr() * w_3_omega;
        T2 = w_2.sqr();
        T1 = T1 + T2;
        T1 = -(T1 + grumpkin_curve_b);

        T2 = delta * w_2 * q_ecc_1;
        T2 = T2 + T2;

        Field x_accumulator_identity = (T0 + T1 + T2) * challenges.alpha_powers[2];

        T0 = (w_2_omega + w_2) * (w_3_omega - w_1);

        T1 = w_1 - w_1_omega;
        T2 = w_2 - (q_ecc_1 * delta);
        T1 = T1 * T2;

        Field y_accumulator_identity = (T0 + T1) * challenges.alpha_powers[3];

        T0 = w_4 - Field(1);
        T1 = T0 - w_3;
        Field accumulator_init_identity = T0 * T1 * challenges.alpha_powers[4];

        Field x_init_identity = -(w_1 * w_3) * challenges.alpha_powers[5];

        T0 = Field(1) - w_4;
        T0 = T0 * q_c;
        T1 = w_2 * w_3;
        Field y_init_identity = (T0 - T1) * challenges.alpha_powers[6];

        Field gate_identity = accumulator_init_identity + x_init_identity + y_init_identity;
        gate_identity = gate_identity * q_c;
        gate_identity =
            gate_identity + accumulator_identity + x_alpha_identity + x_accumulator_identity + y_accumulator_identity;
        gate_identity = gate_identity * q_ecc_1;

        quotient += gate_identity;
    }

    inline static void update_kate_opening_scalars(coefficient_array& linear_terms,
                                                   std::map<std::string, Field>& scalars,
                                                   const challenge_array& challenges)
    {
        const Field& linear_challenge = challenges.elements[ChallengeIndex::LINEAR_NU];
        scalars["Q_M"] += linear_terms[0] * linear_challenge;
        scalars["Q_1"] += linear_terms[1] * linear_challenge;
        scalars["Q_2"] += linear_terms[2] * linear_challenge;
        scalars["Q_3"] += linear_terms[3] * linear_challenge;
        scalars["Q_4"] += linear_terms[4] * linear_challenge;
        scalars["Q_5"] += linear_terms[5] * linear_challenge;
    }
};

} // namespace widget

template <typename Settings>
using ProverTurboFixedBaseWidget = widget::TransitionWidget<barretenberg::fr, Settings, widget::TurboFixedBaseKernel>;

template <typename Field, typename Group, typename Transcript, typename Settings>
using VerifierTurboFixedBaseWidget =
    widget::GenericVerifierWidget<Field, Transcript, Settings, widget::TurboFixedBaseKernel>;

} // namespace waffle