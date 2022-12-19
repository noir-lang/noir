#pragma once

#include <polynomials/polynomial.hpp>
#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include <ecc/curves/bn254/pairing.hpp>

#include "./kate_commitment_scheme_data.hpp"
#include "../commitment_scheme.hpp"

/**
 * @brief Kate commitment scheme class. Conforms to the CommitmentScheme specification (TODO make into a concept)
 *
 * @tparam CommitmentSchemeData parametrises how we represent the commitment scheme's data structures
 * @tparam ChallengeGenerator used to "send" the Verifier data and receive random challenges. Conforms to the
 * ChallengeGenerator concept
 *
 * @details We wish to loosely couple our commitment schemes to any proof system that uses them (e.g. Plonk, Honk!). We
 * wish minimise how much knowledge of the challenge generation process is required by the commitment scheme. The
 * ChallengeGenerator concept describes the minimal functionality required by the challenge module, its implementation
 * details are left to the proof system
 */
template <typename CommitmentSchemeData, typename ChallengeGenerator> class Kate {
  public:
    typedef typename CommitmentSchemeData::Fr Fr;
    typedef typename CommitmentSchemeData::G1 G1;
    typedef typename CommitmentSchemeData::Commitment Commitment;
    typedef typename CommitmentSchemeData::SRS SRS;
    typedef typename CommitmentSchemeData::VerifierSRS VerifierSRS;
    typedef typename CommitmentSchemeData::OpeningProof OpeningProof;
    typedef typename CommitmentSchemeData::OpeningSchema OpeningSchema;

    // using ChallengeGenerator = Fr (*)(const std::vector<Fr>&);

    /**
     * @brief Commit to a vector of polynomials
     *
     * @param polynomials The input polynomials in coefficient form
     * @param n The size of each polynomial (we assume all polynomials have the same size)
     * @param srs The structured reference string required to compute the opening proof group elements PI
     *
     * @return a vector of Commitments
     */
    static std::vector<Commitment> commit(const std::vector<Fr*>& polynomials,
                                          const size_t n,
                                          std::shared_ptr<SRS> const& srs)
    {
        std::vector<Commitment> commitments;
        commitments.reserve(polynomials.size());

        // TODO: need to prevent `pippenger_runtime_state` being constructed each time this method is called.
        // Should be a singleton :o
        auto pippenger_runtime_state = barretenberg::scalar_multiplication::pippenger_runtime_state(n);
        for (auto& poly : polynomials) {
            commitments.push_back(barretenberg::scalar_multiplication::pippenger_unsafe(
                poly, srs->get_monomials(), n, pippenger_runtime_state));
        }

        return commitments;
    }

    /**
     * @brief Compute an opening proof for opening multiple polynomials at multiple evaluation points
     *
     * @param opening_data Data required to compute the opening proof. See OpeningSchema comments for more details
     * @param srs The structured reference string required to compute the opening proof group elements PI
     * @param challenge_generator Used to 'send' the verifier proof outputs and receive random challenges
     *
     * @return an OpeningProof, containing information required to verify whether the polynomial evaluationsin
     * `opening_data` are correct
     *
     * @details At this point this method is called, it is assumed that all polynomials have been committed to,
     * the points at which the polynomials are being evaluated at have been computed
     * and the evaluations of the polynomials at these points have been computed
     */
    static OpeningProof batch_open(const OpeningSchema& opening_data,
                                   std::shared_ptr<SRS> srs,
                                   ChallengeGenerator& challenge_generator)
    {
        OpeningProof result;

        ASSERT(opening_data.variables.size() == 1);

        // Kate doesn't explicitly use shifted polynomials, instead we open at evaluation point * root of unity
        ASSERT(opening_data.shifted_evaluations.size() == 0);

        auto& polynomials = opening_data.polynomials;
        auto& variables = opening_data.variables[0];
        const auto n = opening_data.n;

        std::vector<Fr> flattened_evaluations;
        for (const auto& evals : opening_data.evaluations) {
            for (const auto& eval : evals) {
                flattened_evaluations.push_back(eval);
            }
        }
        const Fr nu = challenge_generator.generate_challenge(flattened_evaluations);

        // reserve some mem for our opening polys
        // TODO: use assigned_alloc
        std::vector<Fr> opening_poly_data(n * variables.size());
        for (auto& ele : opening_poly_data) {
            ele = 0;
        }
        std::vector<Fr*> opening_polynomials;
        for (size_t k = 0; k < variables.size(); ++k) {
            const auto variable = variables[k];

            // assign opening poly
            Fr* opening_poly = &opening_poly_data[k * n];
            opening_polynomials.push_back(opening_poly);

            // compute opening poly numerator
            Fr separator_challenge = 1;
            Fr opening_poly_eval(0);
            for (size_t j = 0; j < polynomials.size(); ++j) {

                const size_t num_evaluations_for_poly = opening_data.evaluations[j].size();
                if (num_evaluations_for_poly <= k) {
                    continue;
                }
                Fr* poly = polynomials[j];
                for (size_t i = 0; i < n; ++i) {
                    opening_poly[i] += poly[i] * separator_challenge;
                }
                opening_poly_eval += opening_data.evaluations[j][k] * separator_challenge;

                separator_challenge *= nu;
            }

            // step 2: divide by (X - z)
            Fr divisor = -variable.invert();
            opening_poly[0] = opening_poly[0] - opening_poly_eval;
            opening_poly[0] *= divisor;
            for (size_t i = 1; i < n; ++i) {
                opening_poly[i] -= opening_poly[i - 1];
                opening_poly[i] *= divisor;
            }
        }

        result.PI = commit(opening_polynomials, n, srs);
        result.commitments = opening_data.commitments;
        result.evaluations = opening_data.evaluations;
        result.variables = variables;
        return result;
    }

    /**
     * @brief Verify the correctness of an OpeningProof
     *
     * @param opening_proof A proof of correctness! See OpeningProof comments for more details
     * @param srs The structured reference string required to compute the opening proof group elements PI
     * @param challenge_generator Used to 'send' the verifier proof outputs and receive random challenges
     *
     * @return true/false depending on if the proof verifies
     */
    static bool batch_verify(const OpeningProof& opening_proof,
                             std::shared_ptr<VerifierSRS> const& srs,
                             ChallengeGenerator& challenge_generator)
    {
        // convenience variables - reference methods of `opening_proof` so we don't have to constantly type
        // `opening_proof.` everywhere
        auto& variables = opening_proof.variables;
        auto& evaluations = opening_proof.evaluations;
        auto& commitments = opening_proof.commitments;
        auto& PI = opening_proof.PI;

        std::vector<typename G1::affine_element> kate_opening_elements(commitments);

        std::vector<Fr> flattened_evaluations;
        for (const auto& evals : evaluations) {
            for (const auto& eval : evals) {
                flattened_evaluations.push_back(eval);
            }
        }
        const Fr nu = challenge_generator.generate_challenge(flattened_evaluations);

        // TODO: flatten into Fr vector
        // should probs add a method into Group for this (takes a std::vector<typename G1::affine_element> spits out
        // std::vector<Fr>)
        // TODO TODO: challenge generator needs to be able to accept both Fq and Fr!
        std::vector<Fr> foo;
        for (const auto& pi : PI) {
            foo.push_back(Fr(pi.x));
            foo.push_back(Fr(pi.y));
        }
        const Fr separator_challenge = challenge_generator.generate_challenge(foo);

        const size_t num_polynomials = commitments.size();
        ASSERT(evaluations.size() == commitments.size());

        std::vector<Fr> kate_opening_scalars;

        Fr poly_separator_challenge = 1;
        Fr batch_evaluation_scalar = 0;

        // 1. compute the scalar multipliers we need to apply to each of our commitments as part of the Kate
        // verification algorithm when batching commitments [P0], ..., [Pn] we use challenge `nu` to create random
        // linear combination: [P] = nu^0*[P0] + nu^1*[P1] + ... + nu^n*[Pn]
        // 2. compute the `batch_evaluation_scalar`, the scalar multiplier we apply to the generator point.
        for (size_t i = 0; i < num_polynomials; ++i) {
            Fr opening_scalar = 0;
            // num evaluations at this poly
            Fr opening_separator_challenge = 1;
            for (size_t j = 0; j < evaluations[i].size(); ++j) {
                opening_scalar += (opening_separator_challenge);
                batch_evaluation_scalar += (evaluations[i][j] * opening_separator_challenge * poly_separator_challenge);
                opening_separator_challenge *= separator_challenge;
            }
            opening_scalar *= poly_separator_challenge;
            kate_opening_scalars.push_back(opening_scalar);
            poly_separator_challenge *= nu;
        }
        kate_opening_elements.push_back(G1::affine_one);
        kate_opening_scalars.push_back(-batch_evaluation_scalar);

        // compute the scalar multipliers we must apply to the opening proof group elements `PI`
        // in both the left hand and right hand pairing argument
        std::vector<typename G1::affine_element> kate_rhs_opening_elements;
        std::vector<Fr> kate_rhs_opening_scalars;
        Fr opening_separator_challenge = 1;
        for (size_t i = 0; i < PI.size(); ++i) {
            kate_opening_elements.push_back(PI[i]);
            kate_opening_scalars.push_back(variables[i] * opening_separator_challenge);
            kate_rhs_opening_elements.push_back(PI[i]);
            kate_rhs_opening_scalars.push_back(-opening_separator_challenge);
            opening_separator_challenge *= separator_challenge;
        }

        typename G1::element P[2];
        P[0] = typename G1::element(kate_opening_elements[0]) * kate_opening_scalars[0];
        P[1] = typename G1::element(kate_rhs_opening_elements[0]) * kate_rhs_opening_scalars[0];

        for (size_t i = 1; i < kate_opening_elements.size(); ++i) {
            P[0] += typename G1::element(kate_opening_elements[i]) * kate_opening_scalars[i];
        }
        for (size_t i = 1; i < kate_rhs_opening_elements.size(); ++i) {
            P[1] += typename G1::element(kate_rhs_opening_elements[i]) * kate_rhs_opening_scalars[i];
        }
        G1::element::batch_normalize(P, 2);

        typename G1::affine_element P_affine[2]{
            { P[0].x, P[0].y },
            { P[1].x, P[1].y },
        };

        // The final pairing check of step 12.
        // TODO: try to template parametrise the pairing + fq12 output :/
        barretenberg::fq12 result =
            barretenberg::pairing::reduced_ate_pairing_batch_precomputed(P_affine, srs->get_precomputed_g2_lines(), 2);

        return (result == barretenberg::fq12::one());
    }
};