#pragma once
#include "relation.hpp"
#include "../polynomials/univariate.hpp"

#pragma GCC diagnostic ignored "-Wunused-variable"
#pragma GCC diagnostic ignored "-Wunused-parameter"

namespace proof_system::honk::sumcheck {

template <typename FF> class LookupGrandProductComputationRelation {
  public:
    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH = 6; // deg(z_lookup * column_selector * wire * q_lookup * table) = 5

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
    inline void add_edge_contribution(Univariate<FF, RELATION_LENGTH>& evals,
                                      const auto& extended_edges,
                                      const RelationParameters<FF>& relation_parameters,
                                      const FF& scaling_factor) const
    {
        const auto& eta = relation_parameters.eta;
        const auto& beta = relation_parameters.beta;
        const auto& gamma = relation_parameters.gamma;
        const auto& grand_product_delta = relation_parameters.lookup_grand_product_delta;

        const auto one_plus_beta = FF::one() + beta;
        const auto gamma_by_one_plus_beta = gamma * one_plus_beta;
        const auto eta_sqr = eta * eta;
        const auto eta_cube = eta_sqr * eta;

        auto w_1 = UnivariateView<FF, RELATION_LENGTH>(extended_edges.w_l);
        auto w_2 = UnivariateView<FF, RELATION_LENGTH>(extended_edges.w_r);
        auto w_3 = UnivariateView<FF, RELATION_LENGTH>(extended_edges.w_o);

        auto w_1_shift = UnivariateView<FF, RELATION_LENGTH>(extended_edges.w_l_shift);
        auto w_2_shift = UnivariateView<FF, RELATION_LENGTH>(extended_edges.w_r_shift);
        auto w_3_shift = UnivariateView<FF, RELATION_LENGTH>(extended_edges.w_o_shift);

        auto table_1 = UnivariateView<FF, RELATION_LENGTH>(extended_edges.table_1);
        auto table_2 = UnivariateView<FF, RELATION_LENGTH>(extended_edges.table_2);
        auto table_3 = UnivariateView<FF, RELATION_LENGTH>(extended_edges.table_3);
        auto table_4 = UnivariateView<FF, RELATION_LENGTH>(extended_edges.table_4);

        auto table_1_shift = UnivariateView<FF, RELATION_LENGTH>(extended_edges.table_1_shift);
        auto table_2_shift = UnivariateView<FF, RELATION_LENGTH>(extended_edges.table_2_shift);
        auto table_3_shift = UnivariateView<FF, RELATION_LENGTH>(extended_edges.table_3_shift);
        auto table_4_shift = UnivariateView<FF, RELATION_LENGTH>(extended_edges.table_4_shift);

        auto s_accum = UnivariateView<FF, RELATION_LENGTH>(extended_edges.sorted_accum);
        auto s_accum_shift = UnivariateView<FF, RELATION_LENGTH>(extended_edges.sorted_accum_shift);

        auto z_lookup = UnivariateView<FF, RELATION_LENGTH>(extended_edges.z_lookup);
        auto z_lookup_shift = UnivariateView<FF, RELATION_LENGTH>(extended_edges.z_lookup_shift);

        auto table_index = UnivariateView<FF, RELATION_LENGTH>(extended_edges.q_o);
        auto column_1_step_size = UnivariateView<FF, RELATION_LENGTH>(extended_edges.q_r);
        auto column_2_step_size = UnivariateView<FF, RELATION_LENGTH>(extended_edges.q_m);
        auto column_3_step_size = UnivariateView<FF, RELATION_LENGTH>(extended_edges.q_c);
        auto q_lookup = UnivariateView<FF, RELATION_LENGTH>(extended_edges.q_lookup);

        auto lagrange_first = UnivariateView<FF, RELATION_LENGTH>(extended_edges.lagrange_first);
        auto lagrange_last = UnivariateView<FF, RELATION_LENGTH>(extended_edges.lagrange_last);

        // (w_1 + q_2*w_1_shift) + η(w_2 + q_m*w_2_shift) + η²(w_3 + q_c*w_3_shift) + η³q_index.
        auto wire_accum = (w_1 + column_1_step_size * w_1_shift) + (w_2 + column_2_step_size * w_2_shift) * eta +
                          (w_3 + column_3_step_size * w_3_shift) * eta_sqr + table_index * eta_cube;

        // t_1 + ηt_2 + η²t_3 + η³t_4
        auto table_accum = table_1 + table_2 * eta + table_3 * eta_sqr + table_4 * eta_cube;
        // t_1_shift + ηt_2_shift + η²t_3_shift + η³t_4_shift
        auto table_accum_shift =
            table_1_shift + table_2_shift * eta + table_3_shift * eta_sqr + table_4_shift * eta_cube;

        // Contribution (1)
        auto tmp = (q_lookup * wire_accum + gamma);
        tmp *= (table_accum + table_accum_shift * beta + gamma_by_one_plus_beta);
        tmp *= one_plus_beta;
        tmp *= (z_lookup + lagrange_first);
        tmp -= (z_lookup_shift + lagrange_last * grand_product_delta) *
               (s_accum + s_accum_shift * beta + gamma_by_one_plus_beta);
        evals += tmp * scaling_factor;
    };

    void add_full_relation_value_contribution(FF& full_honk_relation_value,
                                              auto& purported_evaluations,
                                              const RelationParameters<FF>& relation_parameters) const
    {
        const auto& eta = relation_parameters.eta;
        const auto& beta = relation_parameters.beta;
        const auto& gamma = relation_parameters.gamma;
        const auto& grand_product_delta = relation_parameters.lookup_grand_product_delta;

        const auto one_plus_beta = FF::one() + beta;
        const auto gamma_by_one_plus_beta = gamma * one_plus_beta;
        const auto eta_sqr = eta * eta;
        const auto eta_cube = eta_sqr * eta;

        auto w_1 = purported_evaluations.w_l;
        auto w_2 = purported_evaluations.w_r;
        auto w_3 = purported_evaluations.w_o;

        auto w_1_shift = purported_evaluations.w_l_shift;
        auto w_2_shift = purported_evaluations.w_r_shift;
        auto w_3_shift = purported_evaluations.w_o_shift;

        auto table_1 = purported_evaluations.table_1;
        auto table_2 = purported_evaluations.table_2;
        auto table_3 = purported_evaluations.table_3;
        auto table_4 = purported_evaluations.table_4;

        auto table_1_shift = purported_evaluations.table_1_shift;
        auto table_2_shift = purported_evaluations.table_2_shift;
        auto table_3_shift = purported_evaluations.table_3_shift;
        auto table_4_shift = purported_evaluations.table_4_shift;

        auto s_accum = purported_evaluations.sorted_accum;
        auto s_accum_shift = purported_evaluations.sorted_accum_shift;

        auto z_lookup = purported_evaluations.z_lookup;
        auto z_lookup_shift = purported_evaluations.z_lookup_shift;

        auto table_index = purported_evaluations.q_o;
        auto column_1_step_size = purported_evaluations.q_r;
        auto column_2_step_size = purported_evaluations.q_m;
        auto column_3_step_size = purported_evaluations.q_c;
        auto q_lookup = purported_evaluations.q_lookup;

        auto lagrange_first = purported_evaluations.lagrange_first;
        auto lagrange_last = purported_evaluations.lagrange_last;

        // (w_1 + q_2*w_1_shift) + η(w_2 + q_m*w_2_shift) + η²(w_3 + q_c*w_3_shift) + η³q_index.
        auto wire_accum = (w_1 + column_1_step_size * w_1_shift) + (w_2 + column_2_step_size * w_2_shift) * eta +
                          (w_3 + column_3_step_size * w_3_shift) * eta_sqr + table_index * eta_cube;

        // t_1 + ηt_2 + η²t_3 + η³t_4
        auto table_accum = table_1 + table_2 * eta + table_3 * eta_sqr + table_4 * eta_cube;
        // t_1_shift + ηt_2_shift + η²t_3_shift + η³t_4_shift
        auto table_accum_shift =
            table_1_shift + table_2_shift * eta + table_3_shift * eta_sqr + table_4_shift * eta_cube;

        // Contribution (1)
        auto tmp = (q_lookup * wire_accum + gamma);
        tmp *= (table_accum + beta * table_accum_shift + gamma_by_one_plus_beta);
        tmp *= one_plus_beta;
        tmp *= (z_lookup + lagrange_first);
        tmp -= (z_lookup_shift + lagrange_last * grand_product_delta) *
               (s_accum + beta * s_accum_shift + gamma_by_one_plus_beta);
        full_honk_relation_value += tmp;
    };
};

template <typename FF> class LookupGrandProductInitializationRelation {
  public:
    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH = 3; // deg(lagrange_last * z_lookup_shift) = 2

    /**
     * @brief Compute contribution of the lookup grand prod relation for a given edge (internal function)
     *
     * @details This the relation confirms correct initialization of the lookup grand
     * product polynomial Z_lookup with Z_lookup[circuit_size] = 0.
     *
     * @param evals transformed to `evals + C(extended_edges(X)...)*scaling_factor`
     * @param extended_edges an std::array containing the fully extended Univariate edges.
     * @param parameters contains beta, gamma, and public_input_delta, ....
     * @param scaling_factor optional term to scale the evaluation before adding to evals.
     */
    inline void add_edge_contribution(Univariate<FF, RELATION_LENGTH>& evals,
                                      const auto& extended_edges,
                                      const RelationParameters<FF>& /*unused*/,
                                      const FF& scaling_factor) const
    {
        auto z_lookup_shift = UnivariateView<FF, RELATION_LENGTH>(extended_edges.z_lookup_shift);
        auto lagrange_last = UnivariateView<FF, RELATION_LENGTH>(extended_edges.lagrange_last);

        evals += (lagrange_last * z_lookup_shift) * scaling_factor;
    };

    void add_full_relation_value_contribution(FF& full_honk_relation_value,
                                              auto& purported_evaluations,
                                              const RelationParameters<FF>& /*unused*/) const
    {
        auto z_lookup_shift = purported_evaluations.z_lookup_shift;
        auto lagrange_last = purported_evaluations.lagrange_last;

        full_honk_relation_value += lagrange_last * z_lookup_shift;
    };
};
} // namespace proof_system::honk::sumcheck