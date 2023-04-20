#pragma once
#include "relation.hpp"
#include "barretenberg/honk/flavor/flavor.hpp"
#include "../polynomials/univariate.hpp"
// TODO(luke): change name of this file to permutation_grand_product_relation(s).hpp and move 'init' relation into it.

namespace proof_system::honk::sumcheck {

template <typename FF> class GrandProductComputationRelation {
  public:
    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH = 5;
    using MULTIVARIATE = StandardHonk::MULTIVARIATE;

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
    inline void add_edge_contribution(Univariate<FF, RELATION_LENGTH>& evals,
                                      const auto& extended_edges,
                                      const RelationParameters<FF>& relation_parameters,
                                      const FF& scaling_factor) const
    {
        const auto& beta = relation_parameters.beta;
        const auto& gamma = relation_parameters.gamma;
        const auto& public_input_delta = relation_parameters.public_input_delta;

        auto w_1 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::W_L]);
        auto w_2 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::W_R]);
        auto w_3 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::W_O]);
        auto sigma_1 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::SIGMA_1]);
        auto sigma_2 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::SIGMA_2]);
        auto sigma_3 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::SIGMA_3]);
        auto id_1 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::ID_1]);
        auto id_2 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::ID_2]);
        auto id_3 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::ID_3]);
        auto z_perm = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::Z_PERM]);
        auto z_perm_shift = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::Z_PERM_SHIFT]);
        auto lagrange_first = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::LAGRANGE_FIRST]);
        auto lagrange_last = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::LAGRANGE_LAST]);

        // Contribution (1)
        evals += (((z_perm + lagrange_first) * (w_1 + id_1 * beta + gamma) * (w_2 + id_2 * beta + gamma) *
                   (w_3 + id_3 * beta + gamma)) -
                  ((z_perm_shift + lagrange_last * public_input_delta) * (w_1 + sigma_1 * beta + gamma) *
                   (w_2 + sigma_2 * beta + gamma) * (w_3 + sigma_3 * beta + gamma))) *
                 scaling_factor;
    };

    void add_full_relation_value_contribution(FF& full_honk_relation_value,
                                              auto& purported_evaluations,
                                              const RelationParameters<FF>& relation_parameters) const
    {
        const auto& beta = relation_parameters.beta;
        const auto& gamma = relation_parameters.gamma;
        const auto& public_input_delta = relation_parameters.public_input_delta;

        auto w_1 = purported_evaluations[MULTIVARIATE::W_L];
        auto w_2 = purported_evaluations[MULTIVARIATE::W_R];
        auto w_3 = purported_evaluations[MULTIVARIATE::W_O];
        auto sigma_1 = purported_evaluations[MULTIVARIATE::SIGMA_1];
        auto sigma_2 = purported_evaluations[MULTIVARIATE::SIGMA_2];
        auto sigma_3 = purported_evaluations[MULTIVARIATE::SIGMA_3];
        auto id_1 = purported_evaluations[MULTIVARIATE::ID_1];
        auto id_2 = purported_evaluations[MULTIVARIATE::ID_2];
        auto id_3 = purported_evaluations[MULTIVARIATE::ID_3];
        auto z_perm = purported_evaluations[MULTIVARIATE::Z_PERM];
        auto z_perm_shift = purported_evaluations[MULTIVARIATE::Z_PERM_SHIFT];
        auto lagrange_first = purported_evaluations[MULTIVARIATE::LAGRANGE_FIRST];
        auto lagrange_last = purported_evaluations[MULTIVARIATE::LAGRANGE_LAST];

        // Contribution (1)
        full_honk_relation_value +=
            ((z_perm + lagrange_first) * (w_1 + beta * id_1 + gamma) * (w_2 + beta * id_2 + gamma) *
                 (w_3 + beta * id_3 + gamma) -
             (z_perm_shift + lagrange_last * public_input_delta) * (w_1 + beta * sigma_1 + gamma) *
                 (w_2 + beta * sigma_2 + gamma) * (w_3 + beta * sigma_3 + gamma));
    };
};

