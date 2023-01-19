#pragma once
#include "shplonk.hpp"
#include "honk/pcs/commitment_key.hpp"

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
    using Polynomial = barretenberg::Polynomial<Fr>;

  public:
    /**
     * @brief Batches several single-point 'OpeningClaim' into a single 'OpeningClaim' suitable for
     * a univariate polynomial opening scheme.
     *
     * @param ck CommitmentKey
     * @param claims list of opening claims (Cⱼ, xⱼ, vⱼ) for a witness polynomial fⱼ(X), s.t. fⱼ(xⱼ) = vⱼ.
     * @param witness_polynomials list of polynomials fⱼ(X).
     * @param transcript
     * @return Output{OpeningClaim, WitnessPolynomial, Proof}
     */
    static ProverOutput<Params> reduce_prove(std::shared_ptr<CK> ck,
                                             std::span<const OpeningClaim<Params>> claims,
                                             std::span<const Polynomial> witness_polynomials,
                                             const auto& transcript)
    {
        transcript->apply_fiat_shamir("nu");
        Fr nu = Fr::serialize_from_buffer(transcript->get_challenge("nu").begin());

        const size_t num_claims = claims.size();

        // Find n, the maximum size of all polynomials fⱼ(X)
        size_t max_poly_size{ 0 };
        for (const auto& poly : witness_polynomials) {
            max_poly_size = std::max(max_poly_size, poly.size());
        }
        // Q(X) = ∑ⱼ ρʲ ⋅ ( fⱼ(X) − vⱼ) / ( X − xⱼ )
        Polynomial Q(max_poly_size, max_poly_size);
        Polynomial tmp(max_poly_size, max_poly_size);

        Fr current_nu = Fr::one();
        for (size_t j = 0; j < num_claims; ++j) {
            // (Cⱼ, xⱼ, vⱼ)
            const auto& [commitment_j, opening_j, eval_j] = claims[j];

            // tmp = ρʲ ⋅ ( fⱼ(X) − vⱼ) / ( X − xⱼ )
            tmp = witness_polynomials[j];
            tmp[0] -= eval_j;
            tmp.factor_roots(opening_j);

            Q.add_scaled(tmp, current_nu);
            current_nu *= nu;
        }

        // [Q]
        Commitment Q_commitment = ck->commit(Q);
        transcript->add_element("Q", static_cast<barretenberg::g1::affine_element>(Q_commitment).to_buffer());

        // generate random evaluation challenge zeta_challenge
        transcript->apply_fiat_shamir("z");
        const Fr zeta_challenge = Fr::serialize_from_buffer(transcript->get_challenge("z").begin());

        // {ẑⱼ(r)}ⱼ , where ẑⱼ(r) = 1/zⱼ(r) = 1/(r - xⱼ)
        std::vector<Fr> inverse_vanishing_evals;
        inverse_vanishing_evals.reserve(num_claims);
        {
            for (const auto& claim_j : claims) {
                inverse_vanishing_evals.emplace_back(zeta_challenge - claim_j.opening_point);
            }
            Fr::batch_invert(inverse_vanishing_evals);
        }

        // G(X) = Q(X) - ∑ⱼ ρʲ ⋅ ( fⱼ(X) − vⱼ) / ( r − xⱼ ),
        // s.t. G(r) = 0
        Polynomial& G = Q;

        // G₀ = ∑ⱼ ρʲ ⋅ vⱼ / ( r − xⱼ )
        Fr G_commitment_constant = Fr::zero();
        // [G] = [Q] - ∑ⱼ ρʲ / ( r − xⱼ )⋅[fⱼ] + G₀⋅[1]
        //     = [Q] - [∑ⱼ ρʲ ⋅ ( fⱼ(X) − vⱼ) / ( r − xⱼ )]
        Commitment G_commitment = Q_commitment;
        current_nu = Fr::one();
        for (size_t j = 0; j < num_claims; ++j) {
            // (Cⱼ, xⱼ, vⱼ)
            const auto& [commitment_j, opening_j, eval_j] = claims[j];

            // tmp = ρʲ ⋅ ( fⱼ(X) − vⱼ) / ( r − xⱼ )
            tmp = witness_polynomials[j];
            tmp[0] -= eval_j;
            Fr scaling_factor = current_nu * inverse_vanishing_evals[j]; // = ρʲ / ( r − xⱼ )

            // G -= ρʲ ⋅ ( fⱼ(X) − vⱼ) / ( r − xⱼ )
            G.add_scaled(tmp, -scaling_factor);

            // G₀ += ρʲ / ( r − xⱼ ) ⋅ vⱼ
            G_commitment_constant += scaling_factor * eval_j;
            // [G] -= ρʲ / ( r − xⱼ )⋅[fⱼ]
            G_commitment -= commitment_j * scaling_factor;

            current_nu *= nu;
        }
        // [G] += G₀⋅[1] = [G] + (∑ⱼ ρʲ ⋅ vⱼ / ( r − xⱼ ))⋅[1]
        G_commitment += Commitment::one() * G_commitment_constant;

        return { .claim = { .commitment = G_commitment, .opening_point = zeta_challenge, .eval = Fr::zero() },
                 .witness = std::move(G),
                 .proof = Q_commitment };
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
                                              const Proof<Params>& proof,
                                              const auto& transcript)
    {
        const size_t num_claims = claims.size();
        const Fr nu = Fr::serialize_from_buffer(transcript->get_challenge("nu").begin());
        const Fr zeta_challenge = Fr::serialize_from_buffer(transcript->get_challenge("z").begin());

        // compute simulated commitment to [G] as a linear combination of
        // [Q], { [fⱼ] }, [1]:
        //  [G] = [Q] - ∑ⱼ (1/zⱼ(r))[Bⱼ]  + ( ∑ⱼ (1/zⱼ(r)) Tⱼ(r) )[1]
        //      = [Q] - ∑ⱼ (1/zⱼ(r))[Bⱼ]  +                    G₀ [1]

        // G₀ = ∑ⱼ ρʲ ⋅ vⱼ / ( r − xⱼ )
        Fr G_commitment_constant{ Fr::zero() };

        // [G] = [Q] - ∑ⱼ ρʲ / ( r − xⱼ )⋅[fⱼ] + G₀⋅[1]
        //     = [Q] - [∑ⱼ ρʲ ⋅ ( fⱼ(X) − vⱼ) / ( r − xⱼ )]
        Commitment G_commitment = proof;

        // {ẑⱼ(r)}ⱼ , where ẑⱼ(r) = 1/zⱼ(r) = 1/(r - xⱼ)
        std::vector<Fr> inverse_vanishing_evals;
        inverse_vanishing_evals.reserve(num_claims);
        {
            for (const auto& claim_j : claims) {
                inverse_vanishing_evals.emplace_back(zeta_challenge - claim_j.opening_point);
            }
            Fr::batch_invert(inverse_vanishing_evals);
        }

        Fr current_nu{ Fr::one() };
        for (size_t j = 0; j < num_claims; ++j) {
            // (Cⱼ, xⱼ, vⱼ)
            const auto& [commitment_j, opening_j, eval_j] = claims[j];

            Fr scaling_factor = current_nu * inverse_vanishing_evals[j]; // = ρʲ / ( r − xⱼ )

            // G₀ += ρʲ / ( r − xⱼ ) ⋅ vⱼ
            G_commitment_constant += scaling_factor * eval_j;
            // [G] -= ρʲ / ( r − xⱼ )⋅[fⱼ]
            G_commitment -= commitment_j * scaling_factor;

            current_nu *= nu;
        }
        // [G] += G₀⋅[1] = [G] + (∑ⱼ ρʲ ⋅ vⱼ / ( r − xⱼ ))⋅[1]
        G_commitment += Commitment::one() * G_commitment_constant;

        return { .commitment = G_commitment, .opening_point = zeta_challenge, .eval = Fr::zero() };
    };
};
} // namespace honk::pcs::shplonk
