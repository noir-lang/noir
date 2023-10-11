#pragma once
#include "barretenberg/polynomials/polynomial.hpp"

namespace proof_system::honk::pcs::zeromorph {

/**
 * @brief Compute powers of a given challenge
 *
 * @tparam FF
 * @param challenge
 * @param num_powers
 * @return std::vector<FF>
 */
template <class FF> inline std::vector<FF> powers_of_challenge(const FF challenge, const size_t num_powers)
{
    std::vector<FF> challenge_powers = { FF(1), challenge };
    challenge_powers.reserve(num_powers);
    for (size_t j = 2; j < num_powers; j++) {
        challenge_powers.emplace_back(challenge_powers[j - 1] * challenge);
    }
    return challenge_powers;
};

/**
 * @brief Prover for ZeroMorph multilinear PCS
 *
 * @tparam Curve
 */
template <typename Curve> class ZeroMorphProver_ {
    using FF = typename Curve::ScalarField;
    using Commitment = typename Curve::AffineElement;
    using Polynomial = barretenberg::Polynomial<FF>;

    // TODO(#742): Set this N_max to be the number of G1 elements in the mocked zeromorph SRS once it's in place. (Then,
    // eventually, set it based on the real SRS). For now we set it to be large but more or less arbitrary.
    static const size_t N_max = 1 << 22;

  public:
    /**
     * @brief Compute multivariate quotients q_k(X_0, ..., X_{k-1}) for f(X_0, ..., X_{d-1})
     * @details Given multilinear polynomial f = f(X_0, ..., X_{d-1}) for which f(u) = v, compute q_k such that:
     *
     *  f(X_0, ..., X_{d-1}) - v = \sum_{k=0}^{d-1} (X_k - u_k)q_k(X_0, ..., X_{k-1})
     *
     * The polynomials q_k can be computed explicitly as the difference of the partial evaluation of f in the last
     * (n - k) variables at, respectively, u'' = (u_k + 1, u_{k+1}, ..., u_{n-1}) and u' = (u_k, ..., u_{n-1}). I.e.
     *
     *  q_k(X_0, ..., X_{k-1}) = f(X_0,...,X_{k-1}, u'') - f(X_0,...,X_{k-1}, u')
     *
     * @note In practice, 2^d is equal to the circuit size N
     *
     * TODO(#739): This method has been designed for clarity at the expense of efficiency. Implement the more efficient
     * algorithm detailed in the latest versions of the ZeroMorph paper.
     * @param polynomial Multilinear polynomial f(X_0, ..., X_{d-1})
     * @param u_challenge Multivariate challenge u = (u_0, ..., u_{d-1})
     * @return std::vector<Polynomial> The quotients q_k
     */
    static std::vector<Polynomial> compute_multilinear_quotients(Polynomial polynomial, std::span<FF> u_challenge)
    {
        size_t log_N = numeric::get_msb(polynomial.size());
        // The size of the multilinear challenge must equal the log of the polynomial size
        ASSERT(log_N == u_challenge.size());

        // Define the vector of quotients q_k, k = 0, ..., log_N-1
        std::vector<Polynomial> quotients;
        for (size_t k = 0; k < log_N; ++k) {
            size_t size = 1 << k;
            quotients.emplace_back(Polynomial(size)); // degree 2^k - 1
        }

        // Compute the q_k in reverse order, i.e. q_{n-1}, ..., q_0
        for (size_t k = 0; k < log_N; ++k) {
            // Define partial evaluation point u' = (u_k, ..., u_{n-1})
            auto evaluation_point_size = static_cast<std::ptrdiff_t>(k + 1);
            std::vector<FF> u_partial(u_challenge.end() - evaluation_point_size, u_challenge.end());

            // Compute f' = f(X_0,...,X_{k-1}, u')
            auto f_1 = polynomial.partial_evaluate_mle(u_partial);

            // Increment first element to get altered partial evaluation point u'' = (u_k + 1, u_{k+1}, ..., u_{n-1})
            u_partial[0] += 1;

            // Compute f'' = f(X_0,...,X_{k-1}, u'')
            auto f_2 = polynomial.partial_evaluate_mle(u_partial);

            // Compute q_k = f''(X_0,...,X_{k-1}) - f'(X_0,...,X_{k-1})
            auto q_k = f_2;
            q_k -= f_1;

            quotients[log_N - k - 1] = q_k;
        }

        return quotients;
    }

    /**
     * @brief Construct batched, lifted-degree univariate quotient \hat{q} = \sum_k y^k * X^{N - d_k - 1} * q_k
     * @details The purpose of the batched lifted-degree quotient is to reduce the individual degree checks
     * deg(q_k) <= 2^k - 1 to a single degree check on \hat{q}. This is done by first shifting each of the q_k to the
     * right (i.e. multiplying by an appropriate power of X) so that each is degree N-1, then batching them all together
     * using powers of the provided challenge. Note: In practice, we do not actually compute the shifted q_k, we simply
     * accumulate them into \hat{q} at the appropriate offset.
     *
     * @param quotients Polynomials q_k, interpreted as univariates; deg(q_k) = 2^k - 1
     * @param N circuit size
     * @return Polynomial
     */
    static Polynomial compute_batched_lifted_degree_quotient(std::vector<Polynomial>& quotients,
                                                             FF y_challenge,
                                                             size_t N)
    {
        // Batched lifted degree quotient polynomial
        auto result = Polynomial(N);

        // Compute \hat{q} = \sum_k y^k * X^{N - d_k - 1} * q_k
        size_t k = 0;
        auto scalar = FF(1); // y^k
        for (auto& quotient : quotients) {
            // Rather than explicitly computing the shifts of q_k by N - d_k - 1 (i.e. multiplying q_k by X^{N - d_k -
            // 1}) then accumulating them, we simply accumulate y^k*q_k into \hat{q} at the index offset N - d_k - 1
            auto deg_k = static_cast<size_t>((1 << k) - 1);
            size_t offset = N - deg_k - 1;
            for (size_t idx = 0; idx < deg_k + 1; ++idx) {
                result[offset + idx] += scalar * quotient[idx];
            }
            scalar *= y_challenge; // update batching scalar y^k
            k++;
        }

        return result;
    }

    /**
     * @brief Compute partially evaluated degree check polynomial \zeta_x = q - \sum_k y^k * x^{N - d_k - 1} * q_k
     * @details Compute \zeta_x, where
     *
     *                          \zeta_x = q - \sum_k y^k * x^{N - d_k - 1} * q_k
     *
     * @param batched_quotient
     * @param quotients
     * @param y_challenge
     * @param x_challenge
     * @return Polynomial Degree check polynomial \zeta_x such that \zeta_x(x) = 0
     */
    static Polynomial compute_partially_evaluated_degree_check_polynomial(Polynomial& batched_quotient,
                                                                          std::vector<Polynomial>& quotients,
                                                                          FF y_challenge,
                                                                          FF x_challenge)
    {
        size_t N = batched_quotient.size();
        size_t log_N = quotients.size();

        // Initialize partially evaluated degree check polynomial \zeta_x to \hat{q}
        auto result = batched_quotient;

        auto y_power = FF(1); // y^k
        for (size_t k = 0; k < log_N; ++k) {
            // Accumulate y^k * x^{N - d_k - 1} * q_k into \hat{q}
            auto deg_k = static_cast<size_t>((1 << k) - 1);
            auto x_power = x_challenge.pow(N - deg_k - 1); // x^{N - d_k - 1}

            result.add_scaled(quotients[k], -y_power * x_power);

            y_power *= y_challenge; // update batching scalar y^k
        }

        return result;
    }

    /**
     * @brief Compute partially evaluated zeromorph identity polynomial Z_x
     * @details Compute Z_x, where
     *
     *  Z_x = x * f_batched + g_batched - v * x * \Phi_n(x)
     *           - x * \sum_k (x^{2^k}\Phi_{n-k-1}(x^{2^{k-1}}) - u_k\Phi_{n-k}(x^{2^k})) * q_k
     *
     * where f_batched = \sum_{i=0}^{m-1}\rho^i*f_i, g_batched = \sum_{i=0}^{l-1}\rho^{m+i}*g_i
     *
     * @param input_polynomial
     * @param quotients
     * @param v_evaluation
     * @param x_challenge
     * @return Polynomial
     */
    static Polynomial compute_partially_evaluated_zeromorph_identity_polynomial(Polynomial& f_batched,
                                                                                Polynomial& g_batched,
                                                                                std::vector<Polynomial>& quotients,
                                                                                FF v_evaluation,
                                                                                std::span<FF> u_challenge,
                                                                                FF x_challenge)
    {
        size_t N = f_batched.size();
        size_t log_N = quotients.size();

        // Initialize Z_x with x * \sum_{i=0}^{m-1} f_i + \sum_{i=0}^{l-1} g_i
        auto result = g_batched;
        result.add_scaled(f_batched, x_challenge);

        // Compute Z_x -= v * x * \Phi_n(x)
        auto phi_numerator = x_challenge.pow(N) - 1; // x^N - 1
        auto phi_n_x = phi_numerator / (x_challenge - 1);
        result[0] -= v_evaluation * x_challenge * phi_n_x;

        // Add contribution from q_k polynomials
        auto x_power = x_challenge; // x^{2^k}
        for (size_t k = 0; k < log_N; ++k) {
            x_power = x_challenge.pow(1 << k); // x^{2^k}

            // \Phi_{n-k-1}(x^{2^{k + 1}})
            auto phi_term_1 = phi_numerator / (x_challenge.pow(1 << (k + 1)) - 1);

            // \Phi_{n-k}(x^{2^k})
            auto phi_term_2 = phi_numerator / (x_challenge.pow(1 << k) - 1);

            // x^{2^k} * \Phi_{n-k-1}(x^{2^{k+1}}) - u_k *  \Phi_{n-k}(x^{2^k})
            auto scalar = x_power * phi_term_1 - u_challenge[k] * phi_term_2;

            scalar *= x_challenge;
            scalar *= FF(-1);

            result.add_scaled(quotients[k], scalar);
        }

        return result;
    }

    /**
     * @brief Compute combined evaluation and degree-check quotient polynomial pi
     * @details Compute univariate quotient pi, where
     *
     *  pi = (q_\zeta + z*q_Z) X^{N_{max}-(N-1)}, with q_\zeta = \zeta_x/(X-x), q_Z = Z_x/(X-x)
     *
     * @param Z_x
     * @param zeta_x
     * @param x_challenge
     * @param z_challenge
     * @param N_max
     * @return Polynomial
     */
    static Polynomial compute_batched_evaluation_and_degree_check_quotient(Polynomial& zeta_x,
                                                                           Polynomial& Z_x,
                                                                           FF x_challenge,
                                                                           FF z_challenge)
    {
        // We cannot commit to polynomials with size > N_max
        size_t N = zeta_x.size();
        ASSERT(N <= N_max);

        // Compute q_{\zeta} and q_Z in place
        zeta_x.factor_roots(x_challenge);
        Z_x.factor_roots(x_challenge);

        // Compute batched quotient q_{\zeta} + z*q_Z
        auto batched_quotient = zeta_x;
        batched_quotient.add_scaled(Z_x, z_challenge);

        // TODO(#742): To complete the degree check, we need to commit to (q_{\zeta} + z*q_Z)*X^{N_max - N - 1}.
        // Verification then requires a pairing check similar to the standard KZG check but with [1]_2 replaced by
        // [X^{N_max - N -1}]_2. Two issues: A) we do not have an SRS with these G2 elements (so need to generate a fake
        // setup until we can do the real thing), and B) its not clear to me how to update our pairing algorithms to do
        // this type of pairing. For now, simply construct q_{\zeta} + z*q_Z without the shift and do a standard KZG
        // pairing check. When we're ready, all we have to do to make this fully legit is commit to the shift here and
        // update the pairing check accordingly. Note: When this is implemented properly, it doesnt make sense to store
        // the (massive) shifted polynomial of size N_max. Ideally would only store the unshifted version and just
        // compute the shifted commitment directly via a new method.
        auto batched_shifted_quotient = batched_quotient;

        return batched_shifted_quotient;
    }

    /**
     * @brief Prove a set of multilinear evaluation claims for unshifted polynomials f_i and to-be-shifted polynomials
     * g_i
     *
     * @param f_polynomials Unshifted polynomials
     * @param g_polynomials To-be-shifted polynomials (of which the shifts h_i were evaluated by sumcheck)
     * @param evaluations Set of evaluations v_i = f_i(u), w_i = h_i(u) = g_i_shifted(u)
     * @param multilinear_challenge Multilinear challenge point u
     * @param commitment_key
     * @param transcript
     */
    static void prove(const auto& f_polynomials,
                      const auto& g_polynomials,
                      auto& evaluations,
                      auto& multilinear_challenge,
                      auto& commitment_key,
                      auto& transcript)
    {
        // Generate batching challenge \rho and powers 1,...,\rho^{m-1}
        FF rho = transcript.get_challenge("rho");
        std::vector<FF> rhos = powers_of_challenge(rho, evaluations.size());

        // Extract multilinear challenge u and claimed multilinear evaluations from Sumcheck output
        std::span<FF> u_challenge = multilinear_challenge;
        std::span<FF> claimed_evaluations = evaluations;
        size_t log_N = u_challenge.size();
        size_t N = 1 << log_N;

        // Compute batching of unshifted polynomials f_i and to-be-shifted polynomials g_i:
        // f_batched = sum_{i=0}^{m-1}\rho^i*f_i and g_batched = sum_{i=0}^{l-1}\rho^{m+i}*g_i,
        // and also batched evaluation
        // v = sum_{i=0}^{m-1}\rho^i*f_i(u) + sum_{i=0}^{l-1}\rho^{m+i}*h_i(u).
        // Note: g_batched is formed from the to-be-shifted polynomials, but the batched evaluation incorporates the
        // evaluations produced by sumcheck of h_i = g_i_shifted.
        auto batched_evaluation = FF(0);
        Polynomial f_batched(N); // batched unshifted polynomials
        size_t poly_idx = 0;     // TODO(#391) zip
        for (auto& f_poly : f_polynomials) {
            f_batched.add_scaled(f_poly, rhos[poly_idx]);
            batched_evaluation += rhos[poly_idx] * claimed_evaluations[poly_idx];
            ++poly_idx;
        }

        Polynomial g_batched(N); // batched to-be-shifted polynomials
        for (auto& g_poly : g_polynomials) {
            g_batched.add_scaled(g_poly, rhos[poly_idx]);
            batched_evaluation += rhos[poly_idx] * claimed_evaluations[poly_idx];
            ++poly_idx;
        };

        // Compute the full batched polynomial f = f_batched + g_batched.shifted() = f_batched + h_batched. This is the
        // polynomial for which we compute the quotients q_k and prove f(u) = v_batched.
        auto f_polynomial = f_batched;
        f_polynomial += g_batched.shifted();

        // Compute the multilinear quotients q_k = q_k(X_0, ..., X_{k-1})
        auto quotients = compute_multilinear_quotients(f_polynomial, u_challenge);

        // Compute and send commitments C_{q_k} = [q_k], k = 0,...,d-1
        std::vector<Commitment> q_k_commitments;
        q_k_commitments.reserve(log_N);
        for (size_t idx = 0; idx < log_N; ++idx) {
            q_k_commitments[idx] = commitment_key->commit(quotients[idx]);
            std::string label = "ZM:C_q_" + std::to_string(idx);
            transcript.send_to_verifier(label, q_k_commitments[idx]);
        }

        // Get challenge y
        auto y_challenge = transcript.get_challenge("ZM:y");

        // Compute the batched, lifted-degree quotient \hat{q}
        auto batched_quotient = compute_batched_lifted_degree_quotient(quotients, y_challenge, N);

        // Compute and send the commitment C_q = [\hat{q}]
        auto q_commitment = commitment_key->commit(batched_quotient);
        transcript.send_to_verifier("ZM:C_q", q_commitment);

        // Get challenges x and z
        auto [x_challenge, z_challenge] = transcript.get_challenges("ZM:x", "ZM:z");

        // Compute degree check polynomial \zeta partially evaluated at x
        auto zeta_x =
            compute_partially_evaluated_degree_check_polynomial(batched_quotient, quotients, y_challenge, x_challenge);

        // Compute ZeroMorph identity polynomial Z partially evaluated at x
        auto Z_x = compute_partially_evaluated_zeromorph_identity_polynomial(
            f_batched, g_batched, quotients, batched_evaluation, u_challenge, x_challenge);

        // Compute batched degree-check and ZM-identity quotient polynomial pi
        auto pi_polynomial =
            compute_batched_evaluation_and_degree_check_quotient(zeta_x, Z_x, x_challenge, z_challenge);

        // Compute and send proof commitment pi
        auto pi_commitment = commitment_key->commit(pi_polynomial);
        transcript.send_to_verifier("ZM:PI", pi_commitment);
    }
};

/**
 * @brief Verifier for ZeroMorph multilinear PCS
 *
 * @tparam Curve
 */
template <typename Curve> class ZeroMorphVerifier_ {
    using FF = typename Curve::ScalarField;
    using Commitment = typename Curve::AffineElement;

  public:
    /**
     * @brief Compute commitment to partially evaluated batched lifted degree quotient identity
     * @details Compute commitment C_{\zeta_x} = [\zeta_x]_1 using homomorphicity:
     *
     *  C_{\zeta_x} = [q]_1 - \sum_k y^k * x^{N - d_k - 1} * [q_k]_1
     *
     * @param C_q Commitment to batched lifted degree quotient
     * @param C_q_k Commitments to quotients q_k
     * @param y_challenge
     * @param x_challenge
     * @return Commitment
     */
    static Commitment compute_C_zeta_x(Commitment C_q, std::vector<Commitment>& C_q_k, FF y_challenge, FF x_challenge)
    {
        size_t log_N = C_q_k.size();
        size_t N = 1 << log_N;

        // Instantiate containers for input to batch mul
        std::vector<FF> scalars;
        std::vector<Commitment> commitments;

        // Contribution from C_q
        if constexpr (Curve::is_stdlib_type) {
            auto builder = x_challenge.get_context();
            scalars.emplace_back(FF(builder, 1));
        } else {
            scalars.emplace_back(FF(1));
        }
        commitments.emplace_back(C_q);

        // Contribution from C_q_k, k = 0,...,log_N
        for (size_t k = 0; k < log_N; ++k) {
            auto deg_k = static_cast<size_t>((1 << k) - 1);
            // Compute scalar y^k * x^{N - deg_k - 1}
            auto scalar = y_challenge.pow(k);
            scalar *= x_challenge.pow(N - deg_k - 1);
            scalar *= FF(-1);

            scalars.emplace_back(scalar);
            commitments.emplace_back(C_q_k[k]);
        }

        // Compute batch mul to get the result
        if constexpr (Curve::is_stdlib_type) {
            return Commitment::batch_mul(commitments, scalars);
        } else {
            return batch_mul_native(commitments, scalars);
        }
    }

    /**
     * @brief Compute commitment to partially evaluated ZeroMorph identity Z
     * @details Compute commitment C_{Z_x} = [Z_x]_1 using homomorphicity:
     *
     *  C_{Z_x} = x * \sum_{i=0}^{m-1}\rho^i*[f_i] + \sum_{i=0}^{l-1}\rho^{m+i}*[g_i] - v * x * \Phi_n(x) * [1]_1
     *              - x * \sum_k (x^{2^k}\Phi_{n-k-1}(x^{2^{k-1}}) - u_k\Phi_{n-k}(x^{2^k})) * [q_k]
     *
     * @param f_commitments Commitments to unshifted polynomials [f_i]
     * @param g_commitments Commitments to to-be-shifted polynomials [g_i]
     * @param C_q_k Commitments to q_k
     * @param rho
     * @param batched_evaluation \sum_{i=0}^{m-1} \rho^i*f_i(u) + \sum_{i=0}^{l-1} \rho^{m+i}*h_i(u)
     * @param x_challenge
     * @param u_challenge multilinear challenge
     * @return Commitment
     */
    static Commitment compute_C_Z_x(std::vector<Commitment> f_commitments,
                                    std::vector<Commitment> g_commitments,
                                    std::vector<Commitment>& C_q_k,
                                    FF rho,
                                    FF batched_evaluation,
                                    FF x_challenge,
                                    std::vector<FF> u_challenge)
    {
        size_t log_N = C_q_k.size();
        size_t N = 1 << log_N;

        std::vector<FF> scalars;
        std::vector<Commitment> commitments;

        // Phi_n(x) = (x^N - 1) / (x - 1)
        auto phi_numerator = x_challenge.pow(N) - 1; // x^N - 1
        auto phi_n_x = phi_numerator / (x_challenge - 1);

        // Add contribution: -v * x * \Phi_n(x) * [1]_1
        if constexpr (Curve::is_stdlib_type) {
            auto builder = x_challenge.get_context();
            scalars.emplace_back(FF(builder, -1) * batched_evaluation * x_challenge * phi_n_x);
            commitments.emplace_back(Commitment::one(builder));
        } else {
            scalars.emplace_back(FF(-1) * batched_evaluation * x_challenge * phi_n_x);
            commitments.emplace_back(Commitment::one());
        }

        // Add contribution: x * \sum_{i=0}^{m-1} \rho^i*[f_i]
        auto rho_pow = FF(1);
        for (auto& commitment : f_commitments) {
            scalars.emplace_back(x_challenge * rho_pow);
            commitments.emplace_back(commitment);
            rho_pow *= rho;
        }

        // Add contribution: \sum_{i=0}^{l-1} \rho^{m+i}*[g_i]
        for (auto& commitment : g_commitments) {
            scalars.emplace_back(rho_pow);
            commitments.emplace_back(commitment);
            rho_pow *= rho;
        }

        // Add contributions: scalar * [q_k],  k = 0,...,log_N, where
        // scalar = -x * (x^{2^k} * \Phi_{n-k-1}(x^{2^{k+1}}) - u_k * \Phi_{n-k}(x^{2^k}))
        auto x_pow_2k = x_challenge;                 // x^{2^k}
        auto x_pow_2kp1 = x_challenge * x_challenge; // x^{2^{k + 1}}
        for (size_t k = 0; k < log_N; ++k) {

            auto phi_term_1 = phi_numerator / (x_pow_2kp1 - 1); // \Phi_{n-k-1}(x^{2^{k + 1}})
            auto phi_term_2 = phi_numerator / (x_pow_2k - 1);   // \Phi_{n-k}(x^{2^k})

            auto scalar = x_pow_2k * phi_term_1;
            scalar -= u_challenge[k] * phi_term_2;
            scalar *= x_challenge;
            scalar *= FF(-1);

            scalars.emplace_back(scalar);
            commitments.emplace_back(C_q_k[k]);

            // Update powers of challenge x
            x_pow_2k = x_pow_2kp1;
            x_pow_2kp1 *= x_pow_2kp1;
        }

        if constexpr (Curve::is_stdlib_type) {
            return Commitment::batch_mul(commitments, scalars);
        } else {
            return batch_mul_native(commitments, scalars);
        }
    }

    /**
     * @brief Utility for native batch multiplication of group elements
     * @note This is used only for native verification and is not optimized for efficiency
     */
    static Commitment batch_mul_native(std::vector<Commitment> points, std::vector<FF> scalars)
    {
        auto result = points[0] * scalars[0];
        for (size_t idx = 1; idx < scalars.size(); ++idx) {
            result = result + points[idx] * scalars[idx];
        }
        return result;
    }

    /**
     * @brief Verify a set of multilinear evaluation claims for unshifted polynomials f_i and to-be-shifted polynomials
     * g_i
     *
     * @param commitments Commitments to polynomials f_i and g_i (unshifted and to-be-shifted)
     * @param claimed_evaluations Claimed evaluations v_i = f_i(u) and w_i = h_i(u) = g_i_shifted(u)
     * @param multivariate_challenge Challenge point u
     * @param transcript
     * @return std::array<Commitment, 2> Inputs to the final pairing check
     */
    static std::array<Commitment, 2> verify(auto& commitments,
                                            auto& claimed_evaluations,
                                            auto& multivariate_challenge,
                                            auto& transcript)
    {
        size_t log_N = multivariate_challenge.size();
        FF rho = transcript.get_challenge("rho");

        // Compute powers of batching challenge rho
        std::vector<FF> rhos = pcs::zeromorph::powers_of_challenge(rho, claimed_evaluations.size());

        // Construct batched evaluation v = sum_{i=0}^{m-1}\rho^i*f_i(u) + sum_{i=0}^{l-1}\rho^{m+i}*h_i(u)
        FF batched_evaluation = FF(0);
        size_t evaluation_idx = 0;
        for (auto& value : claimed_evaluations.get_unshifted_then_shifted()) {
            batched_evaluation += value * rhos[evaluation_idx];
            ++evaluation_idx;
        }

        // Receive commitments [q_k]
        std::vector<Commitment> C_q_k;
        C_q_k.reserve(log_N);
        for (size_t i = 0; i < log_N; ++i) {
            C_q_k.emplace_back(transcript.template receive_from_prover<Commitment>("ZM:C_q_" + std::to_string(i)));
        }

        // Challenge y
        auto y_challenge = transcript.get_challenge("ZM:y");

        // Receive commitment C_{q}
        auto C_q = transcript.template receive_from_prover<Commitment>("ZM:C_q");

        // Challenges x, z
        auto [x_challenge, z_challenge] = transcript.get_challenges("ZM:x", "ZM:z");

        // Compute commitment C_{\zeta_x}
        auto C_zeta_x = compute_C_zeta_x(C_q, C_q_k, y_challenge, x_challenge);

        // Compute commitment C_{Z_x}
        Commitment C_Z_x = compute_C_Z_x(commitments.get_unshifted(),
                                         commitments.get_to_be_shifted(),
                                         C_q_k,
                                         rho,
                                         batched_evaluation,
                                         x_challenge,
                                         multivariate_challenge);

        // Compute commitment C_{\zeta,Z}
        auto C_zeta_Z = C_zeta_x + C_Z_x * z_challenge;

        // Receive proof commitment \pi
        auto C_pi = transcript.template receive_from_prover<Commitment>("ZM:PI");

        // Construct inputs and perform pairing check to verify claimed evaluation
        // Note: The pairing check (without the degree check component X^{N_max-N-1}) can be expressed naturally as
        // e(C_{\zeta,Z}, [1]_2) = e(pi, [X - x]_2). This can be rearranged (e.g. see the plonk paper) as
        // e(C_{\zeta,Z} - x*pi, [1]_2) * e(-pi, [X]_2) = 1, or
        // e(P_0, [1]_2) * e(P_1, [X]_2) = 1
        auto P0 = C_zeta_Z + C_pi * x_challenge;
        auto P1 = -C_pi;

        return { P0, P1 };
    }
};

} // namespace proof_system::honk::pcs::zeromorph