// TODO(luke): With Cody's Flavor work it should be easier to create a simple templated relation
// for handling arbitrary width. For now I'm duplicating the width 3 logic for width 4.
template <typename FF> class UltraGrandProductComputationRelation {
  public:
    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH = 6;
    using MULTIVARIATE = proof_system::honk::UltraArithmetization::POLYNOMIAL;

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
    inline void add_edge_contribution(Univariate<FF, RELATION_LENGTH>& evals,
                                      const auto& extended_edges,
                                      const RelationParameters<FF>& relation_parameters,
                                      const FF& scaling_factor) const
    {
        const auto& beta = relation_parameters.beta;
        const auto& gamma = relation_parameters.gamma;
        const auto& public_input_delta = relation_parameters.public_input_delta;

        auto w_1 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::W_L]);
        auto w_2 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::W_R]);
        auto w_3 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::W_O]);
        auto w_4 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::W_4]);
        auto sigma_1 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::SIGMA_1]);
        auto sigma_2 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::SIGMA_2]);
        auto sigma_3 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::SIGMA_3]);
        auto sigma_4 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::SIGMA_4]);
        auto id_1 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::ID_1]);
        auto id_2 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::ID_2]);
        auto id_3 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::ID_3]);
        auto id_4 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::ID_4]);
        auto z_perm = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::Z_PERM]);
        auto z_perm_shift = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::Z_PERM_SHIFT]);
        auto lagrange_first = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::LAGRANGE_FIRST]);
        auto lagrange_last = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::LAGRANGE_LAST]);

        // Contribution (1)
        evals += (((z_perm + lagrange_first) * (w_1 + id_1 * beta + gamma) * (w_2 + id_2 * beta + gamma) *
                   (w_3 + id_3 * beta + gamma) * (w_4 + id_4 * beta + gamma)) -
                  ((z_perm_shift + lagrange_last * public_input_delta) * (w_1 + sigma_1 * beta + gamma) *
                   (w_2 + sigma_2 * beta + gamma) * (w_3 + sigma_3 * beta + gamma) * (w_4 + sigma_4 * beta + gamma))) *
                 scaling_factor;
    };

    void add_full_relation_value_contribution(FF& full_honk_relation_value,
                                              auto& purported_evaluations,
                                              const RelationParameters<FF>& relation_parameters) const
    {
        const auto& beta = relation_parameters.beta;
        const auto& gamma = relation_parameters.gamma;
        const auto& public_input_delta = relation_parameters.public_input_delta;

        auto w_1 = purported_evaluations[MULTIVARIATE::W_L];
        auto w_2 = purported_evaluations[MULTIVARIATE::W_R];
        auto w_3 = purported_evaluations[MULTIVARIATE::W_O];
        auto w_4 = purported_evaluations[MULTIVARIATE::W_4];
        auto sigma_1 = purported_evaluations[MULTIVARIATE::SIGMA_1];
        auto sigma_2 = purported_evaluations[MULTIVARIATE::SIGMA_2];
        auto sigma_3 = purported_evaluations[MULTIVARIATE::SIGMA_3];
        auto sigma_4 = purported_evaluations[MULTIVARIATE::SIGMA_4];
        auto id_1 = purported_evaluations[MULTIVARIATE::ID_1];
        auto id_2 = purported_evaluations[MULTIVARIATE::ID_2];
        auto id_3 = purported_evaluations[MULTIVARIATE::ID_3];
        auto id_4 = purported_evaluations[MULTIVARIATE::ID_4];
        auto z_perm = purported_evaluations[MULTIVARIATE::Z_PERM];
        auto z_perm_shift = purported_evaluations[MULTIVARIATE::Z_PERM_SHIFT];
        auto lagrange_first = purported_evaluations[MULTIVARIATE::LAGRANGE_FIRST];
        auto lagrange_last = purported_evaluations[MULTIVARIATE::LAGRANGE_LAST];

        // Contribution (1)
        full_honk_relation_value +=
            ((z_perm + lagrange_first) * (w_1 + beta * id_1 + gamma) * (w_2 + beta * id_2 + gamma) *
                 (w_3 + beta * id_3 + gamma) * (w_4 + beta * id_4 + gamma) -
             (z_perm_shift + lagrange_last * public_input_delta) * (w_1 + beta * sigma_1 + gamma) *
                 (w_2 + beta * sigma_2 + gamma) * (w_3 + beta * sigma_3 + gamma) * (w_4 + beta * sigma_4 + gamma));
    };
};
} // namespace proof_system::honk::sumcheck
