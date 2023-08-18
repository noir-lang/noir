#pragma once
#include "../polynomials/univariate.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "relation_parameters.hpp"
#include "relation_types.hpp"

namespace proof_system::honk::sumcheck {

/**
 * @brief LookupRelationBase defines the algebra for the lookup polynomial:
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
template <typename FF> class LookupRelationBase {
  public:
    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH = 6; // deg(z_lookup * column_selector * wire * q_lookup * table) = 5

    static constexpr size_t LEN_1 = 6; // grand product construction sub-relation
    static constexpr size_t LEN_2 = 3; // left-shiftable polynomial sub-relation
    template <template <size_t...> typename AccumulatorTypesContainer>
    using AccumulatorTypesBase = AccumulatorTypesContainer<LEN_1, LEN_2>;
    template <typename T> using Accumulator = typename std::tuple_element<0, typename T::Accumulators>::type;

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
     * @param extended_edges
     * @param relation_parameters
     * @param index If calling this method over vector inputs, index >= 0
     * @return Accumulator<AccumulatorTypes> either Univariate or FF depending on context
     */
    template <typename AccumulatorTypes>
    inline static Accumulator<AccumulatorTypes> compute_grand_product_numerator(
        const auto& extended_edges, const RelationParameters<FF>& relation_parameters, const size_t index)
    {
        const auto& beta = relation_parameters.beta;
        const auto& gamma = relation_parameters.gamma;
        const auto& eta = relation_parameters.eta;
        const auto eta_sqr = eta * eta;
        const auto eta_cube = eta_sqr * eta;

        const auto one_plus_beta = FF(1) + beta;
        const auto gamma_by_one_plus_beta = gamma * one_plus_beta;

        auto w_1 = get_view<FF, AccumulatorTypes>(extended_edges.w_l, index);
        auto w_2 = get_view<FF, AccumulatorTypes>(extended_edges.w_r, index);
        auto w_3 = get_view<FF, AccumulatorTypes>(extended_edges.w_o, index);

        auto w_1_shift = get_view<FF, AccumulatorTypes>(extended_edges.w_l_shift, index);
        auto w_2_shift = get_view<FF, AccumulatorTypes>(extended_edges.w_r_shift, index);
        auto w_3_shift = get_view<FF, AccumulatorTypes>(extended_edges.w_o_shift, index);

        auto table_1 = get_view<FF, AccumulatorTypes>(extended_edges.table_1, index);
        auto table_2 = get_view<FF, AccumulatorTypes>(extended_edges.table_2, index);
        auto table_3 = get_view<FF, AccumulatorTypes>(extended_edges.table_3, index);
        auto table_4 = get_view<FF, AccumulatorTypes>(extended_edges.table_4, index);

        auto table_1_shift = get_view<FF, AccumulatorTypes>(extended_edges.table_1_shift, index);
        auto table_2_shift = get_view<FF, AccumulatorTypes>(extended_edges.table_2_shift, index);
        auto table_3_shift = get_view<FF, AccumulatorTypes>(extended_edges.table_3_shift, index);
        auto table_4_shift = get_view<FF, AccumulatorTypes>(extended_edges.table_4_shift, index);

        auto table_index = get_view<FF, AccumulatorTypes>(extended_edges.q_o, index);
        auto column_1_step_size = get_view<FF, AccumulatorTypes>(extended_edges.q_r, index);
        auto column_2_step_size = get_view<FF, AccumulatorTypes>(extended_edges.q_m, index);
        auto column_3_step_size = get_view<FF, AccumulatorTypes>(extended_edges.q_c, index);
        auto q_lookup = get_view<FF, AccumulatorTypes>(extended_edges.q_lookup, index);

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
     * @param extended_edges
     * @param relation_parameters
     * @param index
     * @return Accumulator<AccumulatorTypes> either Univariate or FF depending on context
     */
    template <typename AccumulatorTypes>
    inline static Accumulator<AccumulatorTypes> compute_grand_product_denominator(
        const auto& extended_edges, const RelationParameters<FF>& relation_parameters, const size_t index)
    {
        const auto& beta = relation_parameters.beta;
        const auto& gamma = relation_parameters.gamma;

        const auto one_plus_beta = FF(1) + beta;
        const auto gamma_by_one_plus_beta = gamma * one_plus_beta;

        // Contribution (1)
        auto s_accum = get_view<FF, AccumulatorTypes>(extended_edges.sorted_accum, index);
        auto s_accum_shift = get_view<FF, AccumulatorTypes>(extended_edges.sorted_accum_shift, index);

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
     * @param evals transformed to `evals + C(extended_edges(X)...)*scaling_factor`
     * @param extended_edges an std::array containing the fully extended Univariate edges.
     * @param parameters contains beta, gamma, and public_input_delta, ....
     * @param scaling_factor optional term to scale the evaluation before adding to evals.
     */
    template <typename AccumulatorTypes>
    inline static void add_edge_contribution_impl(typename AccumulatorTypes::Accumulators& accumulators,
                                                  const auto& extended_edges,
                                                  const RelationParameters<FF>& relation_parameters,
                                                  const FF& scaling_factor)
    {
        const auto& grand_product_delta = relation_parameters.lookup_grand_product_delta;

        // Contribution (1)
        {
            using View = typename std::tuple_element<0, typename AccumulatorTypes::AccumulatorViews>::type;

            auto z_lookup = View(extended_edges.z_lookup);
            auto z_lookup_shift = View(extended_edges.z_lookup_shift);

            auto lagrange_first = View(extended_edges.lagrange_first);
            auto lagrange_last = View(extended_edges.lagrange_last);

            const auto lhs = compute_grand_product_numerator<AccumulatorTypes>(extended_edges, relation_parameters, 0);
            const auto rhs =
                compute_grand_product_denominator<AccumulatorTypes>(extended_edges, relation_parameters, 0);

            const auto tmp =
                lhs * (z_lookup + lagrange_first) - rhs * (z_lookup_shift + lagrange_last * grand_product_delta);
            std::get<0>(accumulators) += tmp * scaling_factor;
        }
        {
            using View = typename std::tuple_element<1, typename AccumulatorTypes::AccumulatorViews>::type;
            auto z_lookup_shift = View(extended_edges.z_lookup_shift);
            auto lagrange_last = View(extended_edges.lagrange_last);

            // Contribution (2)
            std::get<1>(accumulators) += (lagrange_last * z_lookup_shift) * scaling_factor;
        }
    };
};

template <typename FF> using LookupRelation = RelationWrapper<FF, LookupRelationBase>;

} // namespace proof_system::honk::sumcheck