#pragma once
#include <cstddef>
#include <numeric>
#include <string>
#include "barretenberg/ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp"
#include "barretenberg/honk/pcs/commitment_key.hpp"
#include "barretenberg/stdlib/primitives/curves/bn254.hpp"

// Suggested by Zac: Future optimisations
// 1: write a program that generates a large set of generator points (2^23?) and writes to a file on disk
// 2: create a SRS class for IPA similar to existing SRS class, that loads these points from disk
//    and stores in struct *and* applies the pippenger point table endomorphism transforation
// 3: when constructing a InnerProductArgument class, pass std::shared_ptr<SRS> as input param and store as member
// variable
// 4: (SRS class should contain a `pippenger_runtime_state` object so it does not need to be repeatedly
// generated)

/**
 * @brief IPA (inner-product argument) commitment scheme class. Conforms to the specification
 * https://hackmd.io/q-A8y6aITWyWJrvsGGMWNA?view.
 *
 */
namespace proof_system::honk::pcs::ipa {
template <typename Params> class InnerProductArgument {
    using Fr = typename Params::Fr;
    using Commitment = typename Params::Commitment;
    using CommitmentAffine = typename Params::C;
    using CK = typename Params::CK;
    using VK = typename Params::VK;
    using Polynomial = barretenberg::Polynomial<Fr>;

  public:
    /**
     * @brief Compute a proof for opening a single polynomial at a single evaluation point
     *
     * @param ck The commitment key containing srs and pippenger_runtime_state for computing MSM
     * @param opening_pair (challenge, evaluation)
     * @param polynomial The witness polynomial whose opening proof needs to be computed
     * @param transcript Prover transcript
     */
    static void reduce_prove(std::shared_ptr<CK> ck,
                             const OpeningPair<Params>& opening_pair,
                             const Polynomial& polynomial,
                             ProverTranscript<Fr>& transcript)
    {
        Fr generator_challenge = transcript.get_challenge("IPA:generator_challenge");
        auto aux_generator = CommitmentAffine::one() * generator_challenge;

        ASSERT(opening_pair.challenge != 0 && "The challenge point should not be zero");
        const size_t poly_degree = polynomial.size();
        // To check poly_degree is greater than zero and a power of two
        // TODO(#220)(Arijit): To accomodate non power of two poly_degree
        ASSERT((poly_degree > 0) && (!(poly_degree & (poly_degree - 1))) &&
               "The poly_degree should be positive and a power of two");

        auto a_vec = polynomial;
        // TODO(#220)(Arijit): to make it more efficient by directly using G_vector for the input points when i = 0 and
        // write the output points to G_vec_local. Then use G_vec_local for rounds where i>0, this can be done after we
        // use SRS instead of G_vector.
        auto srs_elements = ck->srs.get_monomial_points();
        std::vector<CommitmentAffine> G_vec_local(poly_degree);
        for (size_t i = 0; i < poly_degree; i++) {
            G_vec_local[i] = srs_elements[i];
        }
        // Construct b vector
        // TODO(#220)(Arijit): For round i=0, b_vec can be derived in-place.
        // This means that the size of b_vec can be 50% of the current size (i.e. we only write values to b_vec at the
        // end of round 0)
        std::vector<Fr> b_vec(poly_degree);
        Fr b_power = 1;
        for (size_t i = 0; i < poly_degree; i++) {
            b_vec[i] = b_power;
            b_power *= opening_pair.challenge;
        }
        // Iterate for log_2(poly_degree) rounds to compute the round commitments.
        const size_t log_poly_degree = static_cast<size_t>(numeric::get_msb(poly_degree));
        std::vector<Commitment> L_elements(log_poly_degree);
        std::vector<Commitment> R_elements(log_poly_degree);
        size_t round_size = poly_degree;

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
            L_elements[i] = barretenberg::scalar_multiplication::pippenger_without_endomorphism_basis_points(
                &a_vec[0], &G_vec_local[round_size], round_size, ck->pippenger_runtime_state);
            L_elements[i] += aux_generator * inner_prod_L;

            // R_i = < a_vec_hi, G_vec_lo > + inner_prod_R * aux_generator
            R_elements[i] = barretenberg::scalar_multiplication::pippenger_without_endomorphism_basis_points(
                &a_vec[round_size], &G_vec_local[0], round_size, ck->pippenger_runtime_state);
            R_elements[i] += aux_generator * inner_prod_R;

            std::string index = std::to_string(i);
            transcript.send_to_verifier("IPA:L_" + index, CommitmentAffine(L_elements[i]));
            transcript.send_to_verifier("IPA:R_" + index, CommitmentAffine(R_elements[i]));

            // Generate the round challenge.
            const Fr round_challenge = transcript.get_challenge("IPA:round_challenge_" + index);
            const Fr round_challenge_inv = round_challenge.invert();

            // Update the vectors a_vec, b_vec and G_vec.
            // a_vec_next = a_vec_lo * round_challenge + a_vec_hi * round_challenge_inv
            // b_vec_next = b_vec_lo * round_challenge_inv + b_vec_hi * round_challenge
            // G_vec_next = G_vec_lo * round_challenge_inv + G_vec_hi * round_challenge
            for (size_t j = 0; j < round_size; j++) {
                a_vec[j] *= round_challenge;
                a_vec[j] += round_challenge_inv * a_vec[round_size + j];
                b_vec[j] *= round_challenge_inv;
                b_vec[j] += round_challenge * b_vec[round_size + j];

                /*
                TODO(#220)(Arijit): (performance improvement suggested by Zac): We can improve performance here by using
                element::batch_mul_with_endomorphism. This method takes a vector of input points points and a scalar x
                and outputs a vector containing points[i]*x. It's 30% faster than a basic mul operation due to
                performing group additions in 2D affine coordinates instead of 3D projective coordinates (affine point
                additions are usually more expensive than projective additions due to the need to compute a modular
                inverse. However we get around this by computing a single batch inverse. This only works if you are
                adding a lot of independent point pairs so you can amortise the cost of the single batch inversion
                across multiple points).
                */

                auto G_lo = Commitment(G_vec_local[j]) * round_challenge_inv;
                auto G_hi = Commitment(G_vec_local[round_size + j]) * round_challenge;
                auto temp = G_lo + G_hi;
                G_vec_local[j] = temp.normalize();
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
    static bool reduce_verify(std::shared_ptr<VK> vk,
                              const OpeningPair<Params>& opening_pair,
                              size_t poly_degree,
                              VerifierTranscript<Fr>& transcript)
    {
        auto commitment = transcript.template receive_from_prover<CommitmentAffine>("IPA:C");

        Fr generator_challenge = transcript.get_challenge("IPA:generator_challenge");
        auto aux_generator = CommitmentAffine::one() * generator_challenge;

        size_t log_poly_degree = numeric::get_msb(poly_degree);

        // Compute C_prime
        Commitment C_prime = commitment + (aux_generator * opening_pair.evaluation);

        // Compute C_zero = C_prime + ∑_{j ∈ [k]} u_j^2L_j + ∑_{j ∈ [k]} u_j^{-2}R_j
        const size_t pippenger_size = 2 * log_poly_degree;
        std::vector<Fr> round_challenges(log_poly_degree);
        std::vector<Fr> round_challenges_inv(log_poly_degree);
        std::vector<CommitmentAffine> msm_elements(pippenger_size);
        std::vector<Fr> msm_scalars(pippenger_size);
        for (size_t i = 0; i < log_poly_degree; i++) {
            std::string index = std::to_string(i);
            auto element_L = transcript.template receive_from_prover<CommitmentAffine>("IPA:L_" + index);
            auto element_R = transcript.template receive_from_prover<CommitmentAffine>("IPA:R_" + index);
            round_challenges[i] = transcript.get_challenge("IPA:round_challenge_" + index);
            round_challenges_inv[i] = round_challenges[i].invert();

            msm_elements[2 * i] = element_L;
            msm_elements[2 * i + 1] = element_R;
            msm_scalars[2 * i] = round_challenges[i].sqr();
            msm_scalars[2 * i + 1] = round_challenges_inv[i].sqr();
        }
        Commitment LR_sums = barretenberg::scalar_multiplication::pippenger_without_endomorphism_basis_points(
            &msm_scalars[0], &msm_elements[0], pippenger_size, vk->pippenger_runtime_state);
        Commitment C_zero = C_prime + LR_sums;

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
                      (round_challenges[log_poly_degree - 1 - i] * opening_pair.challenge.pow(exponent));
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
        auto srs_elements = vk->srs.get_monomial_points();
        // Copy the G_vector to local memory.
        std::vector<CommitmentAffine> G_vec_local(poly_degree);
        for (size_t i = 0; i < poly_degree; i++) {
            G_vec_local[i] = srs_elements[i];
        }
        auto G_zero = barretenberg::scalar_multiplication::pippenger_without_endomorphism_basis_points(
            &s_vec[0], &G_vec_local[0], poly_degree, vk->pippenger_runtime_state);

        auto a_zero = transcript.template receive_from_prover<Fr>("IPA:a_0");

        Commitment right_hand_side = G_zero * a_zero + aux_generator * a_zero * b_zero;

        return (C_zero.normalize() == right_hand_side.normalize());
    }
};

} // namespace proof_system::honk::pcs::ipa
