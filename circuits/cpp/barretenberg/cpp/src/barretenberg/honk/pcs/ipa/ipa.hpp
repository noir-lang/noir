#pragma once
#include "barretenberg/common/assert.hpp"
#include "barretenberg/ecc/scalar_multiplication/scalar_multiplication.hpp"
#include "barretenberg/honk/pcs/claim.hpp"
#include "barretenberg/honk/pcs/verification_key.hpp"
#include "barretenberg/honk/transcript/transcript.hpp"
#include <cstddef>
#include <numeric>
#include <string>
#include <vector>

/**
 * @brief IPA (inner-product argument) commitment scheme class. Conforms to the specification
 * https://hackmd.io/q-A8y6aITWyWJrvsGGMWNA?view.
 *
 */
namespace proof_system::honk::pcs::ipa {
template <typename Curve> class IPA {
    using Fr = typename Curve::ScalarField;
    using GroupElement = typename Curve::Element;
    using Commitment = typename Curve::AffineElement;
    using CK = CommitmentKey<Curve>;
    using VK = VerifierCommitmentKey<Curve>;
    using Polynomial = barretenberg::Polynomial<Fr>;

  public:
    /**
     * @brief Compute an inner product argument proof for opening a single polynomial at a single evaluation point
     *
     * @param ck The commitment key containing srs and pippenger_runtime_state for computing MSM
     * @param opening_pair (challenge, evaluation)
     * @param polynomial The witness polynomial whose opening proof needs to be computed
     * @param transcript Prover transcript
     */
    static void compute_opening_proof(std::shared_ptr<CK> ck,
                                      const OpeningPair<Curve>& opening_pair,
                                      const Polynomial& polynomial,
                                      ProverTranscript<Fr>& transcript)
    {
        ASSERT(opening_pair.challenge != 0 && "The challenge point should not be zero");
        auto poly_degree = static_cast<size_t>(polynomial.size());
        transcript.send_to_verifier("IPA:poly_degree", static_cast<uint64_t>(poly_degree));
        Fr generator_challenge = transcript.get_challenge("IPA:generator_challenge");
        auto aux_generator = Commitment::one() * generator_challenge;

        // Checks poly_degree is greater than zero and a power of two
        // In the future, we might want to consider if non-powers of two are needed
        ASSERT((poly_degree > 0) && (!(poly_degree & (poly_degree - 1))) &&
               "The poly_degree should be positive and a power of two");

        auto a_vec = polynomial;
        auto srs_elements = ck->srs->get_monomial_points();
        std::vector<Commitment> G_vec_local(poly_degree);
        // The SRS stored in the commitment key is the result after applying the pippenger point table so the
        // values at odd indices contain the point {srs[i-1].x * beta, srs[i-1].y}, where beta is the endomorphism
        // G_vec_local should use only the original SRS thus we extract only the even indices.
        for (size_t i = 0; i < poly_degree * 2; i += 2) {
            G_vec_local[i >> 1] = srs_elements[i];
        }
        std::vector<Fr> b_vec(poly_degree);
        Fr b_power = 1;
        for (size_t i = 0; i < poly_degree; i++) {
            b_vec[i] = b_power;
            b_power *= opening_pair.challenge;
        }
        // Iterate for log(poly_degree) rounds to compute the round commitments.
        auto log_poly_degree = static_cast<size_t>(numeric::get_msb(poly_degree));
        std::vector<GroupElement> L_elements(log_poly_degree);
        std::vector<GroupElement> R_elements(log_poly_degree);
        std::size_t round_size = poly_degree;

        // TODO(#479): restructure IPA so it can be integrated with the pthread alternative to work queue (or even the
        // work queue itself). Investigate whether parallelising parts of each rounds of IPA rounds brings significant
        // improvements and see if reducing the size of G_vec_local and b_vec by taking the first iteration out of the
        // loop can also be integrated.
        for (size_t i = 0; i < log_poly_degree; i++) {
            round_size >>= 1;
            // Compute inner_prod_L := < a_vec_lo, b_vec_hi > and inner_prod_R := < a_vec_hi, b_vec_lo >
            Fr inner_prod_L = Fr::zero();
            Fr inner_prod_R = Fr::zero();
            for (size_t j = 0; j < round_size; j++) {
                inner_prod_L += a_vec[j] * b_vec[round_size + j];
                inner_prod_R += a_vec[round_size + j] * b_vec[j];
            }
            // L_i = < a_vec_lo, G_vec_hi > + inner_prod_L * aux_generator
            L_elements[i] =
                // TODO(#473)
                barretenberg::scalar_multiplication::pippenger_without_endomorphism_basis_points<Curve>(
                    &a_vec[0], &G_vec_local[round_size], round_size, ck->pippenger_runtime_state);
            L_elements[i] += aux_generator * inner_prod_L;

            // R_i = < a_vec_hi, G_vec_lo > + inner_prod_R * aux_generator
            // TODO(#473)
            R_elements[i] = barretenberg::scalar_multiplication::pippenger_without_endomorphism_basis_points<Curve>(
                &a_vec[round_size], &G_vec_local[0], round_size, ck->pippenger_runtime_state);
            R_elements[i] += aux_generator * inner_prod_R;

            std::string index = std::to_string(i);
            transcript.send_to_verifier("IPA:L_" + index, Commitment(L_elements[i]));
            transcript.send_to_verifier("IPA:R_" + index, Commitment(R_elements[i]));

            // Generate the round challenge.
            const Fr round_challenge = transcript.get_challenge("IPA:round_challenge_" + index);
            const Fr round_challenge_inv = round_challenge.invert();

            std::vector<Commitment> G_lo(G_vec_local.begin(), G_vec_local.begin() + static_cast<long>(round_size));
            std::vector<Commitment> G_hi(G_vec_local.begin() + static_cast<long>(round_size), G_vec_local.end());
            G_lo = GroupElement::batch_mul_with_endomorphism(G_lo, round_challenge_inv);
            G_hi = GroupElement::batch_mul_with_endomorphism(G_hi, round_challenge);

            // Update the vectors a_vec, b_vec and G_vec.
            // a_vec_next = a_vec_lo * round_challenge + a_vec_hi * round_challenge_inv
            // b_vec_next = b_vec_lo * round_challenge_inv + b_vec_hi * round_challenge
            // G_vec_next = G_vec_lo * round_challenge_inv + G_vec_hi * round_challenge
            for (size_t j = 0; j < round_size; j++) {
                a_vec[j] *= round_challenge;
                a_vec[j] += round_challenge_inv * a_vec[round_size + j];
                b_vec[j] *= round_challenge_inv;
                b_vec[j] += round_challenge * b_vec[round_size + j];

                G_vec_local[j] = G_lo[j] + G_hi[j];
            }
        }

        transcript.send_to_verifier("IPA:a_0", a_vec[0]);
    }

    /**
     * @brief Verify the correctness of a Proof
     *
     * @param vk Verification_key containing srs and pippenger_runtime_state to be used for MSM
     * @param proof The proof containg L_vec, R_vec and a_zero
     * @param pub_input Data required to verify the proof
     *
     * @return true/false depending on if the proof verifies
     */
    static bool verify(std::shared_ptr<VK> vk,
                       const OpeningClaim<Curve>& opening_claim,
                       VerifierTranscript<Fr>& transcript)
    {
        auto poly_degree = static_cast<size_t>(transcript.template receive_from_prover<uint64_t>("IPA:poly_degree"));
        Fr generator_challenge = transcript.get_challenge("IPA:generator_challenge");
        auto aux_generator = Commitment::one() * generator_challenge;

        auto log_poly_degree = static_cast<size_t>(numeric::get_msb(poly_degree));

        // Compute C_prime
        GroupElement C_prime = opening_claim.commitment + (aux_generator * opening_claim.opening_pair.evaluation);

        // Compute C_zero = C_prime + ∑_{j ∈ [k]} u_j^2L_j + ∑_{j ∈ [k]} u_j^{-2}R_j
        auto pippenger_size = 2 * log_poly_degree;
        std::vector<Fr> round_challenges(log_poly_degree);
        std::vector<Fr> round_challenges_inv(log_poly_degree);
        std::vector<Commitment> msm_elements(pippenger_size);
        std::vector<Fr> msm_scalars(pippenger_size);
        for (size_t i = 0; i < log_poly_degree; i++) {
            std::string index = std::to_string(i);
            auto element_L = transcript.template receive_from_prover<Commitment>("IPA:L_" + index);
            auto element_R = transcript.template receive_from_prover<Commitment>("IPA:R_" + index);
            round_challenges[i] = transcript.get_challenge("IPA:round_challenge_" + index);
            round_challenges_inv[i] = round_challenges[i].invert();

            msm_elements[2 * i] = element_L;
            msm_elements[2 * i + 1] = element_R;
            msm_scalars[2 * i] = round_challenges[i].sqr();
            msm_scalars[2 * i + 1] = round_challenges_inv[i].sqr();
        }
        // TODO(#473)
        GroupElement LR_sums = barretenberg::scalar_multiplication::pippenger_without_endomorphism_basis_points<Curve>(
            &msm_scalars[0], &msm_elements[0], pippenger_size, vk->pippenger_runtime_state);
        GroupElement C_zero = C_prime + LR_sums;

        /**
         * Compute b_zero where b_zero can be computed using the polynomial:
         *
         * g(X) = ∏_{i ∈ [k]} (u_{k-i}^{-1} + u_{k-i}.X^{2^{i-1}}).
         *
         * b_zero = g(evaluation) = ∏_{i ∈ [k]} (u_{k-i}^{-1} + u_{k-i}. (evaluation)^{2^{i-1}})
         */
        Fr b_zero = Fr::one();
        for (size_t i = 0; i < log_poly_degree; i++) {
            auto exponent = static_cast<uint64_t>(Fr(2).pow(i));
            b_zero *= round_challenges_inv[log_poly_degree - 1 - i] +
                      (round_challenges[log_poly_degree - 1 - i] * opening_claim.opening_pair.challenge.pow(exponent));
        }

        // Compute G_zero
        // First construct s_vec
        std::vector<Fr> s_vec(poly_degree);
        for (size_t i = 0; i < poly_degree; i++) {
            Fr s_vec_scalar = Fr::one();
            for (size_t j = (log_poly_degree - 1); j != size_t(-1); j--) {
                auto bit = (i >> j) & 1;
                bool b = static_cast<bool>(bit);
                if (b) {
                    s_vec_scalar *= round_challenges[log_poly_degree - 1 - j];
                } else {
                    s_vec_scalar *= round_challenges_inv[log_poly_degree - 1 - j];
                }
            }
            s_vec[i] = s_vec_scalar;
        }
        auto srs_elements = vk->srs->get_monomial_points();
        // Copy the G_vector to local memory.
        std::vector<Commitment> G_vec_local(poly_degree);
        // The SRS stored in the commitment key is the result after applying the pippenger point table so the
        // values at odd indices contain the point {srs[i-1].x * beta, srs[i-1].y}, where beta is the endomorphism
        // G_vec_local should use only the original SRS thus we extract only the even indices.
        for (size_t i = 0; i < poly_degree * 2; i += 2) {
            G_vec_local[i >> 1] = srs_elements[i];
        }
        // TODO(#473)
        auto G_zero = barretenberg::scalar_multiplication::pippenger_without_endomorphism_basis_points<Curve>(
            &s_vec[0], &G_vec_local[0], poly_degree, vk->pippenger_runtime_state);

        auto a_zero = transcript.template receive_from_prover<Fr>("IPA:a_0");

        GroupElement right_hand_side = G_zero * a_zero + aux_generator * a_zero * b_zero;

        return (C_zero.normalize() == right_hand_side.normalize());
    }
};

} // namespace proof_system::honk::pcs::ipa