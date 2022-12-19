#include "./constraint.hpp"
#include "./multivariates.hpp"
#include "./univariate.hpp"
#include "./barycentric_data.hpp"

#pragma GCC diagnostic ignored "-Wunused-variable"
#pragma GCC diagnostic ignored "-Wunused-parameter"

namespace honk::sumcheck {

template <typename Fr> class GrandProductInitializationConstraint : public Constraint<Fr> {
  public:
    static constexpr size_t CONSTRAINT_LENGTH = 3; // degree of this constraint + 1
    using Constraint<Fr>::HONK_CONSTRAINT_LENGTH;
    using Constraint<Fr>::NUM_HONK_MULTIVARIATES;
    BarycentricData<Fr, CONSTRAINT_LENGTH, HONK_CONSTRAINT_LENGTH> barycentric =
        BarycentricData<Fr, CONSTRAINT_LENGTH, HONK_CONSTRAINT_LENGTH>();
    using UnivariateClass = Univariate<Fr, CONSTRAINT_LENGTH>;
    // using UnivariateView = UnivariateView<Fr, CONSTRAINT_LENGTH>;

  public:
    GrandProductInitializationConstraint() = default;

    /**
     * @brief Add contribution of the permutation constraint for a given edge
     *
     * @detail There are 2 constraints associated with enforcing the wire copy constraints
     * This file handles the constraint Z_perm(0) = 1 via the constraint:
     *
     *                      C(X) = L_1(X)(z_perm(X) - 1)
     */
    void add_edge_contribution(auto& edge_extensions, UnivariateClass& evals)
    {
        auto z_perm = UnivariateView<Fr, CONSTRAINT_LENGTH>(edge_extensions[MULTIVARIATE::Z_PERM]);
        auto lagrange_1 = UnivariateView<Fr, CONSTRAINT_LENGTH>(edge_extensions[MULTIVARIATE::LAGRANGE_1]);
        auto one = Fr(1);

        evals += lagrange_1 * (z_perm - one);
    };

    void add_full_constraint_value_contribution(std::array<Fr, NUM_HONK_MULTIVARIATES> purported_evaluations,
                                                Fr& full_honk_constraint_value)
    {
        auto z_perm = purported_evaluations[MULTIVARIATE::Z_PERM];
        auto lagrange_1 = purported_evaluations[MULTIVARIATE::LAGRANGE_1];
        auto one = Fr(1);

        full_honk_constraint_value += lagrange_1 * (z_perm - one);
    };
};
} // namespace honk::sumcheck
