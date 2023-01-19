#pragma once
#include "shplonk.hpp"
#include "honk/pcs/commitment_key.hpp"

namespace honk::pcs::shplonk {

/**
 * @brief Open several polynomials, each at a different set of points.
 * Reduces to opening a single univariate at a single point.
 *
 * @tparam Params for the given commitment scheme
 */
template <typename Params> class MultiBatchOpeningScheme {
    using CK = typename Params::CK;

    using Fr = typename Params::Fr;
    using Commitment = typename Params::Commitment;
    using Polynomial = barretenberg::Polynomial<Fr>;

  public:
    /**
     * @brief Batches a list of 'MultiOpeningClaim' into a single 'OpeningClaim' suitable for an polynomial opening
     * scheme.
     *
     * @param ck CommitmentKey
     * @param multi_claims a set of commitments and evaluation queries claimed
     * @param witness_polynomials witnesses corresponsing to the commitments in 'batch_claim'.
     *        Note that the order of the polynomials must follow the order of the commitments
     *        in 'batch_claim'.
     * @param transcript
     * @return Output{OpeningClaim, WitnessPolynomial, Proof}
     */
    static ProverOutput<Params> reduce_prove(std::shared_ptr<CK> ck,
                                             std::span<const MultiOpeningClaim<Params>> multi_claims,
                                             std::span<const Polynomial> witness_polynomials,
                                             const auto& transcript)
    {
        transcript->apply_fiat_shamir("nu");
        Fr nu = Fr::serialize_from_buffer(transcript->get_challenge("nu").begin());

        const size_t num_multi_claims = multi_claims.size();

        // allocate the merged polynomials Bₖ(X) and compute them
        // while we could simply allocate vectors of the same size,
        // for Gemini we know that the sizes are powers of 2, so we'd
        // be allocating nlogn space instead of 2n
        std::vector<Polynomial> merged_polynomials;
        merged_polynomials.reserve(num_multi_claims);
        size_t max_poly_size{ 0 };

        // allocate polynomial Tₖ(X) interpolating Yₖ over Ωₖ, where
        // Yₖ = {yᵏ₁, …, yᵏₘₖ}, where yᵏᵢ = ∑ⱼ ρʲ⋅yʲᵢ for all i in 1,…,mₖ
        // is the random linear combination of all evaluations openened at the same opening
        // set Ωₖ
        std::vector<Polynomial> interpolated_polynomials;
        interpolated_polynomials.reserve(num_multi_claims);

        // initialize the {Cₖ}
        // we will then set them to Cₖ = ∑ⱼ ρʲ⋅Cⱼ for all commitments Cⱼ
        // belonging to the k-th opening set.
        std::vector<Commitment> merged_commitments;
        merged_commitments.reserve(num_multi_claims);

        // keep track of number of polynomials we have mergeds
        size_t current_poly_idx = 0;

        Fr current_nu = Fr::one();
        // iterate over all claims, grouped by common opening set
        for (const auto& [queries_k, openings_k] : multi_claims) {
            const size_t num_queries_k = queries_k.size();
            const size_t num_openings_k = openings_k.size();

            // get the max size of each polynomial being opened at Ωₖ
            size_t merged_poly_size{ 0 };
            for (size_t sub_claim_idx = 0; sub_claim_idx < num_openings_k; sub_claim_idx++) {
                merged_poly_size =
                    std::max(merged_poly_size, witness_polynomials[current_poly_idx + sub_claim_idx].size());
            }

            // we loop again, this time actully computing the linear combination of
            // - Bₖ(X) =  ∑ⱼ ρʲ⋅pⱼ(X)
            // - Cₖ = ∑ⱼ ρʲ⋅Cⱼ
            // - Yₖ = {yᵏ₁, …, yᵏₘₖ}, where yᵏᵢ = ∑ⱼ ρʲ⋅yʲᵢ
            // create an empty polynomial in which we merge all polynomials openened at the same claim
            auto& B_k = merged_polynomials.emplace_back(Polynomial(size_t(0), merged_poly_size));
            auto& C_k = merged_commitments.emplace_back(Commitment::zero());
            std::vector<Fr> evals_k(num_queries_k, Fr::zero());

            for (const auto& [C_j, evals_j] : openings_k) {
                // add ρʲ⋅pⱼ to merged polynomial Bₖ(X)
                B_k.add_scaled(witness_polynomials[current_poly_idx], current_nu);
                // add ρʲ⋅Cⱼ to the merged commitment Cₖ
                C_k = C_k + (C_j * current_nu);
                for (size_t i = 0; i < num_queries_k; ++i) {
                    // add ρʲ⋅yʲᵢ to yᵏᵢ
                    evals_k[i] += current_nu * evals_j[i];
                }

                current_nu *= nu;
                current_poly_idx++;
            }
            // compute the interpolation polynomial Tₖ(X) of Yₖ over Ωₖ.
            // for each xᵏᵢ in Ωₖ, we have yᵏᵢ = Bₖ(xᵏᵢ) = Tₖ(xᵏᵢ),
            interpolated_polynomials.emplace_back(Polynomial(queries_k, evals_k));

            // we update the max size across all polys
            max_poly_size = std::max(merged_poly_size, max_poly_size);
        }

        // initialize Q(X) = 0
        Polynomial Q(size_t(0), max_poly_size);
        Polynomial tmp(size_t(0), max_poly_size);
        for (size_t k = 0; k < num_multi_claims; ++k) {
            // Bₖ(X) into temp_poly
            tmp = merged_polynomials[k];
            // subtract Bₖ(X) - Tₖ(X) in-place
            tmp -= interpolated_polynomials[k];

            // compute ( Bₖ(X) - Tₖ(X) ) / zₖ(X)
            tmp.factor_roots(multi_claims[k].queries);
            Q += tmp;
        }

        // commit to Q(X) and add [Q] to the transcript
        Commitment Q_commitment = ck->commit(Q);
        transcript->add_element("Q", static_cast<barretenberg::g1::affine_element>(Q_commitment).to_buffer());

        // generate random evaluation challenge zeta_challenge
        transcript->apply_fiat_shamir("z");
        const Fr zeta_challenge = Fr::serialize_from_buffer(transcript->get_challenge("z").begin());

        // reuse the quotient polynomial Q(X)
        // G(X) = Q(X)
        Polynomial& G = Q;

        // {ẑₖ(r)}ₖ , where ẑₖ(r) = 1/zₖ(r)
        std::vector<Fr> inverse_vanishing_evals;
        {
            for (const auto& [queries_k, openings_k] : multi_claims) {
                Fr eval{ Fr::one() };
                for (const Fr& x : queries_k) {
                    eval *= (zeta_challenge - x);
                }
                inverse_vanishing_evals.emplace_back(eval);
            }
            Fr::batch_invert(inverse_vanishing_evals);
        }

        // evaluations of interpolated polynomials Tₖ(r)
        std::vector<Fr> T_r(num_multi_claims);

        for (size_t k = 0; k < num_multi_claims; ++k) {
            // evaluate Tₖ(r)
            auto& T_k = interpolated_polynomials[k];
            T_r[k] = T_k.evaluate(zeta_challenge);

            // subtract ( Bₖ(X) − Tₖ(r) )/zₖ(r) from G(X)
            merged_polynomials[k][0] -= T_r[k];
            G.add_scaled(merged_polynomials[k], -inverse_vanishing_evals[k]);
        }

        // compute simulated commitment to [G] as
        //   [Q] - ∑ₖ (1/zₖ(r))[Bₖ] + ( ∑ₖ (1/zₖ(r)) Tₖ(r))[1]
        Commitment G_commitment = Q_commitment;
        {
            Fr G_commitment_constant = Fr::zero();

            for (size_t k = 0; k < num_multi_claims; ++k) {
                G_commitment += merged_commitments[k] * (-inverse_vanishing_evals[k]);
                G_commitment_constant += T_r[k] * inverse_vanishing_evals[k];
            }

            G_commitment += (Commitment::one() * G_commitment_constant);
        }

        return { .claim = OpeningClaim<Params>{ .commitment = G_commitment,
                                                .opening_point = zeta_challenge,
                                                .eval = Fr::zero() },
                 .witness = std::move(Q),
                 .proof = Q_commitment };
    };

    /**
     * @brief Recomputes the new claim commitment [G] given the proof and
     * the challenge r. No verification happens so this function always succeeds.
     *
     * @param multi_claims a set of commitments and evaluation queries claimed
     * @param proof [Q(X)]
     * @param transcript
     * @return OpeningClaim
     */
    static OpeningClaim<Params> reduce_verify(std::span<const MultiOpeningClaim<Params>> multi_claims,
                                              const Proof<Params>& proof,
                                              const auto& transcript)
    {
        const Fr nu = Fr::serialize_from_buffer(transcript->get_challenge("nu").begin());
        const Fr zeta_challenge = Fr::serialize_from_buffer(transcript->get_challenge("z").begin());

        // compute simulated commitment to [G] as
        //     [Q] - ∑ₖ (1/zₖ(r))[Bₖ]  + ( ∑ₖ (1/zₖ(r)) Tₖ(r) )[1]
        //  =  [Q] - ∑ₖ (1/zₖ(r))[Bₖ]  + ( ∑ₖ ( ∑ {xᵢ ∈ Ωₖ} yᵢ / ( dᵢ ⋅ (r−xᵢ) ) ) [1]

        // C₀ = ∑ₖ (1/zₖ(r)) Tₖ(r))
        Fr commitment_constant{ Fr::zero() };

        //  [Q] - ∑ₖ (1/zₖ(r)[Bₖ] + C₀[1]
        Commitment commitment = Commitment::zero();

        Fr current_nu{ Fr::one() };

        // iterate over all queries_k = Ωₖ, and openings_k = [(Cⱼ, Yⱼ)]ⱼ
        for (const auto& [queries_k, openings_k] : multi_claims) {
            const size_t num_queries = queries_k.size();

            // merged_evals is a vector Yₖ = {yᵏ₁, …, yᵏₘₖ}, where
            //   yᵏᵢ = ∑ⱼ ρʲ⋅yʲᵢ for all i in 1,…,mₖ
            // for all evaluations at the same opening set Ωₖ
            std::vector<Fr> merged_evals_k(num_queries, Fr::zero());
            // merged_commitment is the linear combination of all commitments
            // for polynomials openened  at the same opening set Ωₖ.
            // Cₖ = ∑ⱼ ρʲ⋅Cⱼ = [Bₖ]
            Commitment merged_commitment = Commitment::zero();
            // If we define the merged polynomial Bₖ(X) = ∑ⱼ ρʲ⋅pⱼ(X), then
            // - Cₖ = Commit(Bₖ(X)) for each k
            // - yᵏᵢ = Bₖ(xᵏᵢ) for all k,i
            for (const auto& [C_j, evals_j] : openings_k) {
                for (size_t i = 0; i < num_queries; ++i) {
                    merged_evals_k[i] += current_nu * evals_j[i];
                }
                merged_commitment += C_j * current_nu;
                current_nu *= nu;
            }

            // zₖ = ∏ (r − xᵢ), ranging over xᵢ ∈ Ωₖ.
            // It is the vanishing polynomial for the k-th opening set Ωₖ
            Fr z_k{ Fr::one() };

            // We have ẑₖ(r) Tₖ(r) = z(r)⋅( ∑ {xᵢ ∈ Ωₖ} yᵢ / ( dᵢ ⋅ (r−xᵢ) ) )
            // where dᵢ = ∏ { j ≠ i} (xᵢ − xⱼ)
            // so we compute ∑ {xᵢ ∈ Ωₖ} yᵢ / ( dᵢ ⋅ (r−xᵢ) ) for each claim
            // constant_k is the running sum computing the above
            Fr constant_k{ Fr::zero() };
            for (size_t i = 0; i < num_queries; ++i) {
                Fr x_i{ queries_k[i] };
                Fr y_i{ merged_evals_k[i] };
                z_k *= (zeta_challenge - x_i);

                // dᵢ = ∏ { j ≠ i} (xᵢ − xⱼ)
                Fr d_i{ Fr::one() };
                for (const auto x_j : queries_k) {
                    if (x_i != x_j) {
                        d_i *= (x_i - x_j);
                    } else {
                        d_i *= (zeta_challenge - x_i);
                    }
                }
                // NOTE: these inversions are expensive, but this method aims to mimick an
                // implementation inside a circuit.
                // moreover, the number of claims is usually small, so this shouldn't be a
                // big performance hit.
                constant_k += (y_i / d_i);
            }
            // add (1/zₖ(r)) Tₖ(r) to the constant term C₀
            commitment_constant += constant_k;

            // add (- ∑ₖ (1/zₖ(r))[Bₖ]) to [Q]
            commitment += merged_commitment * (-z_k.invert());
        }
        commitment += proof;
        commitment += Commitment::one() * commitment_constant;

        return { .commitment = commitment, .opening_point = zeta_challenge, .eval = Fr::zero() };
    };
};
} // namespace honk::pcs::shplonk