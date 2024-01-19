#pragma once

#include "../claim.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/transcript/transcript.hpp"

#include <vector>

/**
 * @brief Protocol for opening several multi-linear polynomials at the same point.
 *
 *
 * m = number of variables
 * n = 2ᵐ
 * u = (u₀,...,uₘ₋₁)
 * f₀, …, fₖ₋₁ = multilinear polynomials,
 * g₀, …, gₕ₋₁ = shifted multilinear polynomial,
 *  Each gⱼ is the left-shift of some f↺ᵢ, and gⱼ points to the same memory location as fᵢ.
 * v₀, …, vₖ₋₁, v↺₀, …, v↺ₕ₋₁ = multilinear evalutions s.t. fⱼ(u) = vⱼ, and gⱼ(u) = f↺ⱼ(u) = v↺ⱼ
 *
 * We use a challenge ρ to create a random linear combination of all fⱼ,
 * and actually define A₀ = F + G↺, where
 *   F  = ∑ⱼ ρʲ fⱼ
 *   G  = ∑ⱼ ρᵏ⁺ʲ gⱼ,
 *   G↺ = is the shift of G
 * where fⱼ is normal, and gⱼ is shifted.
 * The evaluations are also batched, and
 *   v  = ∑ ρʲ⋅vⱼ + ∑ ρᵏ⁺ʲ⋅v↺ⱼ = F(u) + G↺(u)
 *
 * The prover then creates the folded polynomials A₀, ..., Aₘ₋₁,
 * and opens them at different points, as univariates.
 *
 * We open A₀ as univariate at r and -r.
 * Since A₀ = F + G↺, but the verifier only has commitments to the gⱼs,
 * we need to partially evaluate A₀ at both evaluation points.
 * As univariate, we have
 *  A₀(X) = F(X) + G↺(X) = F(X) + G(X)/X
 * So we define
 *  - A₀₊(X) = F(X) + G(X)/r
 *  - A₀₋(X) = F(X) − G(X)/r
 * So that A₀₊(r) = A₀(r) and A₀₋(-r) = A₀(-r).
 * The verifier is able to computed the simulated commitments to A₀₊(X) and A₀₋(X)
 * since they are linear-combinations of the commitments [fⱼ] and [gⱼ].
 */
