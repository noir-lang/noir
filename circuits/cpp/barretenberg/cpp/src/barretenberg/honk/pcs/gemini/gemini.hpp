#pragma once

#include "../claim.hpp"
#include "barretenberg/honk/pcs/commitment_key.hpp"
#include "barretenberg/honk/transcript/transcript.hpp"
#include "barretenberg/polynomials/polynomial.hpp"

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
namespace proof_system::honk::pcs::gemini {

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
 * @tparam Params CommitmentScheme parameters
 */
template <typename Params> struct ProverOutput {
    std::vector<OpeningPair<Params>> opening_pairs;
    std::vector<barretenberg::Polynomial<typename Params::Fr>> witnesses;
};

template <typename Params> class MultilinearReductionScheme {
    using CK = typename Params::CommitmentKey;

    using Fr = typename Params::Fr;
    using GroupElement = typename Params::GroupElement;
    using Commitment = typename Params::Commitment;
    using Polynomial = barretenberg::Polynomial<Fr>;

  public:
    static std::vector<Polynomial> compute_fold_polynomials(std::span<const Fr> mle_opening_point,
                                                            Polynomial&& batched_unshifted,
                                                            Polynomial&& batched_to_be_shifted);

    static ProverOutput<Params> compute_fold_polynomial_evaluations(std::span<const Fr> mle_opening_point,
                                                                    std::vector<Polynomial>&& fold_polynomials,
                                                                    const Fr& r_challenge);

    static std::vector<OpeningClaim<Params>> reduce_verify(std::span<const Fr> mle_opening_point, /* u */
                                                           const Fr batched_evaluation,           /* all */
                                                           GroupElement& batched_f,               /* unshifted */
                                                           GroupElement& batched_g,               /* to-be-shifted */
                                                           VerifierTranscript<Fr>& transcript);

    static std::vector<Fr> powers_of_rho(const Fr rho, const size_t num_powers);

  private:
    static Fr compute_eval_pos(const Fr batched_mle_eval,
                               std::span<const Fr> mle_vars,
                               std::span<const Fr> r_squares,
                               std::span<const Fr> fold_polynomial_evals);

    static std::vector<Fr> squares_of_r(const Fr r, const size_t num_squares);

    static std::pair<GroupElement, GroupElement> compute_simulated_commitments(GroupElement& batched_f,
                                                                               GroupElement& batched_g,
                                                                               Fr r);
}; // namespace proof_system::honk::pcs::gemini
extern template class MultilinearReductionScheme<kzg::Params>;
extern template class MultilinearReductionScheme<ipa::Params>;
} // namespace proof_system::honk::pcs::gemini
