#pragma once
#include "relation_parameters.hpp"
#include "relation_types.hpp"

namespace proof_system {

/**
 * @brief LookupRelationImpl defines the algebra for the lookup polynomial:
 *
 *                       ∏ (1 + β) ⋅ (q_lookup*f_k + γ) ⋅ (t_k + βt_{k+1} + γ(1 + β))
 *  Z_lookup(g^j) = --------------------------------------------------------------------------
 *                                      ∏ (s_k + βs_{k+1} + γ(1 + β))
 *
 *
 * The method `compute_numerator_term` computes polynomials f, t and incorporate them into terms that are ultimately
 * needed to construct the grand product polynomial Z_lookup(X): Note 1: In the above, 't' is associated with table
 * values (and is not to be confused with the quotient polynomial, also refered to as 't' elsewhere). Polynomial 's' is
 * the sorted  concatenation of the witnesses and the table values.
 *
 * @tparam FF parametrises the prime field class being used
 */
template <typename FF_> class LookupRelationImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 2> SUBRELATION_LENGTHS{
        6, // grand product construction sub-relation
        3  // left-shiftable polynomial sub-relation
    };

    /**
     * @brief Get the grand product polynomial object (either from the proving key or AllEntities depending on context)
     *
     * @param input
     * @return auto& either std::span<FF> or Flavor::Polynomial depending on context
     */
    inline static auto& get_grand_product_polynomial(auto& input) { return input.z_lookup; }

    /**
     * @brief Get the shifted grand product polynomial object (either from the proving key or AllEntities depending on
     * context)
     *
     * @param input
     * @return auto& either std::span<FF> or Flavor::Polynomial depending on context
     */
    inline static auto& get_shifted_grand_product_polynomial(auto& input) { return input.z_lookup_shift; }

    /**
     * @brief Compute numerator term of the lookup relation:
     *
     *     N_{index} = (1 + β) ⋅ ∏ (q_lookup*f_k + γ) ⋅ (t_k + βt_{k+1} + γ(1 + β))
     *
     * @tparam AccumulatorTypes
     * @param in
     * @param relation_parameters
     * @param index If calling this method over vector inputs, index >= 0
     */
    template <typename Accumulator, typename AllEntities>
    inline static Accumulator compute_grand_product_numerator(const AllEntities& in,
                                                              const RelationParameters<FF>& relation_parameters)
    {
        const auto& beta = relation_parameters.beta;
        const auto& gamma = relation_parameters.gamma;
        const auto& eta = relation_parameters.eta;
        const auto eta_sqr = eta * eta;
        const auto eta_cube = eta_sqr * eta;

        const auto one_plus_beta = FF(1) + beta;
        const auto gamma_by_one_plus_beta = gamma * one_plus_beta;

        using View = typename Accumulator::View;
        auto w_1 = View(in.w_l);
        auto w_2 = View(in.w_r);
        auto w_3 = View(in.w_o);

        auto w_1_shift = View(in.w_l_shift);
        auto w_2_shift = View(in.w_r_shift);
        auto w_3_shift = View(in.w_o_shift);

        auto table_1 = View(in.table_1);
        auto table_2 = View(in.table_2);
        auto table_3 = View(in.table_3);
        auto table_4 = View(in.table_4);

        auto table_1_shift = View(in.table_1_shift);
        auto table_2_shift = View(in.table_2_shift);
        auto table_3_shift = View(in.table_3_shift);
        auto table_4_shift = View(in.table_4_shift);

        auto table_index = View(in.q_o);
        auto column_1_step_size = View(in.q_r);
        auto column_2_step_size = View(in.q_m);
        auto column_3_step_size = View(in.q_c);
        auto q_lookup = View(in.q_lookup);

        // (w_1 + q_2*w_1_shift) + η(w_2 + q_m*w_2_shift) + η²(w_3 + q_c*w_3_shift) + η³q_index.
        auto wire_accum = (w_1 + column_1_step_size * w_1_shift) + (w_2 + column_2_step_size * w_2_shift) * eta +
                          (w_3 + column_3_step_size * w_3_shift) * eta_sqr + table_index * eta_cube;

        // t_1 + ηt_2 + η²t_3 + η³t_4
        auto table_accum = table_1 + table_2 * eta + table_3 * eta_sqr + table_4 * eta_cube;
        // t_1_shift + ηt_2_shift + η²t_3_shift + η³t_4_shift
        auto table_accum_shift =
            table_1_shift + table_2_shift * eta + table_3_shift * eta_sqr + table_4_shift * eta_cube;

        auto tmp = (q_lookup * wire_accum + gamma);
        tmp *= (table_accum + table_accum_shift * beta + gamma_by_one_plus_beta);
        tmp *= one_plus_beta;
        return tmp;
    }

