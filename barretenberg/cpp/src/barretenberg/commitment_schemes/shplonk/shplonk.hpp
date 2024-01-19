#pragma once
#include "barretenberg/commitment_schemes/claim.hpp"
#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/commitment_schemes/verification_key.hpp"
#include "barretenberg/transcript/transcript.hpp"

/**
 * @brief Reduces multiple claims about commitments, each opened at a single point
 *  into a single claim for a single polynomial opened at a single point.
 *
 * We use the following terminology:
 * - Bₖ(X) is a random linear combination of all polynomials opened at Ωₖ
 *   we refer to it a 'merged_polynomial'.
 * - Tₖ(X) is the polynomial that interpolates Bₖ(X) over Ωₖ,
 * - zₖ(X) is the product of all (X-x), for x ∈ Ωₖ
 * - ẑₖ(X) = 1/zₖ(X)
 *
 * The challenges are ρ (batching) and r (random evaluation).
 *
 */
namespace bb::honk::pcs::shplonk {

/**
 * @brief Polynomial G(X) = Q(X) - ∑ₖ ẑₖ(r)⋅( Bₖ(X) − Tₖ(z) ), where Q(X) = ∑ₖ ( Bₖ(X) − Tₖ(X) ) / zₖ(X)
 *
 * @tparam Curve EC parameters
 */
template <typename Curve> using OutputWitness = bb::Polynomial<typename Curve::ScalarField>;

/**
 * @brief Prover output (claim=([G], r, 0), witness = G(X), proof = [Q])
 * that can be passed on to a univariate opening protocol.
 *
 * @tparam Curve EC parameters
 */
template <typename Curve> struct ProverOutput {
    OpeningPair<Curve> opening_pair; // single opening pair (challenge, evaluation)
    OutputWitness<Curve> witness;    // single polynomial G(X)
};

/**
 * @brief Shplonk Prover
 *
 * @tparam Curve EC parameters
 */
template <typename Curve> class ShplonkProver_ {
    using Fr = typename Curve::ScalarField;
    using Polynomial = bb::Polynomial<Fr>;

  public:
    /**
     * @brief Compute batched quotient polynomial Q(X) = ∑ⱼ ρʲ ⋅ ( fⱼ(X) − vⱼ) / ( X − xⱼ )
     *
     * @param opening_pairs list of opening pairs (xⱼ, vⱼ) for a witness polynomial fⱼ(X), s.t. fⱼ(xⱼ) = vⱼ.
     * @param witness_polynomials list of polynomials fⱼ(X).
     * @param nu
     * @return Polynomial Q(X)
     */
    static Polynomial compute_batched_quotient(std::span<const OpeningPair<Curve>> opening_pairs,
                                               std::span<const Polynomial> witness_polynomials,
                                               const Fr& nu)
    {
        // Find n, the maximum size of all polynomials fⱼ(X)
        size_t max_poly_size{ 0 };
        for (const auto& poly : witness_polynomials) {
            max_poly_size = std::max(max_poly_size, poly.size());
        }
        // Q(X) = ∑ⱼ ρʲ ⋅ ( fⱼ(X) − vⱼ) / ( X − xⱼ )
        Polynomial Q(max_poly_size);
        Polynomial tmp(max_poly_size);

        Fr current_nu = Fr::one();
        for (size_t j = 0; j < opening_pairs.size(); ++j) {
            // (Cⱼ, xⱼ, vⱼ)
            const auto& [challenge, evaluation] = opening_pairs[j];

            // tmp = ρʲ ⋅ ( fⱼ(X) − vⱼ) / ( X − xⱼ )
            tmp = witness_polynomials[j];
            tmp[0] -= evaluation;
            tmp.factor_roots(challenge);

            Q.add_scaled(tmp, current_nu);
            current_nu *= nu;
        }

        // Return batched quotient polynomial Q(X)
        return Q;
    };

    /**
     * @brief Compute partially evaluated batched quotient polynomial difference Q(X) - Q_z(X)
     *
     * @param opening_pairs list of opening pairs (xⱼ, vⱼ) for a witness polynomial fⱼ(X), s.t. fⱼ(xⱼ) = vⱼ.
     * @param witness_polynomials list of polynomials fⱼ(X).
     * @param batched_quotient_Q Q(X) = ∑ⱼ ρʲ ⋅ ( fⱼ(X) − vⱼ) / ( X − xⱼ )
     * @param nu_challenge
     * @param z_challenge
     * @return Output{OpeningPair, Polynomial}
     */
    static ProverOutput<Curve> compute_partially_evaluated_batched_quotient(
        std::span<const OpeningPair<Curve>> opening_pairs,
        std::span<const Polynomial> witness_polynomials,
        Polynomial&& batched_quotient_Q,
        const Fr& nu_challenge,
        const Fr& z_challenge)
    {
        const size_t num_opening_pairs = opening_pairs.size();

        // {ẑⱼ(r)}ⱼ , where ẑⱼ(r) = 1/zⱼ(r) = 1/(r - xⱼ)
        std::vector<Fr> inverse_vanishing_evals;
        inverse_vanishing_evals.reserve(num_opening_pairs);
        for (const auto& pair : opening_pairs) {
            inverse_vanishing_evals.emplace_back(z_challenge - pair.challenge);
        }
        Fr::batch_invert(inverse_vanishing_evals);

        // G(X) = Q(X) - Q_z(X) = Q(X) - ∑ⱼ ρʲ ⋅ ( fⱼ(X) − vⱼ) / ( r − xⱼ ),
        // s.t. G(r) = 0
        Polynomial G(std::move(batched_quotient_Q)); // G(X) = Q(X)

        // G₀ = ∑ⱼ ρʲ ⋅ vⱼ / ( r − xⱼ )
        Fr current_nu = Fr::one();
        Polynomial tmp(G.size());
        for (size_t j = 0; j < num_opening_pairs; ++j) {
            // (Cⱼ, xⱼ, vⱼ)
            const auto& [challenge, evaluation] = opening_pairs[j];

            // tmp = ρʲ ⋅ ( fⱼ(X) − vⱼ) / ( r − xⱼ )
            tmp = witness_polynomials[j];
            tmp[0] -= evaluation;
            Fr scaling_factor = current_nu * inverse_vanishing_evals[j]; // = ρʲ / ( r − xⱼ )

            // G -= ρʲ ⋅ ( fⱼ(X) − vⱼ) / ( r − xⱼ )
            G.add_scaled(tmp, -scaling_factor);

            current_nu *= nu_challenge;
        }

        // Return opening pair (z, 0) and polynomial G(X) = Q(X) - Q_z(X)
        return { .opening_pair = { .challenge = z_challenge, .evaluation = Fr::zero() }, .witness = std::move(G) };
    };
};

/**
 * @brief Shplonk Verifier
 *
 */
template <typename Curve> class ShplonkVerifier_ {
    using Fr = typename Curve::ScalarField;
    using GroupElement = typename Curve::Element;
    using Commitment = typename Curve::AffineElement;
    using VK = VerifierCommitmentKey<Curve>;

  public:
    /**
     * @brief Recomputes the new claim commitment [G] given the proof and
     * the challenge r. No verification happens so this function always succeeds.
     *
     * @param claims list of opening claims (Cⱼ, xⱼ, vⱼ) for a witness polynomial fⱼ(X), s.t. fⱼ(xⱼ) = vⱼ.
     * @param proof [Q(X)] = [ ∑ⱼ ρʲ ⋅ ( fⱼ(X) − vⱼ) / ( X − xⱼ ) ]
     * @param transcript
     * @return OpeningClaim
     */
    static OpeningClaim<Curve> reduce_verification(std::shared_ptr<VK> vk,
                                                   std::span<const OpeningClaim<Curve>> claims,
                                                   auto& transcript)
    {

        const size_t num_claims = claims.size();

        const Fr nu = transcript->get_challenge("Shplonk:nu");

        auto Q_commitment = transcript->template receive_from_prover<Commitment>("Shplonk:Q");

        const Fr z_challenge = transcript->get_challenge("Shplonk:z");

        // [G] = [Q] - ∑ⱼ ρʲ / ( r − xⱼ )⋅[fⱼ] + G₀⋅[1]
        //     = [Q] - [∑ⱼ ρʲ ⋅ ( fⱼ(X) − vⱼ) / ( r − xⱼ )]
        GroupElement G_commitment;

        // compute simulated commitment to [G] as a linear combination of
        // [Q], { [fⱼ] }, [1]:
        //  [G] = [Q] - ∑ⱼ (1/zⱼ(r))[Bⱼ]  + ( ∑ⱼ (1/zⱼ(r)) Tⱼ(r) )[1]
        //      = [Q] - ∑ⱼ (1/zⱼ(r))[Bⱼ]  +                    G₀ [1]
        // G₀ = ∑ⱼ ρʲ ⋅ vⱼ / ( r − xⱼ )
        auto G_commitment_constant = Fr(0);

        // TODO(#673): The recursive and non-recursive (native) logic is completely separated via the following
        // conditional. Much of the logic could be shared, but I've chosen to do it this way since soon the "else"
        // branch should be removed in its entirety, and "native" verification will utilize the recursive code paths
        // using a builder Simulator.
        if constexpr (Curve::is_stdlib_type) {
            auto builder = nu.get_context();

            // Containers for the inputs to the final batch mul
            std::vector<Commitment> commitments;
            std::vector<Fr> scalars;

            // [G] = [Q] - ∑ⱼ ρʲ / ( r − xⱼ )⋅[fⱼ] + G₀⋅[1]
            //     = [Q] - [∑ⱼ ρʲ ⋅ ( fⱼ(X) − vⱼ) / ( r − xⱼ )]
            commitments.emplace_back(Q_commitment);
            scalars.emplace_back(Fr(builder, 1)); // Fr(1)

            // Compute {ẑⱼ(r)}ⱼ , where ẑⱼ(r) = 1/zⱼ(r) = 1/(r - xⱼ)
            std::vector<Fr> inverse_vanishing_evals;
            inverse_vanishing_evals.reserve(num_claims);
            for (const auto& claim : claims) {
                // Note: no need for batch inversion; emulated inversion is cheap. (just show known inverse is valid)
                inverse_vanishing_evals.emplace_back((z_challenge - claim.opening_pair.challenge).invert());
            }

            auto current_nu = Fr(1);
            // Note: commitments and scalars vectors used only in recursion setting for batch mul
            for (size_t j = 0; j < num_claims; ++j) {
                // (Cⱼ, xⱼ, vⱼ)
                const auto& [opening_pair, commitment] = claims[j];

                Fr scaling_factor = current_nu * inverse_vanishing_evals[j]; // = ρʲ / ( r − xⱼ )

                // G₀ += ρʲ / ( r − xⱼ ) ⋅ vⱼ
                G_commitment_constant += scaling_factor * opening_pair.evaluation;

                current_nu *= nu;

                // Store MSM inputs for batch mul
                commitments.emplace_back(commitment);
                scalars.emplace_back(-scaling_factor);
            }

            commitments.emplace_back(GroupElement::one(builder));
            scalars.emplace_back(G_commitment_constant);

            // [G] += G₀⋅[1] = [G] + (∑ⱼ ρʲ ⋅ vⱼ / ( r − xⱼ ))⋅[1]
            G_commitment = GroupElement::batch_mul(commitments, scalars);

        } else {
            // [G] = [Q] - ∑ⱼ ρʲ / ( r − xⱼ )⋅[fⱼ] + G₀⋅[1]
            //     = [Q] - [∑ⱼ ρʲ ⋅ ( fⱼ(X) − vⱼ) / ( r − xⱼ )]
            G_commitment = Q_commitment;

            // Compute {ẑⱼ(r)}ⱼ , where ẑⱼ(r) = 1/zⱼ(r) = 1/(r - xⱼ)
            std::vector<Fr> inverse_vanishing_evals;
            inverse_vanishing_evals.reserve(num_claims);
            for (const auto& claim : claims) {
                inverse_vanishing_evals.emplace_back(z_challenge - claim.opening_pair.challenge);
            }
            Fr::batch_invert(inverse_vanishing_evals);

            auto current_nu = Fr(1);
            // Note: commitments and scalars vectors used only in recursion setting for batch mul
            for (size_t j = 0; j < num_claims; ++j) {
                // (Cⱼ, xⱼ, vⱼ)
                const auto& [opening_pair, commitment] = claims[j];

                Fr scaling_factor = current_nu * inverse_vanishing_evals[j]; // = ρʲ / ( r − xⱼ )

                // G₀ += ρʲ / ( r − xⱼ ) ⋅ vⱼ
                G_commitment_constant += scaling_factor * opening_pair.evaluation;

                // [G] -= ρʲ / ( r − xⱼ )⋅[fⱼ]
                G_commitment -= commitment * scaling_factor;

                current_nu *= nu;
            }

            // [G] += G₀⋅[1] = [G] + (∑ⱼ ρʲ ⋅ vⱼ / ( r − xⱼ ))⋅[1]
            G_commitment += vk->srs->get_first_g1() * G_commitment_constant;
        }

        // Return opening pair (z, 0) and commitment [G]
        return { { z_challenge, Fr(0) }, G_commitment };
    };
};
} // namespace bb::honk::pcs::shplonk
