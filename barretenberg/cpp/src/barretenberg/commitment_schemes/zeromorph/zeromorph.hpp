#pragma once
#include "barretenberg/commitment_schemes/claim.hpp"
#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/commitment_schemes/verification_key.hpp"
#include "barretenberg/common/ref_span.hpp"
#include "barretenberg/common/ref_vector.hpp"
#include "barretenberg/common/zip_view.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/stdlib/primitives/biggroup/biggroup.hpp"
#include "barretenberg/stdlib/primitives/witness/witness.hpp"
#include "barretenberg/transcript/transcript.hpp"

namespace bb {

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
 * @tparam Curve - The curve used for arithmetising ZeroMorph
 */
template <typename Curve> class ZeroMorphProver_ {
    using FF = typename Curve::ScalarField;
    using Commitment = typename Curve::AffineElement;
    using Polynomial = bb::Polynomial<FF>;
    using OpeningClaim = ProverOpeningClaim<Curve>;

    // TODO(#742): Set this N_max to be the number of G1 elements in the mocked zeromorph SRS once it's in place.
    // (Then, eventually, set it based on the real SRS). For now we set it to be larger then the Client IVC recursive
    // verifier circuit.
    static const size_t N_max = 1 << 25;

  public:
    /**
     * @brief Compute multivariate quotients q_k(X_0, ..., X_{k-1}) for f(X_0, ..., X_{n-1})
     * @details Starting from the coefficients of f, compute q_k inductively from k = n - 1, to k = 0.
     *          f needs to be updated at each step.
     *
     *          First, compute q_{n-1} of size N/2 by
     *          q_{n-1}[l] = f[N/2 + l ] - f[l].
     *
     *          Update f by f[l] <- f[l] + u_{n-1} * q_{n-1}[l]; f now has size N/2.
     *          Compute q_{n-2} of size N/(2^2) by
     *          q_{n-2}[l] = f[N/2^2 + l] - f[l].
     *
     *          Update f by f[l] <- f[l] + u_{n-2} * q_{n-2}[l]; f now has size N/(2^2).
     *          Compute q_{n-3} of size N/(2^3) by
     *          q_{n-3}[l] = f[N/2^3 + l] - f[l]. Repeat similarly until you reach q_0.
     *
     * @param polynomial Multilinear polynomial f(X_0, ..., X_{d-1})
     * @param u_challenge Multivariate challenge u = (u_0, ..., u_{d-1})
     * @return std::vector<Polynomial> The quotients q_k
     */
    static std::vector<Polynomial> compute_multilinear_quotients(Polynomial& polynomial,
                                                                 std::span<const FF> u_challenge)
    {
        size_t log_N = numeric::get_msb(polynomial.size());
        // Define the vector of quotients q_k, k = 0, ..., log_N-1
        std::vector<Polynomial> quotients;
        for (size_t k = 0; k < log_N; ++k) {
            size_t size = 1 << k;
            quotients.emplace_back(Polynomial(size)); // degree 2^k - 1
        }

        // Compute the coefficients of q_{n-1}
        size_t size_q = 1 << (log_N - 1);
        Polynomial q{ size_q };
        for (size_t l = 0; l < size_q; ++l) {
            q[l] = polynomial[size_q + l] - polynomial[l];
        }

        quotients[log_N - 1] = q.share();

        std::vector<FF> f_k;
        f_k.resize(size_q);

        std::vector<FF> g(polynomial.data().get(), polynomial.data().get() + size_q);

        // Compute q_k in reverse order from k= n-2, i.e. q_{n-2}, ..., q_0
        for (size_t k = 1; k < log_N; ++k) {
            // Compute f_k
            for (size_t l = 0; l < size_q; ++l) {
                f_k[l] = g[l] + u_challenge[log_N - k] * q[l];
            }

            size_q = size_q / 2;
            q = Polynomial{ size_q };

            for (size_t l = 0; l < size_q; ++l) {
                q[l] = f_k[size_q + l] - f_k[l];
            }

            quotients[log_N - k - 1] = q.share();
            g = f_k;
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
     *           + concatentation_term
     *
     * where f_batched = \sum_{i=0}^{m-1}\rho^i*f_i, g_batched = \sum_{i=0}^{l-1}\rho^{m+i}*g_i
     *
     * and concatenation_term = \sum_{i=0}^{num_chunks_per_group}(x^{i * min_N + 1}concatenation_groups_batched_{i})
     *
     * @note The concatenation term arises from an implementation detail in the Translator and is not part of the
     * conventional ZM protocol
     * @param input_polynomial
     * @param quotients
     * @param v_evaluation
     * @param x_challenge
     * @return Polynomial
     */
    static Polynomial compute_partially_evaluated_zeromorph_identity_polynomial(
        Polynomial& f_batched,
        Polynomial& g_batched,
        std::vector<Polynomial>& quotients,
        FF v_evaluation,
        std::span<const FF> u_challenge,
        FF x_challenge,
        std::vector<Polynomial> concatenation_groups_batched = {})
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

        // If necessary, add to Z_x the contribution related to concatenated polynomials:
        // \sum_{i=0}^{num_chunks_per_group}(x^{i * min_n + 1}concatenation_groups_batched_{i}).
        // We are effectively reconstructing concatenated polynomials from their chunks now that we know x
        // Note: this is an implementation detail related to Translator and is not part of the standard protocol.
        if (!concatenation_groups_batched.empty()) {
            size_t MINICIRCUIT_N = N / concatenation_groups_batched.size();
            auto x_to_minicircuit_N =
                x_challenge.pow(MINICIRCUIT_N); // power of x used to shift polynomials to the right
            auto running_shift = x_challenge;
            for (size_t i = 0; i < concatenation_groups_batched.size(); i++) {
                result.add_scaled(concatenation_groups_batched[i], running_shift);
                running_shift *= x_to_minicircuit_N;
            }
        }

        return result;
    }

    /**
     * @brief Compute combined evaluation and degree-check polynomial pi
     * @details Compute univariate polynomial pi, where
     *
     *  pi = (\zeta_c + z*Z_x) X^{N_{max}-(N-1)}
     *
     * The proof that pi(x) = 0 for some verifier challenge x will then be computed as part of the univariate PCS
     * opening. If this is instantiated with KZG, the PCS is going to compute the quotient
     * q_pi = (q_\zeta + z*q_Z)X^{N_{max}-(N-1)}, with q_\zeta = \zeta_x/(X-x), q_Z = Z_x/(X-x),
     *
     * @param Z_x
     * @param zeta_x
     * @param x_challenge
     * @param z_challenge
     * @param N_max
     * @return Polynomial
     */
    static Polynomial compute_batched_evaluation_and_degree_check_polynomial(Polynomial& zeta_x,
                                                                             Polynomial& Z_x,
                                                                             FF z_challenge)
    {
        // We cannot commit to polynomials with size > N_max
        size_t N = zeta_x.size();
        ASSERT(N <= N_max);

        // Compute batched polynomial zeta_x + Z_x
        auto batched_polynomial = zeta_x;
        batched_polynomial.add_scaled(Z_x, z_challenge);

        // TODO(#742): To complete the degree check, we need to do an opening proof for x_challenge with a univariate
        // PCS for the degree-lifted polynomial (\zeta_c + z*Z_x)*X^{N_max - N - 1}. If this PCS is KZG, verification
        // then requires a pairing check similar to the standard KZG check but with [1]_2 replaced by [X^{N_max - N
        // -1}]_2. Two issues: A) we do not have an SRS with these G2 elements (so need to generate a fake setup until
        // we can do the real thing), and B) its not clear to me how to update our pairing algorithms to do this type of
        // pairing. For now, simply construct pi without the shift and do a standard KZG pairing check if the PCS is
        // KZG. When we're ready, all we have to do to make this fully legit is commit to the shift here and update the
        // pairing check accordingly. Note: When this is implemented properly, it doesnt make sense to store the
        // (massive) shifted polynomial of size N_max. Ideally would only store the unshifted version and just compute
        // the shifted commitment directly via a new method.

        return batched_polynomial;
    }

    /**
     * @brief  * @brief Returns a univariate opening claim equivalent to a set of multilinear evaluation claims for
     * unshifted polynomials f_i and to-be-shifted polynomials g_i to be subsequently proved with a univariate PCS
     *
     * @param f_polynomials Unshifted polynomials
     * @param g_polynomials To-be-shifted polynomials (of which the shifts h_i were evaluated by sumcheck)
     * @param evaluations Set of evaluations v_i = f_i(u), w_i = h_i(u) = g_i_shifted(u)
     * @param multilinear_challenge Multilinear challenge point u
     * @param commitment_key
     * @param transcript
     *
     * @todo https://github.com/AztecProtocol/barretenberg/issues/1030: document concatenation trick
     */
    template <typename Transcript>
    static OpeningClaim prove(FF circuit_size,
                              RefSpan<Polynomial> f_polynomials,
                              RefSpan<Polynomial> g_polynomials,
                              RefSpan<FF> f_evaluations,
                              RefSpan<FF> g_shift_evaluations,
                              std::span<FF> multilinear_challenge,
                              const std::shared_ptr<CommitmentKey<Curve>>& commitment_key,
                              const std::shared_ptr<Transcript>& transcript,
                              RefSpan<Polynomial> concatenated_polynomials = {},
                              RefSpan<FF> concatenated_evaluations = {},
                              const std::vector<RefVector<Polynomial>>& concatenation_groups = {})
    {
        // Generate batching challenge \rho and powers 1,...,\rho^{m-1}
        const FF rho = transcript->template get_challenge<FF>("rho");

        // Extract multilinear challenge u and claimed multilinear evaluations from Sumcheck output
        std::span<const FF> u_challenge = multilinear_challenge;
        size_t log_N = numeric::get_msb(static_cast<uint32_t>(circuit_size));
        size_t N = 1 << log_N;

        // Compute batching of unshifted polynomials f_i and to-be-shifted polynomials g_i:
        // f_batched = sum_{i=0}^{m-1}\rho^i*f_i and g_batched = sum_{i=0}^{l-1}\rho^{m+i}*g_i,
        // and also batched evaluation
        // v = sum_{i=0}^{m-1}\rho^i*f_i(u) + sum_{i=0}^{l-1}\rho^{m+i}*h_i(u).
        // Note: g_batched is formed from the to-be-shifted polynomials, but the batched evaluation incorporates the
        // evaluations produced by sumcheck of h_i = g_i_shifted.
        FF batched_evaluation{ 0 };
        Polynomial f_batched(N); // batched unshifted polynomials
        FF batching_scalar{ 1 };
        for (auto [f_poly, f_eval] : zip_view(f_polynomials, f_evaluations)) {
            f_batched.add_scaled(f_poly, batching_scalar);
            batched_evaluation += batching_scalar * f_eval;
            batching_scalar *= rho;
        }

        Polynomial g_batched{ N }; // batched to-be-shifted polynomials
        for (auto [g_poly, g_shift_eval] : zip_view(g_polynomials, g_shift_evaluations)) {
            g_batched.add_scaled(g_poly, batching_scalar);
            batched_evaluation += batching_scalar * g_shift_eval;
            batching_scalar *= rho;
        };

        size_t num_groups = concatenation_groups.size();
        size_t num_chunks_per_group = concatenation_groups.empty() ? 0 : concatenation_groups[0].size();
        // Concatenated polynomials
        Polynomial concatenated_batched(N);

        // construct concatention_groups_batched
        std::vector<Polynomial> concatenation_groups_batched;
        for (size_t i = 0; i < num_chunks_per_group; ++i) {
            concatenation_groups_batched.push_back(Polynomial(N));
        }
        // for each group
        for (size_t i = 0; i < num_groups; ++i) {
            concatenated_batched.add_scaled(concatenated_polynomials[i], batching_scalar);
            // for each element in a group
            for (size_t j = 0; j < num_chunks_per_group; ++j) {
                concatenation_groups_batched[j].add_scaled(concatenation_groups[i][j], batching_scalar);
            }
            batched_evaluation += batching_scalar * concatenated_evaluations[i];
            batching_scalar *= rho;
        }

        // Compute the full batched polynomial f = f_batched + g_batched.shifted() = f_batched + h_batched. This is the
        // polynomial for which we compute the quotients q_k and prove f(u) = v_batched.
        Polynomial f_polynomial = f_batched;
        f_polynomial += g_batched.shifted();
        f_polynomial += concatenated_batched;

        // Compute the multilinear quotients q_k = q_k(X_0, ..., X_{k-1})
        std::vector<Polynomial> quotients = compute_multilinear_quotients(f_polynomial, u_challenge);
        // Compute and send commitments C_{q_k} = [q_k], k = 0,...,d-1
        for (size_t idx = 0; idx < log_N; ++idx) {
            Commitment q_k_commitment = commitment_key->commit(quotients[idx]);
            std::string label = "ZM:C_q_" + std::to_string(idx);
            transcript->send_to_verifier(label, q_k_commitment);
        }
        // Add buffer elements to remove log_N dependence in proof
        for (size_t idx = log_N; idx < CONST_PROOF_SIZE_LOG_N; ++idx) {
            auto buffer_element = Commitment::one();
            std::string label = "ZM:C_q_" + std::to_string(idx);
            transcript->send_to_verifier(label, buffer_element);
        }

        // Get challenge y
        FF y_challenge = transcript->template get_challenge<FF>("ZM:y");

        // Compute the batched, lifted-degree quotient \hat{q}
        auto batched_quotient = compute_batched_lifted_degree_quotient(quotients, y_challenge, N);

        // Compute and send the commitment C_q = [\hat{q}]
        auto q_commitment = commitment_key->commit(batched_quotient);
        transcript->send_to_verifier("ZM:C_q", q_commitment);

        // Get challenges x and z
        auto [x_challenge, z_challenge] = transcript->template get_challenges<FF>("ZM:x", "ZM:z");

        // Compute degree check polynomial \zeta partially evaluated at x
        auto zeta_x =
            compute_partially_evaluated_degree_check_polynomial(batched_quotient, quotients, y_challenge, x_challenge);

        // Compute ZeroMorph identity polynomial Z partially evaluated at x
        auto Z_x = compute_partially_evaluated_zeromorph_identity_polynomial(f_batched,
                                                                             g_batched,
                                                                             quotients,
                                                                             batched_evaluation,
                                                                             u_challenge,
                                                                             x_challenge,
                                                                             concatenation_groups_batched);

        // Compute batched degree-check and ZM-identity quotient polynomial pi
        auto pi_polynomial = compute_batched_evaluation_and_degree_check_polynomial(zeta_x, Z_x, z_challenge);

        // Returns the claim used to generate an opening proof for the univariate polynomial at x_challenge
        return { pi_polynomial, { .challenge = x_challenge, .evaluation = FF(0) } };
    }
};

/**
 * @brief Verifier for ZeroMorph multilinear PCS
 *
 * @tparam Curve - The Curve used to arithmetise ZeroMorph
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
    static Commitment compute_C_zeta_x(const Commitment& C_q,
                                       std::vector<Commitment>& C_q_k,
                                       FF y_challenge,
                                       FF x_challenge,
                                       const FF log_circuit_size,
                                       const FF circuit_size)
    {
        size_t N{ 0 };
        size_t log_N{ 0 };
        if constexpr (Curve::is_stdlib_type) {
            N = static_cast<uint32_t>(circuit_size.get_value());
            log_N = static_cast<uint32_t>(log_circuit_size.get_value());
        } else {
            N = static_cast<uint32_t>(circuit_size);
            log_N = static_cast<uint32_t>(log_circuit_size);
        }

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

        // Contribution from C_q_k, k = 0,...,log_N-1
        for (size_t k = 0; k < CONST_PROOF_SIZE_LOG_N; ++k) {
            // Utilize dummy rounds in order to make verifier circuit independent of proof size
            bool is_dummy_round = k >= log_N;
            auto deg_k = static_cast<size_t>((1 << k) - 1);
            // Compute scalar y^k * x^{N - deg_k - 1}
            // TODO(https://github.com/AztecProtocol/barretenberg/issues/1039): pow may not add proper constraints
            FF scalar = y_challenge.pow(k);
            size_t x_exponent = is_dummy_round ? 0 : N - deg_k - 1;
            scalar *= x_challenge.pow(x_exponent);
            scalar *= FF(-1);
            if constexpr (Curve::is_stdlib_type) {
                auto builder = x_challenge.get_context();
                FF zero = FF::from_witness(builder, 0);
                stdlib::bool_t dummy_round = stdlib::witness_t(builder, is_dummy_round);
                // TODO(https://github.com/AztecProtocol/barretenberg/issues/1039): is it kosher to reassign like this?
                scalar = FF::conditional_assign(dummy_round, zero, scalar);
            } else {
                if (is_dummy_round) {
                    scalar = 0;
                }
            }
            scalars.emplace_back(scalar);
            commitments.emplace_back(C_q_k[k]);
        }

        // Compute batch mul to get the result
        if constexpr (Curve::is_stdlib_type) {
            // If Ultra and using biggroup, handle edge cases in batch_mul
            if constexpr (IsUltraBuilder<typename Curve::Builder> && stdlib::IsBigGroup<Commitment>) {
                return Commitment::batch_mul(commitments, scalars, /*max_num_bits=*/0, /*with_edgecases=*/true);
            } else {
                return Commitment::batch_mul(commitments, scalars);
            }
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
     *              + concatentation_term
     * where
     *
     *  concatenation_term = \sum{i=0}^{o-1}\sum_{j=0}^{num_chunks_per_group}(rho^{m+l+i} * x^{j * min_N + 1}
     *                       * concatenation_groups_commitments_{i}_{j})
     *
     * @note The concatenation term arises from an implementation detail in the Translator and is not part of the
     * conventional ZM protocol
     * @param g1_identity first element in the SRS
     * @param f_commitments Commitments to unshifted polynomials [f_i]
     * @param g_commitments Commitments to to-be-shifted polynomials [g_i]
     * @param C_q_k Commitments to q_k
     * @param rho
     * @param batched_evaluation \sum_{i=0}^{m-1} \rho^i*f_i(u) + \sum_{i=0}^{l-1} \rho^{m+i}*h_i(u)
     * @param x_challenge
     * @param u_challenge multilinear challenge
     * @param concatenation_groups_commitments
     * @return Commitment
     */
    static Commitment compute_C_Z_x(const Commitment& g1_identity,
                                    RefSpan<Commitment> f_commitments,
                                    RefSpan<Commitment> g_commitments,
                                    std::span<Commitment> C_q_k,
                                    FF rho,
                                    FF batched_evaluation,
                                    FF x_challenge,
                                    std::span<FF> u_challenge,
                                    const FF log_circuit_size,
                                    const FF circuit_size,
                                    const std::vector<RefVector<Commitment>>& concatenation_groups_commitments = {})
    {
        size_t N{ 0 };
        size_t log_N{ 0 };
        if constexpr (Curve::is_stdlib_type) {
            N = static_cast<uint32_t>(circuit_size.get_value());
            log_N = static_cast<uint32_t>(log_circuit_size.get_value());
        } else {
            N = static_cast<uint32_t>(circuit_size);
            log_N = static_cast<uint32_t>(log_circuit_size);
        }

        std::vector<FF> scalars;
        std::vector<Commitment> commitments;

        // Phi_n(x) = (x^N - 1) / (x - 1)
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/1039): pow may not add proper constraints
        auto phi_numerator = x_challenge.pow(N) - 1; // x^N - 1
        auto phi_n_x = phi_numerator / (x_challenge - 1);

        // Add contribution: -v * x * \Phi_n(x) * [1]_1
        scalars.emplace_back(FF(-1) * batched_evaluation * x_challenge * phi_n_x);

        commitments.emplace_back(g1_identity);

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

        // If applicable, add contribution from concatenated polynomial commitments
        // Note: this is an implementation detail related to Translator and is not part of the standard protocol.
        if (!concatenation_groups_commitments.empty()) {
            size_t CONCATENATION_GROUP_SIZE = concatenation_groups_commitments[0].size();
            size_t MINICIRCUIT_N = N / CONCATENATION_GROUP_SIZE;
            std::vector<FF> x_shifts;
            auto current_x_shift = x_challenge;
            auto x_to_minicircuit_n = x_challenge.pow(MINICIRCUIT_N);
            for (size_t i = 0; i < CONCATENATION_GROUP_SIZE; ++i) {
                x_shifts.emplace_back(current_x_shift);
                current_x_shift *= x_to_minicircuit_n;
            }
            for (auto& concatenation_group_commitment : concatenation_groups_commitments) {
                for (size_t i = 0; i < CONCATENATION_GROUP_SIZE; ++i) {
                    scalars.emplace_back(rho_pow * x_shifts[i]);
                    commitments.emplace_back(concatenation_group_commitment[i]);
                }
                rho_pow *= rho;
            }
        }

        // Add contributions: scalar * [q_k],  k = 0,...,log_N, where
        // scalar = -x * (x^{2^k} * \Phi_{n-k-1}(x^{2^{k+1}}) - u_k * \Phi_{n-k}(x^{2^k}))
        auto x_pow_2k = x_challenge;                 // x^{2^k}
        auto x_pow_2kp1 = x_challenge * x_challenge; // x^{2^{k + 1}}
        for (size_t k = 0; k < CONST_PROOF_SIZE_LOG_N; ++k) {
            // Utilize dummy rounds in order to make verifier circuit independent of proof size
            bool is_dummy_round = k >= log_N;
            if constexpr (Curve::is_stdlib_type) {
                auto builder = x_challenge.get_context();
                stdlib::bool_t dummy_scalar = stdlib::witness_t(builder, is_dummy_round);
                auto phi_term_1 = phi_numerator / (x_pow_2kp1 - 1); // \Phi_{n-k-1}(x^{2^{k + 1}})
                auto phi_term_2 = phi_numerator / (x_pow_2k - 1);   // \Phi_{n-k}(x^{2^k})

                auto scalar = x_pow_2k * phi_term_1;
                scalar -= u_challenge[k] * phi_term_2;
                scalar *= x_challenge;
                scalar *= -FF(1);

                FF zero = FF::from_witness(builder, 0);
                scalar = FF::conditional_assign(dummy_scalar, zero, scalar);
                scalars.emplace_back(scalar);
                commitments.emplace_back(C_q_k[k]);

                x_pow_2k = FF::conditional_assign(dummy_scalar, x_pow_2k, x_pow_2kp1);
                x_pow_2kp1 = FF::conditional_assign(dummy_scalar, x_pow_2kp1, x_pow_2kp1 * x_pow_2kp1);
            } else {
                if (is_dummy_round) {
                    scalars.emplace_back(0);
                    commitments.emplace_back(C_q_k[k]);
                } else {
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
            }
        }

        if constexpr (Curve::is_stdlib_type) {
            // If Ultra and using biggroup, handle edge cases in batch_mul
            if constexpr (IsUltraBuilder<typename Curve::Builder> && stdlib::IsBigGroup<Commitment>) {
                return Commitment::batch_mul(commitments, scalars, /*max_num_bits=*/0, /*with_edgecases=*/true);
            } else {
                return Commitment::batch_mul(commitments, scalars);
            }
        } else {
            return batch_mul_native(commitments, scalars);
        }
    }

    /**
     * @brief Utility for native batch multiplication of group elements
     * @note This is used only for native verification and is not optimized for efficiency
     */
    static Commitment batch_mul_native(const std::vector<Commitment>& _points, const std::vector<FF>& _scalars)
    {
        std::vector<Commitment> points;
        std::vector<FF> scalars;
        for (auto [point, scalar] : zip_view(_points, _scalars)) {
            // TODO(https://github.com/AztecProtocol/barretenberg/issues/866) Special handling of point at infinity here
            // due to incorrect serialization.
            if (!scalar.is_zero() && !point.is_point_at_infinity() && !point.y.is_zero()) {
                points.emplace_back(point);
                scalars.emplace_back(scalar);
            }
        }

        if (points.empty()) {
            return Commitment::infinity();
        }

        auto result = points[0] * scalars[0];
        for (size_t idx = 1; idx < scalars.size(); ++idx) {
            result = result + points[idx] * scalars[idx];
        }
        return result;
    }

    /**
     * @brief Return the univariate opening claim used to verify, in a subsequent PCS, a set of multilinear evaluation
     * claims for unshifted polynomials f_i and to-be-shifted polynomials g_i
     *
     * @param commitments Commitments to polynomials f_i and g_i (unshifted and to-be-shifted)
     * @param claimed_evaluations Claimed evaluations v_i = f_i(u) and w_i = h_i(u) = g_i_shifted(u)
     * @param multivariate_challenge Challenge point u
     * @param transcript
     * @return VerifierAccumulator Inputs to the final PCS verification check that will be accumulated
     */
    template <typename Transcript>
    static OpeningClaim<Curve> verify(FF circuit_size,
                                      RefSpan<Commitment> unshifted_commitments,
                                      RefSpan<Commitment> to_be_shifted_commitments,
                                      RefSpan<FF> unshifted_evaluations,
                                      RefSpan<FF> shifted_evaluations,
                                      std::span<FF> multivariate_challenge,
                                      const Commitment& g1_identity,
                                      const std::shared_ptr<Transcript>& transcript,
                                      const std::vector<RefVector<Commitment>>& concatenation_group_commitments = {},
                                      RefSpan<FF> concatenated_evaluations = {})
    {
        FF log_N;
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/1039): Connect witness log_N to circuit size
        if constexpr (Curve::is_stdlib_type) {
            log_N = FF(static_cast<int>(numeric::get_msb(static_cast<uint32_t>(circuit_size.get_value()))));
        } else {
            log_N = numeric::get_msb(static_cast<uint32_t>(circuit_size));
        }
        FF rho = transcript->template get_challenge<FF>("rho");

        // Construct batched evaluation v = sum_{i=0}^{m-1}\rho^i*f_i(u) + sum_{i=0}^{l-1}\rho^{m+i}*h_i(u)
        FF batched_evaluation = FF(0);
        FF batching_scalar = FF(1);
        for (auto& value : unshifted_evaluations) {
            batched_evaluation += value * batching_scalar;
            batching_scalar *= rho;
        }
        for (auto& value : shifted_evaluations) {
            batched_evaluation += value * batching_scalar;
            batching_scalar *= rho;
        }
        for (auto& value : concatenated_evaluations) {
            batched_evaluation += value * batching_scalar;
            batching_scalar *= rho;
        }

        // Receive commitments [q_k]
        std::vector<Commitment> C_q_k;
        C_q_k.reserve(CONST_PROOF_SIZE_LOG_N);
        for (size_t i = 0; i < CONST_PROOF_SIZE_LOG_N; ++i) {
            C_q_k.emplace_back(transcript->template receive_from_prover<Commitment>("ZM:C_q_" + std::to_string(i)));
        }

        // Challenge y
        FF y_challenge = transcript->template get_challenge<FF>("ZM:y");

        // Receive commitment C_{q}
        auto C_q = transcript->template receive_from_prover<Commitment>("ZM:C_q");

        // Challenges x, z
        auto [x_challenge, z_challenge] = transcript->template get_challenges<FF>("ZM:x", "ZM:z");

        // Compute commitment C_{\zeta_x}
        auto C_zeta_x = compute_C_zeta_x(C_q, C_q_k, y_challenge, x_challenge, log_N, circuit_size);

        // Compute commitment C_{Z_x}
        Commitment C_Z_x = compute_C_Z_x(g1_identity,
                                         unshifted_commitments,
                                         to_be_shifted_commitments,
                                         C_q_k,
                                         rho,
                                         batched_evaluation,
                                         x_challenge,
                                         multivariate_challenge,
                                         log_N,
                                         circuit_size,
                                         concatenation_group_commitments);

        // Compute commitment C_{\zeta,Z}
        Commitment C_zeta_Z;
        if constexpr (Curve::is_stdlib_type) {
            // Express operation as a batch_mul in order to use Goblinization if available
            auto builder = z_challenge.get_context();
            std::vector<FF> scalars = { FF(builder, 1), z_challenge };
            std::vector<Commitment> points = { C_zeta_x, C_Z_x };
            // If Ultra and using biggroup, handle edge cases in batch_mul
            if constexpr (IsUltraBuilder<typename Curve::Builder> && stdlib::IsBigGroup<Commitment>) {
                C_zeta_Z = Commitment::batch_mul(points, scalars, /*max_num_bits=*/0, /*with_edgecases=*/true);
            } else {
                C_zeta_Z = Commitment::batch_mul(points, scalars);
            }
        } else {
            C_zeta_Z = C_zeta_x + C_Z_x * z_challenge;
        }

        return { .opening_pair = { .challenge = x_challenge, .evaluation = FF(0) }, .commitment = C_zeta_Z };
    }
};

} // namespace bb
