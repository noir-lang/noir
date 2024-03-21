#pragma once
#include "barretenberg/commitment_schemes/claim.hpp"
#include "barretenberg/commitment_schemes/verification_key.hpp"
#include "barretenberg/common/assert.hpp"
#include "barretenberg/ecc/scalar_multiplication/scalar_multiplication.hpp"
#include "barretenberg/transcript/transcript.hpp"
#include <cstddef>
#include <numeric>
#include <string>
#include <vector>

namespace bb {
// clang-format off

/**
 * @brief IPA (inner product argument) commitment scheme class.
 *
 *@details This implementation of IPA uses the optimized version that only multiplies half of the elements of each
 *vector in each prover round. The implementation uses:
 *
 *1. An SRS (Structured Reference String) \f$\vec{G}=(G_0,G_1...,G_{d-1})\f$ with \f$G_i ∈ E(\mathbb{F}_p)\f$ and
 *\f$\mathbb{F}_r\f$ - the scalar field of the elliptic curve as well as \f$G\f$ which is an independent generator on
 *the same curve.
 *2. A polynomial \f$f(x)=\sum_{i=0}^{d-1}f_ix^i\f$ over field \f$F_r\f$, where the polynomial degree \f$d-1\f$ is such
 *that \f$d=2^k\f$
 *
 *The opening and verification procedures expect that there already exists a commitment to \f$f(x)\f$ which is the
 *scalar product \f$[f(x)]=\langle\vec{f},\vec{G}\rangle\f$, where \f$\vec{f}=(f_0, f_1,..., f_{d-1})\f$​
 *
 * The opening procedure documentation can be found in the description of \link IPA::compute_opening_proof_internal
 compute_opening_proof_internal \endlink. The verification procedure documentation is in \link IPA::verify_internal
 verify_internal \endlink
 *
 * @tparam Curve
 *
 * @remark IPA is not a very intuitive algorithm, so here are a few things that might help internalize it:
 *
 *1. Originally we have two vectors \f$\vec{a}\f$ and \f$\vec{b}\f$, whose product we want to prove, but
 *the prover can't just send vector \f$\vec{a}\f$ to the verifier, it can only provide a commitment
 \f$\langle\vec{a},\vec{G}\rangle\f$
 *2. The verifier computes the \f$C'=C+\langle\vec{a},\vec{b}\rangle\cdot U\f$ to "bind" together the
 commitment and the evaluation (\f$C=\langle\vec{a},\vec{G}\rangle\f$ is the commitment and \f$U=u⋅G\f$ is the auxiliary
 generator independent from \f$\vec{G}\f$)
 *3. The prover wants to reduce the problem of verifying the inner product of
 \f$\vec{a}\f$, \f$\vec{b}\f$ of length
 *\f$n\f$ to a problem of verifying the IPA of 2 vectors \f$\vec{a}_{new}\f$, \f$\vec{b}_{new}\f$ of size
 *\f$\frac{n}{2}\f$​
 *4. If \f$\vec{a}_{new}=\vec{a}_{low}+\alpha\cdot \vec{a}_{high}\f$ and \f$\vec{b}_{new}=\vec{b}_{low}+\alpha^{-1}\cdot
 \vec{b}_{high}\f$, then \f$\langle \vec{a}_{new},\vec{b}_{new}\rangle = \langle\vec{a}_{low},\vec{b}_{low}\rangle +
 \alpha^{-1}\langle\vec{a}_{low},\vec{b}_{high}\rangle+\alpha \langle \vec{a}_{high},\vec{b}_{low}\rangle +
 \langle\vec{a}_{high},\vec{b}_{high}\rangle=
 \langle\vec{a},\vec{b}\rangle+\alpha^{-1}\langle\vec{a}_{low},\vec{b}_{high}\rangle+\alpha \langle
 \vec{a}_{high},\vec{b}_{low}\rangle\f$, so if we provide commitments to the cross-terms
 \f$\langle\vec{a}_{low},\vec{b}_{high}\rangle\f$ and \f$\langle \vec{a}_{high},\vec{b}_{low}\rangle\f$,  the verifier
 can reduce initial commitment to the result \f$\langle \vec{a},\vec{b}\rangle U\f$ to the new commitment \f$\langle
 \vec{a}_{new},\vec{b}_{new}\rangle U\f$
 *5. Analogously, if \f$\vec{G}_{new}=\vec{G}_{low}+\alpha^{-1}\vec{G}_{high}\f$, then we can reduce the initial
 *commitment to \f$\vec{a}\f$ by providing  \f$\langle\vec{a}_{low},\vec{G}_{high}\rangle\f$ and \f$\langle
 \vec{a}_{high},\vec{G}_{low}\rangle\f$. \f$\langle \vec{a}_{new},\vec{G}_{new}\rangle =
 \langle\vec{a},\vec{G}\rangle+\alpha^{-1}\langle\vec{a}_{low},\vec{G}_{high}\rangle+\alpha \langle
 \vec{a}_{high},\vec{G}_{low}\rangle\f$
 *6. We can batch the two reductions together \f$\langle \vec{a}_{new},\vec{b}_{new}\rangle U + \langle
 \vec{a}_{new},\vec{G}_{new}\rangle= \langle\vec{a},\vec{b}\rangle U+ \langle\vec{a},\vec{G}\rangle+
 \alpha^{-1}(\langle\vec{a}_{low},\vec{b}_{high}\rangle U +\langle\vec{a}_{low},\vec{G}_{high}\rangle) +\alpha (\langle
 \vec{a}_{high},\vec{b}_{low}\rangle U+\langle \vec{a}_{high},\vec{G}_{low}\rangle)\f$​
 *7. After \f$k\f$ steps of reductions we are left with \f$\langle \vec{a}_{0},\vec{b}_{0}\rangle U + \langle
 \vec{a}_{0},\vec{G}_{s}\rangle= a_0 b_0 U+a_0G_s\f$. The prover provides \f$a_0\f$. The independence of \f$U\f$ and
 \f$\vec{G}\f$ from which we construct \f$G_s\f$ ensures that \f$a_0\f$ is constructed from \f$\vec{a}_k=\vec{p}\f$
 *correctly, while the correctness of \f$a_0 b_0 U\f$ ensures that the polynomial opening is indeed \f$f(\beta)\f$
 *
 * The old version of documentation is available at <a href="https://hackmd.io/q-A8y6aITWyWJrvsGGMWNA?view">Old IPA
 documentation </a>
 */
template <typename Curve_> class IPA {
  public:
    using Curve = Curve_;
    using Fr = typename Curve::ScalarField;
    using GroupElement = typename Curve::Element;
    using Commitment = typename Curve::AffineElement;
    using CK = CommitmentKey<Curve>;
    using VK = VerifierCommitmentKey<Curve>;
    using Polynomial = bb::Polynomial<Fr>;
    using VerifierAccumulator = bool;

// These allow access to internal functions so that we can never use a mock transcript unless it's fuzzing or testing of IPA specifically
#ifdef IPA_TEST
    FRIEND_TEST(IPATest, ChallengesAreZero);
    FRIEND_TEST(IPATest, AIsZeroAfterOneRound);
#endif
#ifdef IPA_FUZZ_TEST
    friend class ProxyCaller;
#endif
    // clang-format off

    /**
     * @brief Compute an inner product argument proof for opening a single polynomial at a single evaluation point.
     *
     * @tparam Transcript Transcript type. Useful for testing
     * @param ck The commitment key containing srs and pippenger_runtime_state for computing MSM
     * @param opening_pair (challenge, evaluation)
     * @param polynomial The witness polynomial whose opening proof needs to be computed
     * @param transcript Prover transcript
     * https://github.com/AztecProtocol/aztec-packages/pull/3434
     *
     *@details For a vector \f$\vec{v}=(v_0,v_1,...,v_{2n-1})\f$ of length \f$2n\f$ we'll denote
     *\f$\vec{v}_{low}=(v_0,v_1,...,v_{n-1})\f$ and \f$\vec{v}_{high}=(v_{n},v_{n+1},...v_{2n-1})\f$. The procedure runs
     *as follows:
     *
     *1. Send the degree of \f$f(x)\f$ plus one, equal to \f$d\f$ to the verifier
     *2. Receive the generator challenge \f$u\f$ from the verifier. If it is zero, abort
     *3. Compute the auxiliary generator \f$U=u\cdot G\f$, where \f$G\f$ is a generator of \f$E(\mathbb{F}_p)\f$​
     *4. Set \f$\vec{G}_{k}=\vec{G}\f$, \f$\vec{a}_{k}=\vec{p}\f$ where \f$vec{p}\f$ represent the polynomial's
     *coefficients 
 .   *5. Compute the vector \f$\vec{b}_{k}=(1,\beta,\beta^2,...,\beta^{d-1})\f$ where \f$p(\beta)$\f is the
     evaluation we wish to prove.
     *6. Perform \f$k\f$ rounds (for \f$i \in \{k,...,1\}\f$) of:
     *   1. Compute
     \f$L_{i-1}=\langle\vec{a}_{i\_low},\vec{G}_{i\_high}\rangle+\langle\vec{a}_{i\_low},\vec{b}_{i\_high}\rangle\cdot
     U\f$​
     *   2. Compute
     *\f$R_{i-1}=\langle\vec{a}_{i\_high},\vec{G}_{i\_low}\rangle+\langle\vec{a}_{i\_high},\vec{b}_{i\_low}\rangle\cdot
     U\f$
     *   3. Send \f$L_{i-1}\f$ and \f$R_{i-1}\f$ to the verifier
     *   4. Receive round challenge \f$u_{i-1}\f$ from the verifier​, if it is zero, abort
     *   5. Compute \f$\vec{G}_{i-1}=\vec{G}_{i\_low}+u_{i-1}^{-1}\cdot \vec{G}_{i\_high}\f$
     *   6. Compute \f$\vec{a}_{i-1}=\vec{a}_{i\_low}+u_{i-1}\cdot \vec{a}_{i\_high}\f$
     *   7. Compute \f$\vec{b}_{i-1}=\vec{b}_{i\_low}+u_{i-1}^{-1}\cdot \vec{b}_{i\_high}\f$​
     *
     *7. Send the final \f$\vec{a}_{0} = (a_0)\f$ to the verifier
     */
    template <typename Transcript>
    static void compute_opening_proof_internal(const std::shared_ptr<CK>& ck,
                                               const OpeningPair<Curve>& opening_pair,
                                               const Polynomial& polynomial,
                                               const std::shared_ptr<Transcript>& transcript)
    {
        // clang-format on
        auto poly_length = static_cast<size_t>(polynomial.size());

        // Step 1.
        // Send polynomial degree + 1 = d to the verifier
        transcript->send_to_verifier("IPA:poly_degree_plus_1", static_cast<uint32_t>(poly_length));

        // Step 2.
        // Receive challenge for the auxiliary generator
        const Fr generator_challenge = transcript->template get_challenge<Fr>("IPA:generator_challenge");

        if (generator_challenge.is_zero()) {
            throw_or_abort("The generator challenge can't be zero");
        }

        // Step 3.
        // Compute auxiliary generator U
        auto aux_generator = Commitment::one() * generator_challenge;

        // Checks poly_degree is greater than zero and a power of two
        // In the future, we might want to consider if non-powers of two are needed
        ASSERT((poly_length > 0) && (!(poly_length & (poly_length - 1))) &&
               "The polynomial degree plus 1 should be positive and a power of two");

        // Step 4.
        // Set initial vector a to the polynomial monomial coefficients and load vector G
        auto a_vec = polynomial;
        auto* srs_elements = ck->srs->get_monomial_points();
        std::vector<Commitment> G_vec_local(poly_length);

        // The SRS stored in the commitment key is the result after applying the pippenger point table so the
        // values at odd indices contain the point {srs[i-1].x * beta, srs[i-1].y}, where beta is the endomorphism
        // G_vec_local should use only the original SRS thus we extract only the even indices.
        run_loop_in_parallel_if_effective(
            poly_length,
            [&G_vec_local, srs_elements](size_t start, size_t end) {
                for (size_t i = start * 2; i < end * 2; i += 2) {
                    G_vec_local[i >> 1] = srs_elements[i];
                }
            },
            /*finite_field_additions_per_iteration=*/0,
            /*finite_field_multiplications_per_iteration=*/0,
            /*finite_field_inversions_per_iteration=*/0,
            /*group_element_additions_per_iteration=*/0,
            /*group_element_doublings_per_iteration=*/0,
            /*scalar_multiplications_per_iteration=*/0,
            /*sequential_copy_ops_per_iteration=*/1);

        // Step 5.
        // Compute vector b (vector of the powers of the challenge)
        std::vector<Fr> b_vec(poly_length);
        run_loop_in_parallel_if_effective(
            poly_length,
            [&b_vec, &opening_pair](size_t start, size_t end) {
                Fr b_power = opening_pair.challenge.pow(start);
                for (size_t i = start; i < end; i++) {
                    b_vec[i] = b_power;
                    b_power *= opening_pair.challenge;
                }
            },
            /*finite_field_additions_per_iteration=*/0,
            /*finite_field_multiplications_per_iteration=*/1);

        // Iterate for log(poly_degree) rounds to compute the round commitments.
        auto log_poly_degree = static_cast<size_t>(numeric::get_msb(poly_length));

        // Allocate space for L_i and R_i elements
        GroupElement L_i;
        GroupElement R_i;
        std::size_t round_size = poly_length;

#ifndef NO_MULTITHREADING
        //  The inner products we'll be computing in parallel need a mutex to be thread-safe during the last
        //  accumulation
        std::mutex inner_product_accumulation_mutex;
#endif
        // Step 6.
        // Perform IPA reduction rounds
        for (size_t i = 0; i < log_poly_degree; i++) {
            round_size >>= 1;
            // Compute inner_prod_L := < a_vec_lo, b_vec_hi > and inner_prod_R := < a_vec_hi, b_vec_lo >
            Fr inner_prod_L = Fr::zero();
            Fr inner_prod_R = Fr::zero();
            // Run scalar products in parallel
            run_loop_in_parallel_if_effective(
                round_size,
                [&a_vec,
                 &b_vec,
                 round_size,
                 &inner_prod_L,
                 &inner_prod_R
#ifndef NO_MULTITHREADING
                 ,
                 &inner_product_accumulation_mutex
#endif
            ](size_t start, size_t end) {
                    Fr current_inner_prod_L = Fr::zero();
                    Fr current_inner_prod_R = Fr::zero();
                    for (size_t j = start; j < end; j++) {
                        current_inner_prod_L += a_vec[j] * b_vec[round_size + j];
                        current_inner_prod_R += a_vec[round_size + j] * b_vec[j];
                    }
                    // Update the accumulated results thread-safely
                    {
#ifndef NO_MULTITHREADING
                        std::unique_lock<std::mutex> lock(inner_product_accumulation_mutex);
#endif
                        inner_prod_L += current_inner_prod_L;
                        inner_prod_R += current_inner_prod_R;
                    }
                },
                /*finite_field_additions_per_iteration=*/2,
                /*finite_field_multiplications_per_iteration=*/2);

            // Step 6.a (using letters, because doxygen automaticall converts the sublist counters to letters :( )
            // L_i = < a_vec_lo, G_vec_hi > + inner_prod_L * aux_generator
            L_i = bb::scalar_multiplication::pippenger_without_endomorphism_basis_points<Curve>(
                &a_vec[0], &G_vec_local[round_size], round_size, ck->pippenger_runtime_state);
            L_i += aux_generator * inner_prod_L;

            // Step 6.b
            // R_i = < a_vec_hi, G_vec_lo > + inner_prod_R * aux_generator
            R_i = bb::scalar_multiplication::pippenger_without_endomorphism_basis_points<Curve>(
                &a_vec[round_size], &G_vec_local[0], round_size, ck->pippenger_runtime_state);
            R_i += aux_generator * inner_prod_R;

            // Step 6.c
            // Send commitments to the verifier
            std::string index = std::to_string(log_poly_degree - i - 1);
            transcript->send_to_verifier("IPA:L_" + index, Commitment(L_i));
            transcript->send_to_verifier("IPA:R_" + index, Commitment(R_i));

            // Step 6.d
            // Receive the challenge from the verifier
            const Fr round_challenge = transcript->template get_challenge<Fr>("IPA:round_challenge_" + index);

            if (round_challenge.is_zero()) {
                throw_or_abort("IPA round challenge is zero");
            }
            const Fr round_challenge_inv = round_challenge.invert();

            // Step 6.e
            // G_vec_new = G_vec_lo + G_vec_hi * round_challenge_inv
            auto G_hi_by_inverse_challenge = GroupElement::batch_mul_with_endomorphism(
                std::span{ G_vec_local.begin() + static_cast<long>(round_size),
                           G_vec_local.begin() + static_cast<long>(round_size * 2) },
                round_challenge_inv);
            GroupElement::batch_affine_add(
                std::span{ G_vec_local.begin(), G_vec_local.begin() + static_cast<long>(round_size) },
                G_hi_by_inverse_challenge,
                G_vec_local);

            // Steps 6.e and 6.f
            // Update the vectors a_vec, b_vec.
            // a_vec_new = a_vec_lo + a_vec_hi * round_challenge
            // b_vec_new = b_vec_lo + b_vec_hi * round_challenge_inv
            run_loop_in_parallel_if_effective(
                round_size,
                [&a_vec, &b_vec, round_challenge, round_challenge_inv, round_size](size_t start, size_t end) {
                    for (size_t j = start; j < end; j++) {
                        a_vec[j] += round_challenge * a_vec[round_size + j];
                        b_vec[j] += round_challenge_inv * b_vec[round_size + j];
                    }
                },
                /*finite_field_additions_per_iteration=*/4,
                /*finite_field_multiplications_per_iteration=*/8,
                /*finite_field_inversions_per_iteration=*/1);
        }

        // Step 7
        // Send a_0 to the verifier
        transcript->send_to_verifier("IPA:a_0", a_vec[0]);
    }

    /**
     * @brief Verify the correctness of a Proof
     *
     * @tparam Transcript Allows to specify a transcript class. Useful for testing
     * @param vk Verification_key containing srs and pippenger_runtime_state to be used for MSM
     * @param opening_claim Contains the commitment C and opening pair \f$(\beta, f(\beta))\f$
     * @param transcript Transcript with elements from the prover and generated challenges
     *
     * @return true/false depending on if the proof verifies
     *
     * @details The procedure runs as follows:
     *
     *1. Receive \f$d\f$ (polynomial degree plus one) from the prover
     *2. Receive the generator challenge \f$u\f$, abort if it's zero, otherwise compute \f$U=u\cdot G\f$
     *3. Compute  \f$C'=C+f(\beta)\cdot U\f$
     *4. Receive \f$L_j, R_j\f$ and compute challenges \f$u_j\f$ for \f$j \in {k-1,..,0}\f$, abort immediately on
     receiving a \f$u_j=0\f$
     *5. Compute \f$C_0 = C' + \sum_{j=0}^{k-1}(u_j^{-1}L_j + u_jR_j)\f$
     *6. Compute \f$b_0=g(\beta)=\prod_{i=0}^{k-1}(1+u_{i}^{-1}x^{2^{i}})\f$
     *7. Compute vector \f$\vec{s}=(1,u_{0}^{-1},u_{1}^{-1},u_{0}^{-1}u_{1}^{-1},...,\prod_{i=0}^{k-1}u_{i}^{-1})\f$
     *8. Compute \f$G_s=\langle \vec{s},\vec{G}\rangle\f$
     *9. Receive \f$\vec{a}_{0}\f$ of length 1
     *10. Compute \f$C_{right}=a_{0}G_{s}+a_{0}b_{0}U\f$
     *11. Check that \f$C_{right} = C_0\f$. If they match, return true. Otherwise return false.
     */
    template <typename Transcript>
    static VerifierAccumulator reduce_verify_internal(const std::shared_ptr<VK>& vk,
                                                      const OpeningClaim<Curve>& opening_claim,
                                                      const std::shared_ptr<Transcript>& transcript)
    {
        // Step 1.
        // Receive polynomial_degree + 1 = d from the prover
        auto poly_length = static_cast<uint32_t>(transcript->template receive_from_prover<typename Curve::BaseField>(
            "IPA:poly_degree_plus_1")); // note this is base field because this is a uint32_t, which should map
                                        // to a bb::fr, not a grumpkin::fr, which is a BaseField element for
                                        // Grumpkin
        // Step 2.
        // Receive generator challenge u and compute auxiliary generator
        const Fr generator_challenge = transcript->template get_challenge<Fr>("IPA:generator_challenge");

        if (generator_challenge.is_zero()) {
            throw_or_abort("The generator challenge can't be zero");
        }
        auto aux_generator = Commitment::one() * generator_challenge;

        auto log_poly_degree = static_cast<size_t>(numeric::get_msb(poly_length));
        // Step 3.
        // Compute C' = C + f(\beta) ⋅ U
        GroupElement C_prime = opening_claim.commitment + (aux_generator * opening_claim.opening_pair.evaluation);

        auto pippenger_size = 2 * log_poly_degree;
        std::vector<Fr> round_challenges(log_poly_degree);
        std::vector<Fr> round_challenges_inv(log_poly_degree);
        std::vector<Commitment> msm_elements(pippenger_size);
        std::vector<Fr> msm_scalars(pippenger_size);

        // Step 4.
        // Receive all L_i and R_i and prepare for MSM
        for (size_t i = 0; i < log_poly_degree; i++) {
            std::string index = std::to_string(log_poly_degree - i - 1);
            auto element_L = transcript->template receive_from_prover<Commitment>("IPA:L_" + index);
            auto element_R = transcript->template receive_from_prover<Commitment>("IPA:R_" + index);
            round_challenges[i] = transcript->template get_challenge<Fr>("IPA:round_challenge_" + index);
            if (round_challenges[i].is_zero()) {
                throw_or_abort("Round challenges can't be zero");
            }
            round_challenges_inv[i] = round_challenges[i].invert();

            msm_elements[2 * i] = element_L;
            msm_elements[2 * i + 1] = element_R;
            msm_scalars[2 * i] = round_challenges_inv[i];
            msm_scalars[2 * i + 1] = round_challenges[i];
        }

        // Step 5.
        // Compute C₀ = C' + ∑_{j ∈ [k]} u_j^{-1}L_j + ∑_{j ∈ [k]} u_jR_j
        GroupElement LR_sums = bb::scalar_multiplication::pippenger_without_endomorphism_basis_points<Curve>(
            &msm_scalars[0], &msm_elements[0], pippenger_size, vk->pippenger_runtime_state);
        GroupElement C_zero = C_prime + LR_sums;

        //  Step 6.
        // Compute b_zero where b_zero can be computed using the polynomial:
        //  g(X) = ∏_{i ∈ [k]} (1 + u_{i-1}^{-1}.X^{2^{i-1}}).
        //  b_zero = g(evaluation) = ∏_{i ∈ [k]} (1 + u_{i-1}^{-1}. (evaluation)^{2^{i-1}})
        Fr b_zero = Fr::one();
        for (size_t i = 0; i < log_poly_degree; i++) {
            auto exponent = static_cast<uint64_t>(Fr(2).pow(i));
            b_zero *= Fr::one() + (round_challenges_inv[log_poly_degree - 1 - i] *
                                   opening_claim.opening_pair.challenge.pow(exponent));
        }

        // Step 7.
        // Construct vector s
        std::vector<Fr> s_vec(poly_length);

        // TODO(https://github.com/AztecProtocol/barretenberg/issues/857): This code is not efficient as its
        // O(nlogn). This can be optimized to be linear by computing a tree of products. Its very readable, so we're
        // leaving it unoptimized for now.
        run_loop_in_parallel_if_effective(
            poly_length,
            [&s_vec, &round_challenges_inv, log_poly_degree](size_t start, size_t end) {
                for (size_t i = start; i < end; i++) {
                    Fr s_vec_scalar = Fr::one();
                    for (size_t j = (log_poly_degree - 1); j != size_t(-1); j--) {
                        auto bit = (i >> j) & 1;
                        bool b = static_cast<bool>(bit);
                        if (b) {
                            s_vec_scalar *= round_challenges_inv[log_poly_degree - 1 - j];
                        }
                    }
                    s_vec[i] = s_vec_scalar;
                }
            },
            /*finite_field_additions_per_iteration=*/0,
            /*finite_field_multiplications_per_iteration=*/log_poly_degree);

        auto* srs_elements = vk->srs->get_monomial_points();

        // Copy the G_vector to local memory.
        std::vector<Commitment> G_vec_local(poly_length);

        // The SRS stored in the commitment key is the result after applying the pippenger point table so the
        // values at odd indices contain the point {srs[i-1].x * beta, srs[i-1].y}, where beta is the endomorphism
        // G_vec_local should use only the original SRS thus we extract only the even indices.
        run_loop_in_parallel_if_effective(
            poly_length,
            [&G_vec_local, srs_elements](size_t start, size_t end) {
                for (size_t i = start * 2; i < end * 2; i += 2) {
                    G_vec_local[i >> 1] = srs_elements[i];
                }
            },
            /*finite_field_additions_per_iteration=*/0,
            /*finite_field_multiplications_per_iteration=*/0,
            /*finite_field_inversions_per_iteration=*/0,
            /*group_element_additions_per_iteration=*/0,
            /*group_element_doublings_per_iteration=*/0,
            /*scalar_multiplications_per_iteration=*/0,
            /*sequential_copy_ops_per_iteration=*/1);

        // Step 8.
        // Compute G₀
        auto G_zero = bb::scalar_multiplication::pippenger_without_endomorphism_basis_points<Curve>(
            &s_vec[0], &G_vec_local[0], poly_length, vk->pippenger_runtime_state);

        // Step 9.
        // Receive a₀ from the prover
        auto a_zero = transcript->template receive_from_prover<Fr>("IPA:a_0");

        // Step 10.
        // Compute C_right
        GroupElement right_hand_side = G_zero * a_zero + aux_generator * a_zero * b_zero;

        // Step 11.
        // Check if C_right == C₀
        return (C_zero.normalize() == right_hand_side.normalize());
    }

  public:
    /**
     * @brief Compute an inner product argument proof for opening a single polynomial at a single evaluation point.
     *
     * @param ck The commitment key containing srs and pippenger_runtime_state for computing MSM
     * @param opening_pair (challenge, evaluation)
     * @param polynomial The witness polynomial whose opening proof needs to be computed
     * @param transcript Prover transcript
     *
     * @remark Detailed documentation can be found in \link IPA::compute_opening_proof_internal
     * compute_opening_proof_internal \endlink.
     */
    static void compute_opening_proof(const std::shared_ptr<CK>& ck,
                                      const OpeningPair<Curve>& opening_pair,
                                      const Polynomial& polynomial,
                                      const std::shared_ptr<NativeTranscript>& transcript)
    {
        compute_opening_proof_internal(ck, opening_pair, polynomial, transcript);
    }

    /**
     * @brief Verify the correctness of a Proof
     *
     * @param vk Verification_key containing srs and pippenger_runtime_state to be used for MSM
     * @param opening_claim Contains the commitment C and opening pair \f$(\beta, f(\beta))\f$
     * @param transcript Transcript with elements from the prover and generated challenges
     *
     * @return true/false depending on if the proof verifies
     *
     *@remark The verification procedure documentation is in \link IPA::verify_internal verify_internal \endlink
     */
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/912): Return the proper VerifierAccumulator once
    // implemented
    static VerifierAccumulator reduce_verify(const std::shared_ptr<VK>& vk,
                                             const OpeningClaim<Curve>& opening_claim,
                                             const std::shared_ptr<NativeTranscript>& transcript)
    {
        return reduce_verify_internal(vk, opening_claim, transcript);
    }
};

} // namespace bb