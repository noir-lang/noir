#pragma once
#include "relation.hpp"
#include <proof_system/flavor/flavor.hpp>
#include "../polynomials/univariate.hpp"

namespace honk::sumcheck {

template <typename FF> class GrandProductInitializationRelation : public Relation<FF> {
  public:
    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH = 3;
    using MULTIVARIATE = StandardHonk::MULTIVARIATE; // could just get from StandardArithmetization

    GrandProductInitializationRelation() = default;
    explicit GrandProductInitializationRelation(auto){}; // NOLINT(readability-named-parameter)

    /**
     * @brief Add contribution of the permutation relation for a given edge
     *
     * @detail There are 2 relations associated with enforcing the wire copy relations
     * This file handles the relation Z_perm_shift(n_last) = 0 via the relation:
     *
     *                      C(X) = L_LAST(X) * Z_perm_shift(X)
     */
    void add_edge_contribution(auto& extended_edges, Univariate<FF, RELATION_LENGTH>& evals)
    {
        add_edge_contribution_internal(extended_edges, evals);
    };

    /**
     * @brief Internal function computing the actual contribution for GP intialization relation
     *
     * @param extended_edges
     * @param evals
     */
    void add_edge_contribution_internal(auto& extended_edges, Univariate<FF, RELATION_LENGTH>& evals)
    {
        auto z_perm_shift = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::Z_PERM_SHIFT]);
        auto lagrange_last = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::LAGRANGE_LAST]);

        evals += lagrange_last * z_perm_shift;
    }
    /**
     * @brief A version of `add_edge_contribution` used for testing the relation
     *
     * @tparam T
     * @param extended_edges
     * @param evals
     * @param challenges
     */
    // TODO(kesha): Change once challenges are being supplied to regular contribution
    template <typename T>
    void add_edge_contribution_testing(auto& extended_edges, Univariate<FF, RELATION_LENGTH>& evals, T)
    {
        add_edge_contribution_internal(extended_edges, evals);
    }

    void add_full_relation_value_contribution(auto& purported_evaluations, FF& full_honk_relation_value)
    {
        auto z_perm_shift = purported_evaluations[MULTIVARIATE::Z_PERM_SHIFT];
        auto lagrange_last = purported_evaluations[MULTIVARIATE::LAGRANGE_LAST];

        full_honk_relation_value += lagrange_last * z_perm_shift;
    };
};
} // namespace honk::sumcheck
