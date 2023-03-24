#pragma once
#include "barretenberg/honk/pcs/claim.hpp"
#include "shplonk.hpp"
#include "barretenberg/honk/pcs/commitment_key.hpp"
#include "barretenberg/honk/transcript/transcript.hpp"

namespace honk::pcs::shplonk {

/**
 * @brief Protocol for opening several polynomials, each in a single different point.
 * It is a simplification of the MultiBatchOpeningScheme.
 *
 * @tparam Params for the given commitment scheme
 */
template <typename Params> class SingleBatchOpeningScheme {
    using CK = typename Params::CK;

    using Fr = typename Params::Fr;
    using Commitment = typename Params::Commitment;
    using CommitmentAffine = typename Params::C;
    using Polynomial = barretenberg::Polynomial<Fr>;

  public:
    /**
     * @brief Batches several single-point 'OpeningClaim' into a single 'OpeningClaim' suitable for
     * a univariate polynomial opening scheme.
     *
     * @param ck CommitmentKey
     * @param opening_pairs list of opening pairs (xⱼ, vⱼ) for a witness polynomial fⱼ(X), s.t. fⱼ(xⱼ) = vⱼ.
     * @param witness_polynomials list of polynomials fⱼ(X).
     * @param transcript
     * @return Output{OpeningClaim, WitnessPolynomial, Proof}
     */
    static ProverOutput<Params> reduce_prove(std::shared_ptr<CK> ck,
                                             std::span<const OpeningPair<Params>> opening_pairs,
                                             std::span<const Polynomial> witness_polynomials,
                                             ProverTranscript<Fr>& transcript)
    {
        Fr nu = transcript.get_challenge("Shplonk:nu");

        const size_t num_opening_pairs = opening_pairs.size();

        // Find n, the maximum size of all polynomials fⱼ(X)
        size_t max_poly_size{ 0 };
        for (const auto& poly : witness_polynomials) {
            max_poly_size = std::max(max_poly_size, poly.size());
        }
        // Q(X) = ∑ⱼ ρʲ ⋅ ( fⱼ(X) − vⱼ) / ( X − xⱼ )
        Polynomial Q(max_poly_size);
        Polynomial tmp(max_poly_size);

        Fr current_nu = Fr::one();
        for (size_t j = 0; j < num_opening_pairs; ++j) {
            // (Cⱼ, xⱼ, vⱼ)
            const auto& [query, evaluation] = opening_pairs[j];

            // tmp = ρʲ ⋅ ( fⱼ(X) − vⱼ) / ( X − xⱼ )
            tmp = witness_polynomials[j];
            tmp[0] -= evaluation;
            tmp.factor_roots(query);

            Q.add_scaled(tmp, current_nu);
            current_nu *= nu;
        }

        // commit to Q(X) and add [Q] to the transcript
        auto Q_commitment = ck->commit(Q);
        transcript.send_to_verifier("Shplonk:Q", Q_commitment);

        // generate random evaluation challenge zeta_challenge
        const Fr z_challenge = transcript.get_challenge("Shplonk:z");

        // {ẑⱼ(r)}ⱼ , where ẑⱼ(r) = 1/zⱼ(r) = 1/(r - xⱼ)
        std::vector<Fr> inverse_vanishing_evals;
        inverse_vanishing_evals.reserve(num_opening_pairs);
        for (const auto& pair : opening_pairs) {
            inverse_vanishing_evals.emplace_back(z_challenge - pair.query);
        }
        Fr::batch_invert(inverse_vanishing_evals);

        // G(X) = Q(X) - Q_z(X) = Q(X) - ∑ⱼ ρʲ ⋅ ( fⱼ(X) − vⱼ) / ( r − xⱼ ),
        // s.t. G(r) = 0
        Polynomial& G = Q;

        // G₀ = ∑ⱼ ρʲ ⋅ vⱼ / ( r − xⱼ )
        current_nu = Fr::one();
        for (size_t j = 0; j < num_opening_pairs; ++j) {
            // (Cⱼ, xⱼ, vⱼ)
            const auto& [query, evaluation] = opening_pairs[j];

            // tmp = ρʲ ⋅ ( fⱼ(X) − vⱼ) / ( r − xⱼ )
            tmp = witness_polynomials[j];
            tmp[0] -= evaluation;
            Fr scaling_factor = current_nu * inverse_vanishing_evals[j]; // = ρʲ / ( r − xⱼ )

            // G -= ρʲ ⋅ ( fⱼ(X) − vⱼ) / ( r − xⱼ )
            G.add_scaled(tmp, -scaling_factor);

            current_nu *= nu;
        }

        // Return opening pair (z, 0) and polynomial G(X) = Q(X) - Q_z(X)
        return { .opening_pair = { .query = z_challenge, .evaluation = Fr::zero() }, .witness = std::move(G) };
    };

    /**
     * @brief Recomputes the new claim commitment [G] given the proof and
     * the challenge r. No verification happens so this function always succeeds.
     *
     * @param claims list of opening claims (Cⱼ, xⱼ, vⱼ) for a witness polynomial fⱼ(X), s.t. fⱼ(xⱼ) = vⱼ.
     * @param proof [Q(X)] = [ ∑ⱼ ρʲ ⋅ ( fⱼ(X) − vⱼ) / ( X − xⱼ ) ]
     * @param transcript
     * @return OpeningClaim
     */
    static OpeningClaim<Params> reduce_verify(std::span<const OpeningClaim<Params>> claims,
                                              VerifierTranscript<Fr>& transcript)
    {
        const size_t num_claims = claims.size();

        const Fr nu = transcript.get_challenge("Shplonk:nu");

        auto Q_commitment = transcript.template receive_from_prover<CommitmentAffine>("Shplonk:Q");

        const Fr z_challenge = transcript.get_challenge("Shplonk:z");

        // compute simulated commitment to [G] as a linear combination of
        // [Q], { [fⱼ] }, [1]:
        //  [G] = [Q] - ∑ⱼ (1/zⱼ(r))[Bⱼ]  + ( ∑ⱼ (1/zⱼ(r)) Tⱼ(r) )[1]
        //      = [Q] - ∑ⱼ (1/zⱼ(r))[Bⱼ]  +                    G₀ [1]

        // G₀ = ∑ⱼ ρʲ ⋅ vⱼ / ( r − xⱼ )
        Fr G_commitment_constant{ Fr::zero() };

        // [G] = [Q] - ∑ⱼ ρʲ / ( r − xⱼ )⋅[fⱼ] + G₀⋅[1]
        //     = [Q] - [∑ⱼ ρʲ ⋅ ( fⱼ(X) − vⱼ) / ( r − xⱼ )]
        Commitment G_commitment = Q_commitment;

        // {ẑⱼ(r)}ⱼ , where ẑⱼ(r) = 1/zⱼ(r) = 1/(r - xⱼ)
        std::vector<Fr> inverse_vanishing_evals;
        inverse_vanishing_evals.reserve(num_claims);
        for (const auto& claim : claims) {
            inverse_vanishing_evals.emplace_back(z_challenge - claim.opening_pair.query);
        }
        Fr::batch_invert(inverse_vanishing_evals);

        Fr current_nu{ Fr::one() };
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
        G_commitment += Commitment::one() * G_commitment_constant;

        // Return opening pair (z, 0) and commitment [G]
        return { { z_challenge, Fr::zero() }, G_commitment };
    };
};
} // namespace honk::pcs::shplonk
