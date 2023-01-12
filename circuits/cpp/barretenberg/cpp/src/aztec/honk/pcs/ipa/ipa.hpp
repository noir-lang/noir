#pragma once
#include <numeric>
#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include "stdlib/primitives/curves/bn254.hpp"

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
 * @tparam Fr: The underlying field
 * @tparam Fq: The field corresponding to the elliptic curve
 * @tparam G1: The elliptic curve group
 */
template <typename Fr, typename Fq, typename G1> class InnerProductArgument {
  public:
    using element = typename G1::element;
    using affine_element = typename G1::affine_element;
    struct IpaProof {
        std::vector<affine_element> L_vec;
        std::vector<affine_element> R_vec;
        Fr a_zero;
    };
    // To contain the public inputs for IPA proof
    // For now we are including the aux_generator and round_challenges in public input. They will be computed by the
    // prover and the verifier by Fiat-Shamir when the challengeGenerator is defined.
    struct IpaPubInput {
        element commitment;
        Fr challenge_point;
        Fr evaluation;
        size_t poly_degree;
        element aux_generator;            // To be removed
        std::vector<Fr> round_challenges; // To be removed
    };

    /**
     * @brief Commit to a polynomial
     *
     * @param polynomial The input polynomial in the coefficient form
     * @param poly_degree The degree of the polynomial
     * @param G_vector The common set of generators required to compute the commitment, to be replaced by srs
     *
     * @return a group element
     */
    static element commit(std::vector<Fr>& polynomial, const size_t poly_degree, std::vector<affine_element>& G_vector)
    {
        auto pippenger_runtime_state = barretenberg::scalar_multiplication::pippenger_runtime_state(poly_degree);
        auto commitment = barretenberg::scalar_multiplication::pippenger_without_endomorphism_basis_points(
            &polynomial[0], &G_vector[0], poly_degree, pippenger_runtime_state);
        return commitment;
    }

    /**
     * @brief Compute an IpaProof for opening a single polynomial at a single evaluation point
     *
     * @param ipa_pub_input Data required to compute the opening proof. See spec for more details
     * @param polynomial The witness polynomial whose opening proof needs to be computed
     * @param G_vector the common set of generators, to be replaced by the srs
     *
     * @return an IpaProof, containing information required to verify whether the commitment is computed correctly and
     * the polynomial evaluation is correct in the given challenge point.
     */
    static IpaProof ipa_prove(const IpaPubInput& ipa_pub_input,
                              const std::vector<Fr>& polynomial,
                              const std::vector<affine_element>& G_vector)
    {
        IpaProof proof;
        auto& challenge_point = ipa_pub_input.challenge_point;
        ASSERT(challenge_point != 0 && "The challenge point should not be zero");
        const size_t poly_degree = ipa_pub_input.poly_degree;
        // To check poly_degree is greater than zero and a power of two
        // Todo: To accomodate non power of two poly_degree
        ASSERT((poly_degree > 0) && (!(poly_degree & (poly_degree - 1))) &&
               "The poly_degree should be positive and a power of two");
        auto& aux_generator = ipa_pub_input.aux_generator;
        auto a_vec = polynomial;
        // Todo:  to make it more efficient by directly using G_vector for the input points when i = 0 and write the
        // output points to G_vec_local. Then use G_vec_local for rounds where i>0, this can be done after we use SRS
        // instead of G_vector.
        std::vector<affine_element> G_vec_local(poly_degree);
        for (size_t i = 0; i < poly_degree; i++) {
            G_vec_local[i] = G_vector[i];
        }
        // Construct b vector
        // Todo: For round i=0, b_vec can be derived in-place.
        // This means that the size of b_vec can be 50% of the current size (i.e. we only write values to b_vec at the
        // end of round 0)
        std::vector<Fr> b_vec(poly_degree);
        Fr b_power = 1;
        for (size_t i = 0; i < poly_degree; i++) {
            b_vec[i] = b_power;
            b_power *= challenge_point;
        }
        // Iterate for log_2(poly_degree) rounds to compute the round commitments.
        const size_t log_poly_degree = static_cast<size_t>(numeric::get_msb(poly_degree));
        std::vector<element> L_elements(log_poly_degree);
        std::vector<element> R_elements(log_poly_degree);
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
            // Todo: Remove usage of multiple runtime_state, pass it as an element of the SRS.
            auto pippenger_runtime_state = barretenberg::scalar_multiplication::pippenger_runtime_state(round_size);
            element partial_L = barretenberg::scalar_multiplication::pippenger_without_endomorphism_basis_points(
                &a_vec[0], &G_vec_local[round_size], round_size, pippenger_runtime_state);
            partial_L += aux_generator * inner_prod_L;

            // R_i = < a_vec_hi, G_vec_lo > + inner_prod_R * aux_generator
            element partial_R = barretenberg::scalar_multiplication::pippenger_without_endomorphism_basis_points(
                &a_vec[round_size], &G_vec_local[0], round_size, pippenger_runtime_state);
            partial_R += aux_generator * inner_prod_R;

            L_elements[i] = affine_element(partial_L);
            R_elements[i] = affine_element(partial_R);

            // Generate the round challenge. Todo: Use Fiat-Shamir
            const Fr round_challenge = ipa_pub_input.round_challenges[i];
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
                Todo: (performance improvement suggested by Zac): We can improve performance here by using
                element::batch_mul_with_endomorphism. This method takes a vector of input points points and a scalar x
                and outputs a vector containing points[i]*x. It's 30% faster than a basic mul operation due to
                performing group additions in 2D affine coordinates instead of 3D projective coordinates (affine point
                additions are usually more expensive than projective additions due to the need to compute a modular
                inverse. However we get around this by computing a single batch inverse. This only works if you are
                adding a lot of independent point pairs so you can amortise the cost of the single batch inversion
                across multiple points).
                */

                auto G_lo = element(G_vec_local[j]) * round_challenge_inv;
                auto G_hi = element(G_vec_local[round_size + j]) * round_challenge;
                auto temp = G_lo + G_hi;
                G_vec_local[j] = temp.normalize();
            }
        }
        proof.L_vec = std::vector<affine_element>(log_poly_degree);
        proof.R_vec = std::vector<affine_element>(log_poly_degree);
        for (size_t i = 0; i < log_poly_degree; i++) {
            proof.L_vec[i] = L_elements[i];
            proof.R_vec[i] = R_elements[i];
        }
        proof.a_zero = a_vec[0];
        return proof;
    }

    /**
     * @brief Verify the correctness of an IpaProof
     *
     * @param ipa_proof The proof containg L_vec, R_vec and a_zero
     * @param Ipa_pub_input Data required to verify the ipa_proof
     * @param G_vector The common set of generators, to be replaced by the srs
     *
     * @return true/false depending on if the proof verifies
     */
    static bool ipa_verify(const IpaProof& ipa_proof,
                           const IpaPubInput& ipa_pub_input,
                           const std::vector<affine_element>& G_vector)
    {
        // Local copies of public inputs
        auto& a_zero = ipa_proof.a_zero;
        auto& commitment = ipa_pub_input.commitment;
        auto& challenge_point = ipa_pub_input.challenge_point;
        auto& evaluation = ipa_pub_input.evaluation;
        auto& poly_degree = ipa_pub_input.poly_degree;
        auto& aux_generator = ipa_pub_input.aux_generator;

        // Compute C_prime
        element C_prime = commitment + (aux_generator * evaluation);

        // Compute the round challeneges and their inverses.
        const size_t log_poly_degree = static_cast<size_t>(numeric::get_msb(poly_degree));
        std::vector<Fr> round_challenges(log_poly_degree);
        for (size_t i = 0; i < log_poly_degree; i++) {
            round_challenges[i] = ipa_pub_input.round_challenges[i];
        }
        std::vector<Fr> round_challenges_inv(log_poly_degree);
        for (size_t i = 0; i < log_poly_degree; i++) {
            round_challenges_inv[i] = ipa_pub_input.round_challenges[i];
        }
        Fr::batch_invert(&round_challenges_inv[0], log_poly_degree);
        std::vector<affine_element> L_vec(log_poly_degree);
        std::vector<affine_element> R_vec(log_poly_degree);
        for (size_t i = 0; i < log_poly_degree; i++) {
            L_vec[i] = ipa_proof.L_vec[i];
            R_vec[i] = ipa_proof.R_vec[i];
        }

        // Compute C_zero = C_prime + ∑_{j ∈ [k]} u_j^2L_j + ∑_{j ∈ [k]} u_j^{-2}R_j
        const size_t pippenger_size = 2 * log_poly_degree;
        std::vector<affine_element> msm_elements(pippenger_size);
        std::vector<Fr> msm_scalars(pippenger_size);
        for (size_t i = 0; i < log_poly_degree; i++) {
            msm_elements[size_t(2) * i] = L_vec[i];
            msm_elements[size_t(2) * i + 1] = R_vec[i];
            msm_scalars[size_t(2) * i] = round_challenges[i] * round_challenges[i];
            msm_scalars[size_t(2) * i + 1] = round_challenges_inv[i] * round_challenges_inv[i];
        }
        auto pippenger_runtime_state_1 = barretenberg::scalar_multiplication::pippenger_runtime_state(pippenger_size);
        element LR_sums = barretenberg::scalar_multiplication::pippenger_without_endomorphism_basis_points(
            &msm_scalars[0], &msm_elements[0], pippenger_size, pippenger_runtime_state_1);
        element C_zero = C_prime + LR_sums;

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
                      (round_challenges[log_poly_degree - 1 - i] * challenge_point.pow(exponent));
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
        // Copy the G_vector to local memory.
        std::vector<affine_element> G_vec_local(poly_degree);
        for (size_t i = 0; i < poly_degree; i++) {
            G_vec_local[i] = G_vector[i];
        }
        auto pippenger_runtime_state_2 = barretenberg::scalar_multiplication::pippenger_runtime_state(poly_degree);
        auto G_zero = barretenberg::scalar_multiplication::pippenger_without_endomorphism_basis_points(
            &s_vec[0], &G_vec_local[0], poly_degree, pippenger_runtime_state_2);
        element right_hand_side = G_zero * a_zero;
        Fr a_zero_b_zero = a_zero * b_zero;
        right_hand_side += aux_generator * a_zero_b_zero;
        return (C_zero.normalize() == right_hand_side.normalize());
    }
};