    /**
     * @brief Compute denominator term of the lookup relation:
     *
     *      (s_k + βs_{k+1} + γ(1 + β))
     *
     * @tparam AccumulatorTypes
     * @param in
     * @param relation_parameters
     * @param index
     */
    template <typename Accumulator, typename AllEntities>
    inline static Accumulator compute_grand_product_denominator(const AllEntities& in,
                                                                const RelationParameters<FF>& relation_parameters)
    {

        const auto& beta = relation_parameters.beta;
        const auto& gamma = relation_parameters.gamma;

        const auto one_plus_beta = FF(1) + beta;
        const auto gamma_by_one_plus_beta = gamma * one_plus_beta;

        using View = typename Accumulator::View;

        // Contribution (1)
        auto s_accum = View(in.sorted_accum);
        auto s_accum_shift = View(in.sorted_accum_shift);

        auto tmp = (s_accum + s_accum_shift * beta + gamma_by_one_plus_beta);
        return tmp;
    }

    /**
     * @brief Compute contribution of the lookup grand prod relation for a given edge (internal function)
     *
     * @details This the relation confirms faithful calculation of the lookup grand
     * product polynomial Z_lookup. The contribution is
     *      z_lookup * (1 + β) * [q_lookup * f + γ] * (t_accum_k + βt_accum_{k+1} + γ(1 + β)) -
     *      z_lookup_shift * (s_accum_k + βs_accum_{k+1} + γ(1 + β))
     * where
     *      f = (w_1 + q_2*w_1_shift) + η(w_2 + q_m*w_2_shift) + η²(w_3 + q_c*w_3_shift) + η³q_index,
     *      t_accum = table_1 + ηtable_2 + η²table_3 + η³table_4, and
     *      s_accum = s_1 + ηs_2 + η²s_3 + η³s_4.
     * Note: Selectors q_2, q_m and q_c are repurposed as 'column step size' for lookup gates.
     *
     * @param evals transformed to `evals + C(in(X)...)*scaling_factor`
     * @param in an std::array containing the fully extended Univariate edges.
     * @param parameters contains beta, gamma, and public_input_delta, ....
     * @param scaling_factor optional term to scale the evaluation before adding to evals.
     */
    template <typename ContainerOverSubrelations, typename AllEntities>
    void static accumulate(ContainerOverSubrelations& accumulators,
                           const AllEntities& in,
                           const RelationParameters<FF>& relation_parameters,
                           const FF& scaling_factor)
    {
        const auto& grand_product_delta = relation_parameters.lookup_grand_product_delta;

        {
            using Accumulator = std::tuple_element_t<0, ContainerOverSubrelations>;
            using View = typename Accumulator::View;

            auto z_lookup = View(in.z_lookup);
            auto z_lookup_shift = View(in.z_lookup_shift);

            auto lagrange_first = View(in.lagrange_first);
            auto lagrange_last = View(in.lagrange_last);

            const auto lhs = compute_grand_product_numerator<Accumulator>(in, relation_parameters);
            const auto rhs = compute_grand_product_denominator<Accumulator>(in, relation_parameters);

            const auto tmp =
                lhs * (z_lookup + lagrange_first) - rhs * (z_lookup_shift + lagrange_last * grand_product_delta);
            std::get<0>(accumulators) += tmp * scaling_factor;
        };

        {
            using Accumulator = std::tuple_element_t<1, ContainerOverSubrelations>;
            using View = typename Accumulator::View;
            auto z_lookup_shift = View(in.z_lookup_shift);
            auto lagrange_last = View(in.lagrange_last);

            // Contribution (2)
            std::get<1>(accumulators) += (lagrange_last * z_lookup_shift) * scaling_factor;
        };
    };
};

template <typename FF> using LookupRelation = Relation<LookupRelationImpl<FF>>;

} // namespace proof_system