namespace bb::honk::pcs::gemini {

/**
 * @brief Prover output (evalutation pair, witness) that can be passed on to Shplonk batch opening.
 * @details Evaluation pairs {r, A₀₊(r)}, {-r, A₀₋(-r)}, {-r^{2^j}, Aⱼ(-r^{2^j)}, j = [1, ..., m-1]
 * and witness (Fold) polynomials
 * [
 *   A₀₊(X) = F(X) + r⁻¹⋅G(X)
 *   A₀₋(X) = F(X) - r⁻¹⋅G(X)
 *   A₁(X) = (1-u₀)⋅even(A₀)(X) + u₀⋅odd(A₀)(X)
 *   ...
 *   Aₘ₋₁(X) = (1-uₘ₋₂)⋅even(Aₘ₋₂)(X) + uₘ₋₂⋅odd(Aₘ₋₂)(X)
 * ]
 * @tparam Curve CommitmentScheme parameters
 */
template <typename Curve> struct ProverOutput {
    std::vector<OpeningPair<Curve>> opening_pairs;
    std::vector<bb::Polynomial<typename Curve::ScalarField>> witnesses;
};

/**
 * @brief Compute powers of challenge ρ
 *
 * @tparam Fr
 * @param rho
 * @param num_powers
 * @return std::vector<Fr>
 */
template <class Fr> inline std::vector<Fr> powers_of_rho(const Fr rho, const size_t num_powers)
{
    std::vector<Fr> rhos = { Fr(1), rho };
    rhos.reserve(num_powers);
    for (size_t j = 2; j < num_powers; j++) {
        rhos.emplace_back(rhos[j - 1] * rho);
    }
    return rhos;
};

/**
 * @brief Compute squares of folding challenge r
 *
 * @param r
 * @param num_squares The number of foldings
 * @return std::vector<typename Curve::ScalarField>
 */
template <class Fr> inline std::vector<Fr> squares_of_r(const Fr r, const size_t num_squares)
{
    std::vector<Fr> squares = { r };
    squares.reserve(num_squares);
    for (size_t j = 1; j < num_squares; j++) {
        squares.emplace_back(squares[j - 1].sqr());
    }
    return squares;
};

template <typename Curve> class GeminiProver_ {
    using Fr = typename Curve::ScalarField;
    using Polynomial = bb::Polynomial<Fr>;

  public:
    static std::vector<Polynomial> compute_gemini_polynomials(std::span<const Fr> mle_opening_point,
                                                              Polynomial&& batched_unshifted,
                                                              Polynomial&& batched_to_be_shifted);

    static ProverOutput<Curve> compute_fold_polynomial_evaluations(std::span<const Fr> mle_opening_point,
                                                                   std::vector<Polynomial>&& gemini_polynomials,
                                                                   const Fr& r_challenge);
}; // namespace bb::honk::pcs::gemini

template <typename Curve> class GeminiVerifier_ {
    using Fr = typename Curve::ScalarField;
    using GroupElement = typename Curve::Element;
    using Commitment = typename Curve::AffineElement;

  public:
    /**
     * @brief Returns univariate opening claims for the Fold polynomials to be checked later
     *
     * @param mle_opening_point the MLE evaluation point u
     * @param batched_evaluation batched evaluation from multivariate evals at the point u
     * @param batched_f batched commitment to unshifted polynomials
     * @param batched_g batched commitment to to-be-shifted polynomials
     * @param proof commitments to the m-1 folded polynomials, and alleged evaluations.
     * @param transcript
     * @return Fold polynomial opening claims: (r, A₀(r), C₀₊), (-r, A₀(-r), C₀₋), and
     * (Cⱼ, Aⱼ(-r^{2ʲ}), -r^{2}), j = [1, ..., m-1]
     */
    static std::vector<OpeningClaim<Curve>> reduce_verification(std::span<const Fr> mle_opening_point, /* u */
                                                                const Fr batched_evaluation,           /* all */
                                                                GroupElement& batched_f,               /* unshifted */
                                                                GroupElement& batched_g, /* to-be-shifted */
                                                                auto& transcript)
    {
        const size_t num_variables = mle_opening_point.size();

        // Get polynomials Fold_i, i = 1,...,m-1 from transcript
        std::vector<Commitment> commitments;
        commitments.reserve(num_variables - 1);
        for (size_t i = 0; i < num_variables - 1; ++i) {
            auto commitment =
                transcript->template receive_from_prover<Commitment>("Gemini:FOLD_" + std::to_string(i + 1));
            commitments.emplace_back(commitment);
        }

        // compute vector of powers of random evaluation point r
        const Fr r = transcript->get_challenge("Gemini:r");
        std::vector<Fr> r_squares = squares_of_r(r, num_variables);

        // Get evaluations a_i, i = 0,...,m-1 from transcript
        std::vector<Fr> evaluations;
        evaluations.reserve(num_variables);
        for (size_t i = 0; i < num_variables; ++i) {
            auto eval = transcript->template receive_from_prover<Fr>("Gemini:a_" + std::to_string(i));
            evaluations.emplace_back(eval);
        }

        // Compute evaluation A₀(r)
        auto a_0_pos = compute_eval_pos(batched_evaluation, mle_opening_point, r_squares, evaluations);

        // C₀_r_pos = ∑ⱼ ρʲ⋅[fⱼ] + r⁻¹⋅∑ⱼ ρᵏ⁺ʲ [gⱼ]
        // C₀_r_pos = ∑ⱼ ρʲ⋅[fⱼ] - r⁻¹⋅∑ⱼ ρᵏ⁺ʲ [gⱼ]
        auto [c0_r_pos, c0_r_neg] = compute_simulated_commitments(batched_f, batched_g, r);

        std::vector<OpeningClaim<Curve>> fold_polynomial_opening_claims;
        fold_polynomial_opening_claims.reserve(num_variables + 1);

        // ( [A₀₊], r, A₀(r) )
        fold_polynomial_opening_claims.emplace_back(OpeningClaim<Curve>{ { r, a_0_pos }, c0_r_pos });
        // ( [A₀₋], -r, A₀(-r) )
        fold_polynomial_opening_claims.emplace_back(OpeningClaim<Curve>{ { -r, evaluations[0] }, c0_r_neg });
        for (size_t l = 0; l < num_variables - 1; ++l) {
            // ([A₀₋], −r^{2ˡ}, Aₗ(−r^{2ˡ}) )
            fold_polynomial_opening_claims.emplace_back(
                OpeningClaim<Curve>{ { -r_squares[l + 1], evaluations[l + 1] }, commitments[l] });
        }

        return fold_polynomial_opening_claims;
    }

  private:
    /**
     * @brief Compute the expected evaluation of the univariate commitment to the batched polynomial.
     *
     * @param batched_mle_eval The evaluation of the folded polynomials
     * @param mle_vars MLE opening point u
     * @param r_squares squares of r, r², ..., r^{2ᵐ⁻¹}
     * @param fold_polynomial_evals series of Aᵢ₋₁(−r^{2ⁱ⁻¹})
     * @return evaluation A₀(r)
     */
    static Fr compute_eval_pos(const Fr batched_mle_eval,
                               std::span<const Fr> mle_vars,
                               std::span<const Fr> r_squares,
                               std::span<const Fr> fold_polynomial_evals)
    {
        const size_t num_variables = mle_vars.size();

        const auto& evals = fold_polynomial_evals;

        // Initialize eval_pos with batched MLE eval v = ∑ⱼ ρʲ vⱼ + ∑ⱼ ρᵏ⁺ʲ v↺ⱼ
        Fr eval_pos = batched_mle_eval;
        for (size_t l = num_variables; l != 0; --l) {
            const Fr r = r_squares[l - 1];    // = rₗ₋₁ = r^{2ˡ⁻¹}
            const Fr eval_neg = evals[l - 1]; // = Aₗ₋₁(−r^{2ˡ⁻¹})
            const Fr u = mle_vars[l - 1];     // = uₗ₋₁

            // The folding property ensures that
            //                     Aₗ₋₁(r^{2ˡ⁻¹}) + Aₗ₋₁(−r^{2ˡ⁻¹})      Aₗ₋₁(r^{2ˡ⁻¹}) - Aₗ₋₁(−r^{2ˡ⁻¹})
            // Aₗ(r^{2ˡ}) = (1-uₗ₋₁) ----------------------------- + uₗ₋₁ -----------------------------
            //                                   2                                2r^{2ˡ⁻¹}
            // We solve the above equation in Aₗ₋₁(r^{2ˡ⁻¹}), using the previously computed Aₗ(r^{2ˡ}) in eval_pos
            // and using Aₗ₋₁(−r^{2ˡ⁻¹}) sent by the prover in the proof.
            eval_pos = ((r * eval_pos * 2) - eval_neg * (r * (Fr(1) - u) - u)) / (r * (Fr(1) - u) + u);
        }

        return eval_pos; // return A₀(r)
    }

    /**
     * @brief Computes two commitments to A₀ partially evaluated in r and -r.
     *
     * @param batched_f batched commitment to non-shifted polynomials
     * @param batched_g batched commitment to to-be-shifted polynomials
     * @param r evaluation point at which we have partially evaluated A₀ at r and -r.
     * @return std::pair<Commitment, Commitment>  c0_r_pos, c0_r_neg
     */
    static std::pair<GroupElement, GroupElement> compute_simulated_commitments(GroupElement& batched_f,
                                                                               GroupElement& batched_g,
                                                                               Fr r)
    {
        // C₀ᵣ₊ = [F] + r⁻¹⋅[G]
        GroupElement C0_r_pos;
        // C₀ᵣ₋ = [F] - r⁻¹⋅[G]
        GroupElement C0_r_neg;
        Fr r_inv = r.invert(); // r⁻¹

        // If in a recursive setting, perform a batch mul. Otherwise, accumulate directly.
        // TODO(#673): The following if-else represents the stldib/native code paths. Once the "native" verifier is
        // achieved through a builder Simulator, the stdlib codepath should become the only codepath.
        if constexpr (Curve::is_stdlib_type) {
            std::vector<GroupElement> commitments = { batched_f, batched_g };
            auto builder = r.get_context();
            auto one = Fr(builder, 1);
            // TODO(#707): these batch muls include the use of 1 as a scalar. This is handled appropriately as a non-mul
            // (add-accumulate) in the goblin batch_mul but is done inefficiently as a scalar mul in the conventional
            // emulated batch mul.
            C0_r_pos = GroupElement::batch_mul(commitments, { one, r_inv });
            C0_r_neg = GroupElement::batch_mul(commitments, { one, -r_inv });
        } else {
            C0_r_pos = batched_f;
            C0_r_neg = batched_f;
            if (!batched_g.is_point_at_infinity()) {
                batched_g = batched_g * r_inv;
                C0_r_pos += batched_g;
                C0_r_neg -= batched_g;
            }
        }

        return { C0_r_pos, C0_r_neg };
    }

}; // namespace bb::honk::pcs::gemini

} // namespace bb::honk::pcs::gemini
