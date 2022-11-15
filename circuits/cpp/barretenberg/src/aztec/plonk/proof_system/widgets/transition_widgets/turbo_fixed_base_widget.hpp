#pragma once

#include "./transition_widget.hpp"

namespace waffle {
namespace widget {

/**
 * This gate computes 2-bit NAF elliptic curve addition (aka fixed-based scalar multiplication).
 * The theory is explained in detail in [1]. Suppose the (n+1) gates are strutured as following:
 *
 * +---------+---------+-----------+---------+
 * | w_1     | w_2     | w_3       | w_4     |
 * |---------|---------|-----------|---------|
 * | x_0     | y_0     | c         | a_0     |
 * | x_1     | y_1     | x_{α,0}   | a_1     |
 * | .       | .       | .         | .       |
 * | .       | .       | .         | .       |
 * | .       | .       | .         | .       |
 * | x_i     | y_i     | x_{α,i-1} | a_i     |<- i th gate
 * | x_{i+1} | y_{i+1} | x_{α,i}   | a_{i+1} |
 * | .       | .       | .         | .       |
 * | .       | .       | .         | .       |
 * | .       | .       | .         | .       |
 * | x_n     | y_n     | x_{α,n-1} | a_n     |
 * +---------+---------+-----------+---------+
 *
 * Here, (x_{i+1}, y_{i+1}) = [(x_i, y_i)] + b_i.[(x_{α,i}, y_{α,i})] for i = {0, 1, ..., n-1}.
 * Since the values (a_i) are intermediate sums, the actual quad value b_i is:
 * b_i := a_{i+1} - 4 * a_i.
 *
 * In the implementation below, we call b_i as delta (δ).
 * Let P_0 = 4^{n-1}.[g] and P_1 = (1 + 4^{n-1}).[g].
 * We need the following constraints:
 *
 *
 * 0. Quad value is either of {-3, -1, 1, 3}. See page 6 of [1].
 * => (b_i + 3)(b_i + 1)(b_i - 1)(b_i - 3) = 0
 *
 * 1. Check if x-coordinate of the point to be added is correct. See page 5 of [1].
 * => q_1 * b_i^2 + q_2 = x_{α,i}
 *
 * 2. Check if x-coordinate of new point is correct. See page 7 of [1].
 * => (x_{i+1} + x_i + x_{α,i}) * (x_{α,i} - x_i)^2 +
 *    (2.b_i.y_1) * (q_3.x_{α,i} + q_{ecc,1}) -
 *    (x_{α,i}^3 + y_i^2 + b_{grumpkin}) = 0
 *
 * 3. Check if y-coordinate of new point is correct. See page 7 of [1].
 * => (y_{i+1} + y_i) * (x_{α,i} - x_i) -
 *    (b_i.(q_3.x_{α,i} + q_{ecc,1}) - y_i) * (x_i - x_{i+1}) = 0
 *
 * 4. Initialization: a_0 must be either 1 or (1 + 4^{-n}). See page 7 of [1].
 * => q_c * (1 - a_0).(1 - a_0 - 4^{-n}) = 0
 *
 * 5. Initialization: x_0 must be x-coordinate of either P_0 or P_1.
 * => q_c * (c.(q_4 - x_0) + q_5.(1 - a_0)) = 0
 *
 * 6. Initialization: y_0 must be y-coordinate of either P_0 or P_1.
 * => q_c * (c.(q_m - y_0) + q_c.(1 - a_0)) = 0
 *
 *
 * We combine all of these constraints using powers of the challenge α. Further, the linear and non-linear parts in
 * the constraines are computed separately in the functions `compute_linear_terms` and `compute_linear_terms`
 * respectively. More details on how selector values for i=0  are specially chosen are explained in [3].
 *
 * References:
 * [1] The Turbo-PLONK program syntax for specifying SNARK programs, Ariel G and Zac W.
 *     Link: https://docs.zkproof.org/pages/standards/accepted-workshop3/proposal-turbo_plonk.pdf
 * [2] Constant b_{grumpkin} = -17.
 * [3] Fixed-base Multiplication gate, Cody G.
 *     Link: https://hackmd.io/MCmV2bipRYelT1WUNLj02g
 *
 **/
template <class Field, class Getters, typename PolyContainer> class TurboFixedBaseKernel {
  public:
    static constexpr size_t num_independent_relations = 7;
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
        static const std::set<PolynomialIndex> required_polynomial_ids = {
            PolynomialIndex::Q_1, PolynomialIndex::Q_2, PolynomialIndex::Q_3, PolynomialIndex::Q_4,
            PolynomialIndex::Q_5, PolynomialIndex::Q_M, PolynomialIndex::Q_C, PolynomialIndex::Q_FIXED_BASE_SELECTOR,
            PolynomialIndex::W_1, PolynomialIndex::W_2, PolynomialIndex::W_3, PolynomialIndex::W_4
        };
        return required_polynomial_ids;
    }

    inline static void compute_linear_terms(PolyContainer& polynomials,
                                            const challenge_array& challenges,
                                            coefficient_array& linear_terms,
                                            const size_t i = 0)
    {

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
        const Field& w_3_omega =
            Getters::template get_value<EvaluationType::SHIFTED, PolynomialIndex::W_3>(polynomials, i);
        const Field& w_4_omega =
            Getters::template get_value<EvaluationType::SHIFTED, PolynomialIndex::W_4>(polynomials, i);
        const Field& q_c =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_C>(polynomials, i);
        const Field& q_ecc_1 =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_FIXED_BASE_SELECTOR>(
                polynomials, i);

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
        const Field& q_1 =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_1>(polynomials, i);
        const Field& q_2 =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_2>(polynomials, i);
        const Field& q_3 =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_3>(polynomials, i);
        const Field& q_4 =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_4>(polynomials, i);
        const Field& q_5 =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_5>(polynomials, i);
        const Field& q_m =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_M>(polynomials, i);

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
        const Field& w_2_omega =
            Getters::template get_value<EvaluationType::SHIFTED, PolynomialIndex::W_2>(polynomials, i);
        const Field& w_3_omega =
            Getters::template get_value<EvaluationType::SHIFTED, PolynomialIndex::W_3>(polynomials, i);
        const Field& w_4_omega =
            Getters::template get_value<EvaluationType::SHIFTED, PolynomialIndex::W_4>(polynomials, i);
        const Field& q_c =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_C>(polynomials, i);
        const Field& q_ecc_1 =
            Getters::template get_value<EvaluationType::NON_SHIFTED, PolynomialIndex::Q_FIXED_BASE_SELECTOR>(
                polynomials, i);

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
                                                   const challenge_array&)
    {
        scalars["Q_M"] += linear_terms[0];
        scalars["Q_1"] += linear_terms[1];
        scalars["Q_2"] += linear_terms[2];
        scalars["Q_3"] += linear_terms[3];
        scalars["Q_4"] += linear_terms[4];
        scalars["Q_5"] += linear_terms[5];
    }
};

} // namespace widget

template <typename Settings>
using ProverTurboFixedBaseWidget = widget::TransitionWidget<barretenberg::fr, Settings, widget::TurboFixedBaseKernel>;

template <typename Field, typename Group, typename Transcript, typename Settings>
using VerifierTurboFixedBaseWidget =
    widget::GenericVerifierWidget<Field, Transcript, Settings, widget::TurboFixedBaseKernel>;

} // namespace waffle