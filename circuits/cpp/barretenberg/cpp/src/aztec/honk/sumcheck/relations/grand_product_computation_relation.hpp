#include "relation.hpp"
#include "../../flavor/flavor.hpp"
#include "../polynomials/multivariates.hpp" // TODO(Cody): don't need?
#include "../polynomials/univariate.hpp"
#include "../polynomials/barycentric_data.hpp"
#include "../challenge_container.hpp"

#pragma GCC diagnostic ignored "-Wunused-variable"
#pragma GCC diagnostic ignored "-Wunused-parameter"

namespace honk::sumcheck {

template <typename FF> class GrandProductComputationRelation : public Relation<FF> {
  public:
    // 1 + polynomial degree of this relation
    static constexpr size_t RELATION_LENGTH = 5;
    using MULTIVARIATE = StandardHonk::MULTIVARIATE;

  public:
    const FF beta;
    const FF gamma;

    explicit GrandProductComputationRelation(auto& challenge_container)
        : beta(challenge_container.get_grand_product_beta_challenge())
        , gamma(challenge_container.get_grand_product_gamma_challenge()){};

    /**
     * @brief Add contribution of the permutation relation for a given edge
     *
     * @detail There are 2 relations associated with enforcing the wire copy relations
     * This file handles the relation that confirms faithful calculation of the grand
     * product polynomial Z_perm. (Initialization relation Z_perm(0) = 1 is handled elsewhere).
     *
     *      z_perm(X)*P(X) - z_perm_shift(X)*Q(X), where
     *      P(X) = Prod_{i=1:3} w_i(X) + β*(n*(i-1) + idx(X)) + γ
     *      Q(X) = Prod_{i=1:3} w_i(X) + β*σ_i(X) + γ
     *
     */
    void add_edge_contribution(auto& extended_edges, Univariate<FF, RELATION_LENGTH>& evals)
    {
        auto w_1 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::W_L]);
        auto w_2 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::W_R]);
        auto w_3 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::W_O]);
        auto sigma_1 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::SIGMA_1]);
        auto sigma_2 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::SIGMA_2]);
        auto sigma_3 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::SIGMA_3]);
        auto id_1 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::ID_1]);
        auto id_2 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::ID_1]);
        auto id_3 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::ID_1]);
        auto z_perm = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::Z_PERM]);
        auto z_perm_shift = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::Z_PERM_SHIFT]);
        // auto lagrange_1 = UnivariateView<FF, RELATION_LENGTH>(extended_edges[MULTIVARIATE::LAGRANGE_1]);

        // Contribution (1)
        evals += z_perm;
        evals *= w_1 + id_1 * beta + gamma;
        evals *= w_2 + id_2 * beta + gamma;
        evals *= w_3 + id_3 * beta + gamma;
        evals -= z_perm_shift * (w_1 + sigma_1 * beta + gamma) * (w_2 + sigma_2 * beta + gamma) *
                 (w_3 + sigma_3 * beta + gamma);
    };

    void add_full_relation_value_contribution(auto& purported_evaluations, FF& full_honk_relation_value)
    {
        FF beta = 1; // to be obtained from transcript
        FF gamma = 1;

        auto w_1 = purported_evaluations[MULTIVARIATE::W_L];
        auto w_2 = purported_evaluations[MULTIVARIATE::W_R];
        auto w_3 = purported_evaluations[MULTIVARIATE::W_O];
        auto sigma_1 = purported_evaluations[MULTIVARIATE::SIGMA_1];
        auto sigma_2 = purported_evaluations[MULTIVARIATE::SIGMA_2];
        auto sigma_3 = purported_evaluations[MULTIVARIATE::SIGMA_3];
        auto id_1 = purported_evaluations[MULTIVARIATE::ID_1];
        auto id_2 = purported_evaluations[MULTIVARIATE::ID_1];
        auto id_3 = purported_evaluations[MULTIVARIATE::ID_1];
        auto z_perm = purported_evaluations[MULTIVARIATE::Z_PERM];
        auto z_perm_shift = purported_evaluations[MULTIVARIATE::Z_PERM_SHIFT];
        // auto lagrange_1 = purported_evaluations[MULTIVARIATE::LAGRANGE_1];

        // Contribution (1)
        full_honk_relation_value += z_perm;
        full_honk_relation_value *= w_1 + beta * id_1 + gamma;
        full_honk_relation_value *= w_2 + beta * id_2 + gamma;
        full_honk_relation_value *= w_3 + beta * id_3 + gamma;
        full_honk_relation_value -= z_perm_shift * (w_1 + beta * sigma_1 + gamma) * (w_2 + beta * sigma_2 + gamma) *
                                    (w_3 + beta * sigma_3 + gamma);
    };

    /* ********* ********* ********* ********* ********* ********* ********* ********* ********* ********* */

    // TODO(luke): This function probably doesn't belong in this class. It could be moved to e.g. the composer
    /**
     * @brief Compute the permutation grand product polynomial Z_perm(X)
     * *
     * @detail (This description assumes program_width 3). Z_perm may be defined in terms of its values
     * on X_i = 0,1,...,n-1 as Z_perm[0] = 1 and for i = 1:n-1
     *
     *                  (w_1(j) + β⋅id_1(j) + γ) ⋅ (w_2(j) + β⋅id_2(j) + γ) ⋅ (w_3(j) + β⋅id_3(j) + γ)
     * Z_perm[i] = ∏ --------------------------------------------------------------------------------
     *                  (w_1(j) + β⋅σ_1(j) + γ) ⋅ (w_2(j) + β⋅σ_2(j) + γ) ⋅ (w_3(j) + β⋅σ_3(j) + γ)
     *
     * where ∏ := ∏_{j=0:i-1} and id_i(X) = id(X) + n*(i-1). These evaluations are constructed over the
     * course of three steps. For expositional simplicity, write Z_perm[i] as
     *
     *                A_1(j) ⋅ A_2(j) ⋅ A_3(j)
     * Z_perm[i] = ∏ --------------------------
     *                B_1(j) ⋅ B_2(j) ⋅ B_3(j)
     *
     * Step 1) Compute the 2*program_width length-n polynomials A_i and B_i
     * Step 2) Compute the 2*program_width length-n polynomials ∏ A_i(j) and ∏ B_i(j)
     * Step 3) Compute the two length-n polynomials defined by
     *          numer[i] = ∏ A_1(j)⋅A_2(j)⋅A_3(j), and denom[i] = ∏ B_1(j)⋅B_2(j)⋅B_3(j)
     * Step 4) Compute Z_perm[i+1] = numer[i]/denom[i] (recall: Z_perm[0] = 1)
     *
     * Note: Step (4) utilizes Montgomery batch inversion to replace n-many inversions with
     * one batch inversion (at the expense of more multiplications)
     */
    // template <size_t program_width>
    void compute_grand_product_polynomial_z(/*transcript::StandardTranscript& transcript*/)
    {
        const size_t program_width = 3; // eventually a template param?
        size_t key_n = 100;             // temp placeholder to get things building

        // Allocate scratch space for accumulators
        FF* numererator_accum[program_width];
        FF* denominator_accum[program_width];
        for (size_t i = 0; i < program_width; ++i) {
            // TODO(Cody): clang-tidy is not happy with these lines.
            numererator_accum[i] = static_cast<FF*>(aligned_alloc(64, sizeof(FF) * key_n));
            denominator_accum[i] = static_cast<FF*>(aligned_alloc(64, sizeof(FF) * key_n));
        }

        // Popoulate wire and permutation polynomials
        std::array<const FF*, program_width> wires;
        std::array<const FF*, program_width> sigmas;
        for (size_t i = 0; i < program_width; ++i) {
            std::string wire_id = "w_" + std::to_string(i + 1) + "_lagrange";
            std::string sigma_id = "sigma_" + std::to_string(i + 1) + "_lagrange";
            // wires[i] = key->polynomial_cache.get(wire_id).get_coefficients();
            // sigmas[i] = key->polynomial_cache.get(sigma_id).get_coefficients();
        }

        // Get random challenges (to be obtained from transcript)
        FF beta = FF::random_element();
        FF gamma = FF::random_element();

        // Step (1)
        for (size_t i = 0; i < key_n; ++i) {
            for (size_t k = 0; k < program_width; ++k) {
                // TODO(luke): maybe this idx is replaced by proper ID polys in the future
                FF idx = k * key_n + i;
                numererator_accum[k][i] = wires[k][i] + (idx * beta) + gamma;          // w_k(i) + β.(k*n+i) + γ
                denominator_accum[k][i] = wires[k][i] + (sigmas[k][i] * beta) + gamma; // w_k(i) + β.σ_k(i) + γ
            }
        }

        // Step (2)
        for (size_t k = 0; k < program_width; ++k) {
            for (size_t i = 0; i < key_n - 1; ++i) {
                numererator_accum[k][i + 1] *= numererator_accum[k][i];
                denominator_accum[k][i + 1] *= denominator_accum[k][i];
            }
        }

        // Step (3)
        for (size_t i = 0; i < key_n; ++i) {
            for (size_t k = 1; k < program_width; ++k) {
                numererator_accum[0][i] *= numererator_accum[k][i];
                denominator_accum[0][i] *= denominator_accum[k][i];
            }
        }

        // Step (4)
        // Use Montgomery batch inversion to compute z_perm[i+1] = numererator_accum[0][i] / denominator_accum[0][i]
        FF* inversion_coefficients = &denominator_accum[1][0]; // arbitrary scratch space
        FF inversion_accumulator = FF::one();
        for (size_t i = 0; i < key_n; ++i) {
            inversion_coefficients[i] = numererator_accum[0][i] * inversion_accumulator;
            inversion_accumulator *= denominator_accum[0][i];
        }

        inversion_accumulator = inversion_accumulator.invert(); // perform single inversion per thread
        for (size_t i = key_n - 1; i != -1; --i) {
            // TODO(luke): What needs to be done Re the comment below:
            // We can avoid fully reducing z_perm[i + 1] as the inverse fft will take care of that for us
            numererator_accum[0][i] = inversion_accumulator * inversion_coefficients[i];
            inversion_accumulator *= denominator_accum[0][i];
        }

        // Construct permutation polynomial 'z_perm' in lagrange form as:
        // z_perm = [1 numererator_accum[0][0] numererator_accum[0][1] ... numererator_accum[0][n-2]]
        // polynomial z_perm(key_n, key_n);
        // z_perm[0] = FF::one();
        // barretenberg::polynomial_arithmetic::copy_polynomial(numererator_accum[0], &z_perm[1], key_n - 1, key_n -
        // 1);

        // free memory allocated for scratch space
        for (size_t k = 0; k < program_width; ++k) {
            aligned_free(numererator_accum[k]);
            aligned_free(denominator_accum[k]);
        }

        // TODO(luke): Commit to z here?

        // key->polynomial_cache.put("z_perm", std::move(z_perm));
    }
};
} // namespace honk::sumcheck
