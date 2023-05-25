#pragma once
#include "relation_parameters.hpp"
#include "../polynomials/univariate.hpp"
// TODO(luke): change name of this file to permutation_grand_product_relation(s).hpp and move 'init' relation into it.

namespace proof_system::honk::sumcheck {

template <typename FF> class PermutationRelation {
  public:
    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH = 5;

    static constexpr size_t NUM_CONSTRAINTS = 2;
    static constexpr std::array<size_t, NUM_CONSTRAINTS> CONSTRAINT_LENGTH = { 5, 3 };

    using RelationUnivariates = std::tuple<Univariate<FF, CONSTRAINT_LENGTH[0]>, Univariate<FF, CONSTRAINT_LENGTH[1]>>;
    using RelationValues = std::array<FF, NUM_CONSTRAINTS>;

    /**
     * @brief Compute contribution of the permutation relation for a given edge (internal function)
     *
     * @details There are 2 relations associated with enforcing the wire copy relations
     * This file handles the relation that confirms faithful calculation of the grand
     * product polynomial Z_perm. (Initialization relation Z_perm(0) = 1 is handled elsewhere).
     *
     *  C(extended_edges(X)...) =
     *      ( z_perm(X) + lagrange_first(X) )*P(X)
     *         - ( z_perm_shift(X) + delta * lagrange_last(X))*Q(X),
     * where P(X) = Prod_{i=1:3} w_i(X) + β*(n*(i-1) + idx(X)) + γ
     *       Q(X) = Prod_{i=1:3} w_i(X) + β*σ_i(X) + γ
     *
     * @param evals transformed to `evals + C(extended_edges(X)...)*scaling_factor`
     * @param extended_edges an std::array containing the fully extended Univariate edges.
     * @param parameters contains beta, gamma, and public_input_delta, ....
     * @param scaling_factor optional term to scale the evaluation before adding to evals.
     */
    inline void add_edge_contribution(RelationUnivariates& evals,
                                      const auto& extended_edges,
                                      const RelationParameters<FF>& relation_parameters,
                                      const FF& scaling_factor) const
    {
        const auto& beta = relation_parameters.beta;
        const auto& gamma = relation_parameters.gamma;
        const auto& public_input_delta = relation_parameters.public_input_delta;

        {
            static constexpr size_t LENGTH = CONSTRAINT_LENGTH[0];
            auto w_1 = UnivariateView<FF, LENGTH>(extended_edges.w_l);
            auto w_2 = UnivariateView<FF, LENGTH>(extended_edges.w_r);
            auto w_3 = UnivariateView<FF, LENGTH>(extended_edges.w_o);
            auto sigma_1 = UnivariateView<FF, LENGTH>(extended_edges.sigma_1);
            auto sigma_2 = UnivariateView<FF, LENGTH>(extended_edges.sigma_2);
            auto sigma_3 = UnivariateView<FF, LENGTH>(extended_edges.sigma_3);
            auto id_1 = UnivariateView<FF, LENGTH>(extended_edges.id_1);
            auto id_2 = UnivariateView<FF, LENGTH>(extended_edges.id_2);
            auto id_3 = UnivariateView<FF, LENGTH>(extended_edges.id_3);
            auto z_perm = UnivariateView<FF, LENGTH>(extended_edges.z_perm);
            auto z_perm_shift = UnivariateView<FF, LENGTH>(extended_edges.z_perm_shift);
            auto lagrange_first = UnivariateView<FF, LENGTH>(extended_edges.lagrange_first);
            auto lagrange_last = UnivariateView<FF, LENGTH>(extended_edges.lagrange_last);

            // Contribution (1)
            std::get<0>(evals) +=
                (((z_perm + lagrange_first) * (w_1 + id_1 * beta + gamma) * (w_2 + id_2 * beta + gamma) *
                  (w_3 + id_3 * beta + gamma)) -
                 ((z_perm_shift + lagrange_last * public_input_delta) * (w_1 + sigma_1 * beta + gamma) *
                  (w_2 + sigma_2 * beta + gamma) * (w_3 + sigma_3 * beta + gamma))) *
                scaling_factor;
        }
        {
            static constexpr size_t LENGTH = CONSTRAINT_LENGTH[1];
            auto z_perm_shift = UnivariateView<FF, LENGTH>(extended_edges.z_perm_shift);
            auto lagrange_last = UnivariateView<FF, LENGTH>(extended_edges.lagrange_last);

            // Contribution (2)
            std::get<1>(evals) += (lagrange_last * z_perm_shift) * scaling_factor;
        }
    };

    /**
     * @brief Add the result of each identity in this relation evaluated at the multivariate evaluations produced by the
     * Sumcheck Prover.
     *
     * @param full_honk_relation_value
     * @param purported_evaluations
     */
    void add_full_relation_value_contribution(RelationValues& full_honk_relation_value,
                                              auto& purported_evaluations,
                                              const RelationParameters<FF>& relation_parameters) const
    {
        const auto& beta = relation_parameters.beta;
        const auto& gamma = relation_parameters.gamma;
        const auto& public_input_delta = relation_parameters.public_input_delta;

        auto w_1 = purported_evaluations.w_l;
        auto w_2 = purported_evaluations.w_r;
        auto w_3 = purported_evaluations.w_o;
        auto sigma_1 = purported_evaluations.sigma_1;
        auto sigma_2 = purported_evaluations.sigma_2;
        auto sigma_3 = purported_evaluations.sigma_3;
        auto id_1 = purported_evaluations.id_1;
        auto id_2 = purported_evaluations.id_2;
        auto id_3 = purported_evaluations.id_3;
        auto z_perm = purported_evaluations.z_perm;
        auto z_perm_shift = purported_evaluations.z_perm_shift;
        auto lagrange_first = purported_evaluations.lagrange_first;
        auto lagrange_last = purported_evaluations.lagrange_last;

        // Contribution (1)
        std::get<0>(full_honk_relation_value) +=
            ((z_perm + lagrange_first) * (w_1 + beta * id_1 + gamma) * (w_2 + beta * id_2 + gamma) *
                 (w_3 + beta * id_3 + gamma) -
             (z_perm_shift + lagrange_last * public_input_delta) * (w_1 + beta * sigma_1 + gamma) *
                 (w_2 + beta * sigma_2 + gamma) * (w_3 + beta * sigma_3 + gamma));

        // Contribution (2)
        std::get<1>(full_honk_relation_value) += lagrange_last * z_perm_shift;
    };
};

// TODO(luke): With Cody's Flavor work it should be easier to create a simple templated relation
// for handling arbitrary width. For now I'm duplicating the width 3 logic for width 4.
template <typename FF> class UltraPermutationRelation {
  public:
    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH = 6;

    static constexpr size_t NUM_CONSTRAINTS = 2;
    static constexpr std::array<size_t, NUM_CONSTRAINTS> CONSTRAINT_LENGTH = { 6, 3 };

    using RelationUnivariates = std::tuple<Univariate<FF, CONSTRAINT_LENGTH[0]>, Univariate<FF, CONSTRAINT_LENGTH[1]>>;
    using RelationValues = std::array<FF, NUM_CONSTRAINTS>;

    /**
     * @brief Compute contribution of the permutation relation for a given edge (internal function)
     *
     * @details This the relation confirms faithful calculation of the grand
     * product polynomial Z_perm.
     *
     * @param evals transformed to `evals + C(extended_edges(X)...)*scaling_factor`
     * @param extended_edges an std::array containing the fully extended Univariate edges.
     * @param parameters contains beta, gamma, and public_input_delta, ....
     * @param scaling_factor optional term to scale the evaluation before adding to evals.
     */
    inline void add_edge_contribution(RelationUnivariates& evals,
                                      const auto& extended_edges,
                                      const RelationParameters<FF>& relation_parameters,
                                      const FF& scaling_factor) const
    {
        const auto& beta = relation_parameters.beta;
        const auto& gamma = relation_parameters.gamma;
        const auto& public_input_delta = relation_parameters.public_input_delta;

        // Contribution (1)
        {
            static constexpr size_t LENGTH = CONSTRAINT_LENGTH[0];
            auto w_1 = UnivariateView<FF, LENGTH>(extended_edges.w_l);
            auto w_2 = UnivariateView<FF, LENGTH>(extended_edges.w_r);
            auto w_3 = UnivariateView<FF, LENGTH>(extended_edges.w_o);
            auto w_4 = UnivariateView<FF, LENGTH>(extended_edges.w_4);
            auto sigma_1 = UnivariateView<FF, LENGTH>(extended_edges.sigma_1);
            auto sigma_2 = UnivariateView<FF, LENGTH>(extended_edges.sigma_2);
            auto sigma_3 = UnivariateView<FF, LENGTH>(extended_edges.sigma_3);
            auto sigma_4 = UnivariateView<FF, LENGTH>(extended_edges.sigma_4);
            auto id_1 = UnivariateView<FF, LENGTH>(extended_edges.id_1);
            auto id_2 = UnivariateView<FF, LENGTH>(extended_edges.id_2);
            auto id_3 = UnivariateView<FF, LENGTH>(extended_edges.id_3);
            auto id_4 = UnivariateView<FF, LENGTH>(extended_edges.id_4);
            auto z_perm = UnivariateView<FF, LENGTH>(extended_edges.z_perm);
            auto z_perm_shift = UnivariateView<FF, LENGTH>(extended_edges.z_perm_shift);
            auto lagrange_first = UnivariateView<FF, LENGTH>(extended_edges.lagrange_first);
            auto lagrange_last = UnivariateView<FF, LENGTH>(extended_edges.lagrange_last);

            std::get<0>(evals) +=
                (((z_perm + lagrange_first) * (w_1 + id_1 * beta + gamma) * (w_2 + id_2 * beta + gamma) *
                  (w_3 + id_3 * beta + gamma) * (w_4 + id_4 * beta + gamma)) -
                 ((z_perm_shift + lagrange_last * public_input_delta) * (w_1 + sigma_1 * beta + gamma) *
                  (w_2 + sigma_2 * beta + gamma) * (w_3 + sigma_3 * beta + gamma) * (w_4 + sigma_4 * beta + gamma))) *
                scaling_factor;
        }
        // Contribution (2)
        {
            static constexpr size_t LENGTH = CONSTRAINT_LENGTH[1];
            auto z_perm_shift = UnivariateView<FF, LENGTH>(extended_edges.z_perm_shift);
            auto lagrange_last = UnivariateView<FF, LENGTH>(extended_edges.lagrange_last);

            std::get<1>(evals) += (lagrange_last * z_perm_shift) * scaling_factor;
        }
    };

    /**
     * @brief Add the result of each identity in this relation evaluated at the multivariate evaluations produced by the
     * Sumcheck Prover.
     *
     * @param full_honk_relation_value
     * @param purported_evaluations
     */
    void add_full_relation_value_contribution(RelationValues& full_honk_relation_value,
                                              auto& purported_evaluations,
                                              const RelationParameters<FF>& relation_parameters) const
    {
        const auto& beta = relation_parameters.beta;
        const auto& gamma = relation_parameters.gamma;
        const auto& public_input_delta = relation_parameters.public_input_delta;

        auto w_1 = purported_evaluations.w_l;
        auto w_2 = purported_evaluations.w_r;
        auto w_3 = purported_evaluations.w_o;
        auto w_4 = purported_evaluations.w_4;
        auto sigma_1 = purported_evaluations.sigma_1;
        auto sigma_2 = purported_evaluations.sigma_2;
        auto sigma_3 = purported_evaluations.sigma_3;
        auto sigma_4 = purported_evaluations.sigma_4;
        auto id_1 = purported_evaluations.id_1;
        auto id_2 = purported_evaluations.id_2;
        auto id_3 = purported_evaluations.id_3;
        auto id_4 = purported_evaluations.id_4;
        auto z_perm = purported_evaluations.z_perm;
        auto z_perm_shift = purported_evaluations.z_perm_shift;
        auto lagrange_first = purported_evaluations.lagrange_first;
        auto lagrange_last = purported_evaluations.lagrange_last;

        // Contribution (1)
        std::get<0>(full_honk_relation_value) +=
            ((z_perm + lagrange_first) * (w_1 + beta * id_1 + gamma) * (w_2 + beta * id_2 + gamma) *
                 (w_3 + beta * id_3 + gamma) * (w_4 + beta * id_4 + gamma) -
             (z_perm_shift + lagrange_last * public_input_delta) * (w_1 + beta * sigma_1 + gamma) *
                 (w_2 + beta * sigma_2 + gamma) * (w_3 + beta * sigma_3 + gamma) * (w_4 + beta * sigma_4 + gamma));

        // Contribution (2)
        std::get<1>(full_honk_relation_value) += lagrange_last * z_perm_shift;
    };
};
} // namespace proof_system::honk::sumcheck
