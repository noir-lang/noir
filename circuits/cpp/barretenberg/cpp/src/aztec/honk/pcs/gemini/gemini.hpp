#pragma once

#include "../claim.hpp"
#include "common/log.hpp"
#include "honk/pcs/commitment_key.hpp"
#include "polynomials/polynomial.hpp"

#include <common/assert.hpp>
#include <memory>
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
 * commits and opens them at different points, as univariates.
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
namespace honk::pcs::gemini {

/**
 * @brief A Gemini proof contains the m-1 commitments to the
 * folded univariates, and corresponding evaluations
 * at -r, -r², …, r^{2ᵐ⁻¹}.
 *
 * The evaluations allow the verifier to reconstruct the evaluation of A₀(r).
 *
 * @tparam Params CommitmentScheme parameters
 */
template <typename Params> struct Proof {
    /** @brief Commitments to folded polynomials (size = m-1)
     *
     * [ C₁, …,  Cₘ₋₁], where Cₗ = commit(Aₗ(X)) of size 2ᵐ⁻ˡ
     */
    std::vector<typename Params::Commitment> commitments;

    /**
     * @brief Evaluations of batched and folded polynomials (size m)
     *
     * [A₀(-r) , ..., Aₘ₋₁(-r^{2ᵐ⁻¹})]
     */
    std::vector<typename Params::Fr> evals;
};

/**
 * @brief Univariate opening claims for multiple polynomials,
 * each opened at a single different point (size = m+1).
 *
 * [
 *   (C₀₊ , A₀  ( r)       ,  r )
 *   (C₀₋ , A₀  (-r)       , -r )
 *   (C₁  , A₁  (-r²)      , -r²)
 *   ...
 *   (Cₘ₋₁, Aₘ₋₁(-r^{2ᵐ⁻¹}), -r^{2ᵐ⁻¹})
 * ]
 * where
 *   C₀₊ is a simulated commitment to A₀ partially evaluated at r
 *   C₀₋ is a simulated commitment to A₀ partially evaluated at -r
 *
 * @tparam Params CommitmentScheme parameters
 */
template <typename Params> using OutputClaim = std::vector<OpeningClaim<Params>>;

/**
 * @brief Univariate witness polynomials for opening all the
 *
 * [
 *   A₀₊(X) = F(X) + r⁻¹⋅G(X)
 *   A₀₋(X) = F(X) - r⁻¹⋅G(X)
 *   A₁(X) = (1-u₀)⋅even(A₀)(X) + u₀⋅odd(A₀)(X)
 *   ...
 *   Aₘ₋₁(X) = (1-uₘ₋₂)⋅even(Aₘ₋₂)(X) + uₘ₋₂⋅odd(Aₘ₋₂)(X)
 * ]
 * where
 *           /  r ⁻¹ ⋅ fⱼ(X)  if fⱼ is a shift
 * fⱼ₊(X) = |
 *           \  fⱼ(X)         otherwise
 *
 *           / (-r)⁻¹ ⋅ fⱼ(X) if fⱼ is a shift
 * fⱼ₋(X) = |
 *           \  fⱼ(X)         otherwise
 *
 *
 * @tparam Params CommitmentScheme parameters
 */
template <typename Params> using OutputWitness = std::vector<barretenberg::Polynomial<typename Params::Fr>>;

/**
 * @brief Prover output (claim, witness, proof) that can be passed on to Shplonk batch opening.
 *
 * @tparam Params CommitmentScheme parameters
 */
template <typename Params> struct ProverOutput {
    OutputClaim<Params> claim;

    OutputWitness<Params> witness;

    Proof<Params> proof;
};

template <typename Params> class MultilinearReductionScheme {
    using CK = typename Params::CK;

    using Fr = typename Params::Fr;
    using Commitment = typename Params::Commitment;
    using CommitmentAffine = typename Params::C; // TODO(luke): find a better name
    using Polynomial = barretenberg::Polynomial<Fr>;

  public:
    /**
     * @brief reduces claims about multiple (shifted) MLE evaluation
     *
     * @param ck is the commitment key for creating the new commitments
     * @param mle_opening_point = u =(u₀,...,uₘ₋₁) is the MLE opening point
     * @param claims a set of MLE claims for the same point u
     * @param mle_witness_polynomials the MLE polynomials for each evaluation.
     *      Internally, it contains a reference to the non-shifted polynomial.
     * @param transcript
     * @return Output (result_claims, proof, folded_witness_polynomials)
     *
     * Note: Only the proof and witness produced by this function are needed
     * in the simple construction and verification of a single Honk proof. The
     * result_claims constructed in this function are only relevant in a
     * recursion setting.
     */
    static ProverOutput<Params> reduce_prove(std::shared_ptr<CK> ck,
                                             std::span<const Fr> mle_opening_point,
                                             std::span<const MLEOpeningClaim<Params>> claims,
                                             std::span<const MLEOpeningClaim<Params>> claims_shifted,
                                             const std::vector<Polynomial*>& mle_witness_polynomials,
                                             const std::vector<Polynomial*>& mle_witness_polynomials_shifted,
                                             const auto& transcript)
    {
        // Relabel inputs to be consistent with the comments
        auto& claims_f = claims;
        auto& claims_g = claims_shifted;
        auto& polys_f = mle_witness_polynomials;
        auto& polys_g = mle_witness_polynomials_shifted;

        const size_t num_variables = mle_opening_point.size(); // m
        const size_t n = 1 << num_variables;
        const size_t num_polys_f = polys_f.size();
        const size_t num_polys_g = polys_g.size();
        const size_t num_polys = num_polys_f + num_polys_g;
        ASSERT(claims_f.size() == num_polys_f);
        ASSERT(claims_g.size() == num_polys_g);

        // Generate batching challenge ρ and powers 1,ρ,…,ρᵐ⁻¹
        transcript->apply_fiat_shamir("rho");
        Fr rho = Fr::serialize_from_buffer(transcript->get_challenge("rho").begin());
        const std::vector<Fr> rhos = powers_of_rho(rho, num_polys);
        std::span<const Fr> rhos_span{ rhos };
        std::span rhos_f = rhos_span.subspan(0, num_polys_f);
        std::span rhos_g = rhos_span.subspan(num_polys_f, num_polys_g);

        // Allocate m+1 witness polynomials
        //
        // At the end, the first two will contain the batched polynomial
        // partially evaluated at the challenges r,-r.
        // The other m-1 polynomials correspond to the foldings of A₀
        std::vector<Polynomial> witness_polynomials;
        witness_polynomials.reserve(num_variables + 1);

        // Create the batched polynomials
        // F(X) = ∑ⱼ ρʲ   fⱼ(X)
        // G(X) = ∑ⱼ ρᵏ⁺ʲ gⱼ(X)
        // using powers of the challenge ρ.
        //
        // In what follows, we use indices j, k for non-shifted and shifted polynomials respectively.
        //
        // We separate A₀(X) into two polynomials F(X), G↺(X)
        // such that A₀(X) = F(X) + G↺(X) = F(X) + G(X)/X.

        //  F(X) = ∑ⱼ ρʲ fⱼ(X)
        Polynomial& batched_F = witness_polynomials.emplace_back(Polynomial(n, n));
        for (size_t j = 0; j < num_polys_f; ++j) {
            const size_t n_j = polys_f[j]->size();
            ASSERT(n_j <= n);
            // F(X) += ρʲ fⱼ(X)
            batched_F.add_scaled(*polys_f[j], rhos_f[j]);
        }

        //  G(X) = ∑ⱼ ρʲ gⱼ(X)
        Polynomial& batched_G = witness_polynomials.emplace_back(Polynomial(n, n));
        for (size_t j = 0; j < num_polys_g; ++j) {
            const size_t n_j = polys_g[j]->size();
            ASSERT(n_j <= n);
            // G(X) += ρʲ gⱼ(X)
            batched_G.add_scaled(*polys_g[j], rhos_g[j]);
        }

        // A₀(X) = F(X) + G↺(X) = F(X) + G(X)/X.
        Polynomial A_0(batched_F);
        A_0 += batched_G.shifted();

        // Create the folded polynomials A₁(X),…,Aₘ₋₁(X)
        //
        // A_l = Aₗ(X) is the polynomial being folded
        // in the first iteration, we take the batched polynomial
        // in the next iteration, it is the previously folded one
        Fr* A_l = A_0.get_coefficients();
        for (size_t l = 0; l < num_variables - 1; ++l) {
            const Fr u_l = mle_opening_point[l];

            // size of the previous polynomial/2
            const size_t n_l = 1 << (num_variables - l - 1);

            // A_l_fold = Aₗ₊₁(X) = (1-uₗ)⋅even(Aₗ)(X) + uₗ⋅odd(Aₗ)(X)
            Fr* A_l_fold = witness_polynomials.emplace_back(Polynomial(n_l, n_l)).get_coefficients();

            // fold the previous polynomial with odd and even parts
            for (size_t i = 0; i < n_l; ++i) {
                // TODO parallelize

                // fold(Aₗ)[i] = (1-uₗ)⋅even(Aₗ)[i] + uₗ⋅odd(Aₗ)[i]
                //            = (1-uₗ)⋅Aₗ[2i]      + uₗ⋅Aₗ[2i+1]
                //            = Aₗ₊₁[i]
                A_l_fold[i] = A_l[i << 1] + u_l * (A_l[(i << 1) + 1] - A_l[i << 1]);
            }

            // set Aₗ₊₁ = Aₗ for the next iteration
            A_l = A_l_fold;
        }

        /*
         * Create commitments C₁,…,Cₘ₋₁
         */
        std::vector<Commitment> commitments;
        commitments.reserve(num_variables - 1);
        for (size_t l = 0; l < num_variables - 1; ++l) {
            commitments.emplace_back(ck->commit(witness_polynomials[l + 2]));
        }

        /*
         * Add commitments FOLD_i, i = 1,...,d-1 to transcript and generate evaluation challenge r, and derive -r, r²
         */
        for (size_t i = 0; i < commitments.size(); ++i) {
            std::string label = "FOLD_" + std::to_string(i + 1);
            transcript->add_element(label, static_cast<CommitmentAffine>(commitments[i]).to_buffer());
        }
        transcript->apply_fiat_shamir("r");
        const Fr r = Fr::serialize_from_buffer(transcript->get_challenge("r").begin());

        /*
         * Compute the witness polynomials for the resulting claim
         *
         *
         * We are batching all polynomials together, and linearly combining them with
         * powers of ρ
         */

        // 2 simulated polynomials and (m-1) polynomials from this round
        Fr r_inv = r.invert();
        // G(X) *= r⁻¹
        batched_G *= r_inv;

        // To avoid an extra allocation, we reuse the following polynomials
        // but rename them to represent the result.
        // tmp     = A₀(X) (&tmp     == &A_0)
        // A_0_pos = F(X)  (&A_0_pos == &batched_F)
        Polynomial& tmp = A_0;
        Polynomial& A_0_pos = witness_polynomials[0];

        tmp = batched_F;
        // A₀₊(X) = F(X) + G(X)/r, s.t. A₀₊(r) = A₀(r)
        A_0_pos += batched_G;

        std::swap(tmp, batched_G);
        // After the swap, we have
        // tmp = G(X)/r
        // A_0_neg = F(X) (since &batched_F == &A_0_neg)
        Polynomial& A_0_neg = witness_polynomials[1];

        // A₀₋(X) = F(X) - G(X)/r, s.t. A₀₋(-r) = A₀(-r)
        A_0_neg -= tmp;

        /*
         * compute rₗ = r^{2ˡ} for l = 0, 1, ..., m-1
         */
        std::vector<Fr> r_squares = squares_of_r(r, num_variables);

        // evaluate all new polynomials Aₗ at -rₗ
        std::vector<Fr> evals;
        evals.reserve(num_variables);
        for (size_t l = 0; l < num_variables; ++l) {
            const Polynomial& A_l = witness_polynomials[l + 1];
            const Fr r_l_neg = -r_squares[l];
            evals.emplace_back(A_l.evaluate(r_l_neg));
        }

        /*
         * Add evaluations a_i, i = 0,...,m-1 to transcript
         */
        for (size_t i = 0; i < evals.size(); ++i) {
            std::string label = "a_" + std::to_string(i);
            transcript->add_element(label, evals[i].to_buffer());
        }

        /*
         * Construct the 'Proof' which consists of:
         * (1) The m-1 commitments [Fold^{l}], l = 1, ..., m-1
         * (2) The m evaluations a_0 = Fold_{-r}^(0)(-r), and a_l = Fold^(l)(-r^{2^l}), l = 1, ..., m-1
         */
        Proof<Params> proof = { commitments, evals };

        /*
         * Compute new claims and add them to the output
         */
        auto result_claims =
            compute_output_claim_from_proof(claims_f, claims_g, mle_opening_point, rhos, r_squares, proof);

        return { result_claims, std::move(witness_polynomials), proof };
    };

    /**
     * @brief Checks that all MLE evaluations vⱼ contained in the list of m MLE opening claims
     * is correct, and returns univariate polynomial opening claims to be checked later
     *
     * @param mle_opening_point the MLE evaluation point for all claims
     * @param claims MLE claims with (C, v) and C is a univariate commitment
     * @param claims_shifted MLE claims with (C, v↺) and C is a univariate commitment
     *      to the non-shifted polynomial
     * @param proof commitments to the m-1 folded polynomials, and alleged evaluations.
     * @param transcript
     * @return BatchOpeningClaim
     */
    static OutputClaim<Params> reduce_verify(std::span<const Fr> mle_opening_point,
                                             std::span<const MLEOpeningClaim<Params>> claims,
                                             std::span<const MLEOpeningClaim<Params>> claims_shifted,
                                             const Proof<Params>& proof,
                                             const auto& transcript)
    {
        // Relabel inputs to be more consistent with the math comments.
        auto& claims_f = claims;
        auto& claims_g = claims_shifted;

        const size_t num_variables = mle_opening_point.size();
        const size_t num_claims_f = claims_f.size();
        const size_t num_claims_g = claims_g.size();
        const size_t num_claims = num_claims_f + num_claims_g;

        // batching challenge ρ
        const Fr rho = Fr::serialize_from_buffer(transcript->get_challenge("rho").begin());
        // compute vector of powers of rho only once
        std::vector<Fr> rhos = powers_of_rho(rho, num_claims);

        // random evaluation point r
        const Fr r = Fr::serialize_from_buffer(transcript->get_challenge("r").begin());

        std::vector<Fr> r_squares = squares_of_r(r, num_variables);

        return compute_output_claim_from_proof(claims_f, claims_g, mle_opening_point, rhos, r_squares, proof);
    };

  private:
    /**
     * @brief computes the output claim given the transcript.
     * This method is common for both prover and verifier.
     *
     * @param claims_f set of input claims for non-shifted evaluations
     * @param claims_g set of input claims for shifted evaluations
     * @param mle_vars MLE opening point u
     * @param rhos powers of the initial batching challenge ρ
     * @param r_squares squares of r, r², ..., r^{2ᵐ⁻¹}
     * @param proof the proof produced by the prover
     * @return OutputClaim<Params>
     */
    static OutputClaim<Params> compute_output_claim_from_proof(std::span<const MLEOpeningClaim<Params>> claims_f,
                                                               std::span<const MLEOpeningClaim<Params>> claims_g,
                                                               std::span<const Fr> mle_vars,
                                                               std::span<const Fr> rhos,
                                                               std::span<const Fr> r_squares,
                                                               const Proof<Params>& proof)
    {
        const size_t num_variables = mle_vars.size();
        const size_t num_claims_f = claims_f.size();
        const size_t num_claims_g = claims_g.size();

        const Fr r = r_squares[0];

        const auto& evals = proof.evals;

        // compute the batched MLE evaluation
        // v = ∑ⱼ ρʲ vⱼ + ∑ⱼ ρᵏ⁺ʲ v↺ⱼ
        Fr mle_eval{ Fr::zero() };

        // add non-shifted evaluations
        std::span rhos_f = rhos.subspan(0, num_claims_f);
        for (size_t j = 0; j < num_claims_f; ++j) {
            mle_eval += claims_f[j].evaluation * rhos_f[j];
        }
        // add shifted evaluations
        std::span rhos_g = rhos.subspan(num_claims_f, num_claims_g);
        for (size_t j = 0; j < num_claims_g; ++j) {
            mle_eval += claims_g[j].evaluation * rhos_g[j];
        }

        // For l = m, ..., 1
        // Initialize eval_pos = Aₘ(r^2ᵐ) = v
        Fr eval_pos = mle_eval;
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
        // eval_pos now equals A₀(r)

        // add the claim for the first polynomial A₀
        // if there is a shift, then we need to add a separate claim for the
        // evaluation at r and -r.
        // C₀_r_pos = ∑ⱼ ρʲ⋅[fⱼ] + r⁻¹⋅∑ⱼ ρᵏ⁺ʲ [gⱼ]
        // C₀_r_pos = ∑ⱼ ρʲ⋅[fⱼ] - r⁻¹⋅∑ⱼ ρᵏ⁺ʲ [gⱼ]
        auto [c0_r_pos, c0_r_neg] = compute_simulated_commitments(claims_f, claims_g, rhos, r);

        std::vector<OpeningClaim<Params>> result_claims;
        result_claims.reserve(num_variables + 1);

        // ( [A₀₊], r, A₀(r) )
        result_claims.emplace_back(OpeningClaim<Params>{ c0_r_pos, r, eval_pos });
        // ( [A₀₋], -r, A₀(-r) )
        result_claims.emplace_back(OpeningClaim<Params>{ c0_r_neg, -r, evals[0] });
        for (size_t l = 0; l < num_variables - 1; ++l) {
            // ([A₀₋], -r, Aₗ(−r^{2ˡ}) )
            result_claims.emplace_back(OpeningClaim<Params>{ proof.commitments[l], -r_squares[l + 1], evals[l + 1] });
        }

        return result_claims;
    };

    static std::vector<Fr> powers_of_rho(const Fr rho, const size_t num_powers)
    {
        std::vector<Fr> rhos = { Fr(1), rho };
        rhos.reserve(num_powers);
        for (size_t j = 2; j < num_powers; j++) {
            rhos.emplace_back(rhos[j - 1] * rho);
        }
        return rhos;
    };

    static std::vector<Fr> squares_of_r(const Fr r, const size_t num_squares)
    {
        std::vector<Fr> squares = { r };
        squares.reserve(num_squares);
        for (size_t j = 1; j < num_squares; j++) {
            squares.emplace_back(squares[j - 1].sqr());
        }
        return squares;
    };

    /**
     * @brief Computes two commitments to A₀ partially evaluated in r and -r.
     *
     * @param claims_f array of claims containing commitments to non-shifted polynomials
     * @param claims_g array of claims containing commitments to shifted polynomials
     * @param rhos vector of m powers of rho used for linear combination
     * @param r evaluation point at which we have partially evaluated A₀ at r and -r.
     * @return std::pair<Commitment, Commitment>  c0_r_pos, c0_r_neg
     */
    static std::pair<Commitment, Commitment> compute_simulated_commitments(
        std::span<const MLEOpeningClaim<Params>> claims_f,
        std::span<const MLEOpeningClaim<Params>> claims_g,
        std::span<const Fr> rhos,
        Fr r)
    {
        const size_t num_claims_f = claims_f.size();
        const size_t num_claims_g = claims_g.size();

        Fr r_inv = r.invert();

        // Commitment to F(X), G(X)
        Commitment batched_f = Commitment::zero();
        std::span rhos_f = rhos.subspan(0, num_claims_f);
        for (size_t j = 0; j < num_claims_f; ++j) {
            batched_f += claims_f[j].commitment * rhos_f[j];
        }

        // Commitment to G(X)
        Commitment batched_g = Commitment::zero();
        std::span rhos_g = rhos.subspan(num_claims_f, num_claims_g);
        for (size_t j = 0; j < num_claims_g; ++j) {
            batched_g += claims_g[j].commitment * rhos_g[j];
        }

        // C₀ᵣ₊ = [F] + r⁻¹⋅[G]
        Commitment C0_r_pos = batched_f;
        // C₀ᵣ₋ = [F] - r⁻¹⋅[G]
        Commitment C0_r_neg = batched_f;
        if (!batched_g.is_point_at_infinity()) {
            batched_g *= r_inv;
            C0_r_pos += batched_g;
            C0_r_neg -= batched_g;
        }
        return { C0_r_pos, C0_r_neg };
    };
};
} // namespace honk::pcs::gemini