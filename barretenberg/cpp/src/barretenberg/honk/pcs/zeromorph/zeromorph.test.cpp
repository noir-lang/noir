#include "zeromorph.hpp"
#include "../commitment_key.test.hpp"
#include "barretenberg/transcript/transcript.hpp"

#include <gtest/gtest.h>

namespace proof_system::honk::pcs::zeromorph {

template <class Curve> class ZeroMorphTest : public CommitmentTest<Curve> {
  public:
    using Fr = typename Curve::ScalarField;
    using Polynomial = barretenberg::Polynomial<Fr>;
    using Commitment = typename Curve::AffineElement;
    using GroupElement = typename Curve::Element;
    using ZeroMorphProver = ZeroMorphProver_<Curve>;
    using ZeroMorphVerifier = ZeroMorphVerifier_<Curve>;

    // Evaluate Phi_k(x) = \sum_{i=0}^k x^i using the direct inefficent formula
    Fr Phi(Fr challenge, size_t subscript)
    {
        size_t length = 1 << subscript;
        auto result = Fr(0);
        for (size_t idx = 0; idx < length; ++idx) {
            result += challenge.pow(idx);
        }
        return result;
    }

    /**
     * @brief Construct and verify ZeroMorph proof of batched multilinear evaluation with shifts
     * @details The goal is to construct and verify a single batched multilinear evaluation proof for m polynomials f_i
     * and l polynomials h_i. It is assumed that the h_i are shifts of polynomials g_i (the "to-be-shifted"
     * polynomials), which are a subset of the f_i. This is what is encountered in practice. We accomplish this using
     * evaluations of h_i but commitments to only their unshifted counterparts g_i (which we get for "free" since
     * commitments [g_i] are contained in the set of commitments [f_i]).
     *
     */
    bool execute_zeromorph_protocol(size_t NUM_UNSHIFTED, size_t NUM_SHIFTED)
    {
        bool verified = false;

        size_t N = 16;
        size_t log_N = numeric::get_msb(N);

        auto u_challenge = this->random_evaluation_point(log_N);

        // Construct some random multilinear polynomials f_i and their evaluations v_i = f_i(u)
        std::vector<Polynomial> f_polynomials; // unshifted polynomials
        std::vector<Fr> v_evaluations;
        for (size_t i = 0; i < NUM_UNSHIFTED; ++i) {
            f_polynomials.emplace_back(this->random_polynomial(N));
            f_polynomials[i][0] = Fr(0); // ensure f is "shiftable"
            v_evaluations.emplace_back(f_polynomials[i].evaluate_mle(u_challenge));
        }

        // Construct some "shifted" multilinear polynomials h_i as the left-shift-by-1 of f_i
        std::vector<Polynomial> g_polynomials; // to-be-shifted polynomials
        std::vector<Polynomial> h_polynomials; // shifts of the to-be-shifted polynomials
        std::vector<Fr> w_evaluations;
        for (size_t i = 0; i < NUM_SHIFTED; ++i) {
            g_polynomials.emplace_back(f_polynomials[i]);
            h_polynomials.emplace_back(g_polynomials[i].shifted());
            w_evaluations.emplace_back(h_polynomials[i].evaluate_mle(u_challenge));
            // ASSERT_EQ(w_evaluations[i], g_polynomials[i].evaluate_mle(u_challenge, /* shift = */ true));
        }

        // Compute commitments [f_i]
        std::vector<Commitment> f_commitments;
        for (size_t i = 0; i < NUM_UNSHIFTED; ++i) {
            f_commitments.emplace_back(this->commit(f_polynomials[i]));
        }

        // Construct container of commitments of the "to-be-shifted" polynomials [g_i] (= [f_i])
        std::vector<Commitment> g_commitments;
        for (size_t i = 0; i < NUM_SHIFTED; ++i) {
            g_commitments.emplace_back(f_commitments[i]);
        }

        // Initialize an empty BaseTranscript
        auto prover_transcript = BaseTranscript<Fr>::prover_init_empty();

        // Execute Prover protocol
        {
            auto rho = prover_transcript.get_challenge("ZM:rho");

            // Compute batching of f_i and g_i polynomials: sum_{i=0}^{m-1}\rho^i*f_i and
            // sum_{i=0}^{l-1}\rho^{m+i}*h_i, and also batched evaluation v = sum_{i=0}^{m-1}\rho^i*v_i +
            // sum_{i=0}^{l-1}\rho^{m+i}*w_i.
            auto f_batched = Polynomial(N);
            auto g_batched = Polynomial(N);
            auto v_evaluation = Fr(0);
            auto rho_pow = Fr(1);
            for (size_t i = 0; i < NUM_UNSHIFTED; ++i) {
                f_batched.add_scaled(f_polynomials[i], rho_pow);
                v_evaluation += rho_pow * v_evaluations[i];
                rho_pow *= rho;
            }
            for (size_t i = 0; i < NUM_SHIFTED; ++i) {
                g_batched.add_scaled(g_polynomials[i], rho_pow);
                v_evaluation += rho_pow * w_evaluations[i];
                rho_pow *= rho;
            }

            // The new f is f_batched + g_batched.shifted() = f_batched + h_batched
            auto f_polynomial = f_batched;
            f_polynomial += g_batched.shifted();

            // Compute the multilinear quotients q_k = q_k(X_0, ..., X_{k-1})
            auto quotients = ZeroMorphProver::compute_multilinear_quotients(f_polynomial, u_challenge);

            // Compute and send commitments C_{q_k} = [q_k], k = 0,...,d-1
            std::vector<Commitment> q_k_commitments;
            q_k_commitments.reserve(log_N);
            for (size_t idx = 0; idx < log_N; ++idx) {
                q_k_commitments[idx] = this->commit(quotients[idx]);
                std::string label = "ZM:C_q_" + std::to_string(idx);
                prover_transcript.send_to_verifier(label, q_k_commitments[idx]);
            }

            // Get challenge y
            auto y_challenge = prover_transcript.get_challenge("ZM:y");

            // Compute the batched, lifted-degree quotient \hat{q}
            auto batched_quotient = ZeroMorphProver::compute_batched_lifted_degree_quotient(quotients, y_challenge, N);

            // Compute and send the commitment C_q = [\hat{q}]
            auto q_commitment = this->commit(batched_quotient);
            prover_transcript.send_to_verifier("ZM:C_q", q_commitment);

            // Get challenges x and z
            auto [x_challenge, z_challenge] = prover_transcript.get_challenges("ZM:x", "ZM:z");

            // Compute degree check polynomial \zeta partially evaluated at x
            auto zeta_x = ZeroMorphProver::compute_partially_evaluated_degree_check_polynomial(
                batched_quotient, quotients, y_challenge, x_challenge);

            // Compute ZeroMorph identity polynomial Z partially evaluated at x
            auto Z_x = ZeroMorphProver::compute_partially_evaluated_zeromorph_identity_polynomial(
                f_batched, g_batched, quotients, v_evaluation, u_challenge, x_challenge);

            // Compute batched degree and ZM-identity quotient polynomial pi
            auto pi_polynomial = ZeroMorphProver::compute_batched_evaluation_and_degree_check_quotient(
                zeta_x, Z_x, x_challenge, z_challenge);

            // Compute and send proof commitment pi
            auto pi_commitment = this->commit(pi_polynomial);
            prover_transcript.send_to_verifier("ZM:PI", pi_commitment);
        }

        auto verifier_transcript = BaseTranscript<Fr>::verifier_init_empty(prover_transcript);

        // Execute Verifier protocol
        {
            // Challenge rho
            auto rho = verifier_transcript.get_challenge("ZM:rho");

            // Construct batched evaluation v = sum_{i=0}^{m-1}\rho^i*v_i + sum_{i=0}^{l-1}\rho^{m+i}*w_i
            auto v_evaluation = Fr(0);
            auto rho_pow = Fr(1);
            for (size_t i = 0; i < NUM_UNSHIFTED; ++i) {
                v_evaluation += rho_pow * v_evaluations[i];
                rho_pow *= rho;
            }
            for (size_t i = 0; i < NUM_SHIFTED; ++i) {
                v_evaluation += rho_pow * w_evaluations[i];
                rho_pow *= rho;
            }

            // Receive commitments [q_k]
            std::vector<Commitment> C_q_k;
            C_q_k.reserve(log_N);
            for (size_t i = 0; i < log_N; ++i) {
                C_q_k.emplace_back(
                    verifier_transcript.template receive_from_prover<Commitment>("ZM:C_q_" + std::to_string(i)));
            }

            // Challenge y
            auto y_challenge = verifier_transcript.get_challenge("ZM:y");

            // Receive commitment C_{q}
            auto C_q = verifier_transcript.template receive_from_prover<Commitment>("ZM:C_q");

            // Challenges x, z
            auto [x_challenge, z_challenge] = verifier_transcript.get_challenges("ZM:x", "ZM:z");

            // Compute commitment C_{\zeta_x}
            auto C_zeta_x = ZeroMorphVerifier::compute_C_zeta_x(C_q, C_q_k, y_challenge, x_challenge);

            // Compute commitment C_{Z_x}
            Commitment C_Z_x = ZeroMorphVerifier::compute_C_Z_x(
                f_commitments, g_commitments, C_q_k, rho, v_evaluation, x_challenge, u_challenge);

            // Compute commitment C_{\zeta,Z}
            auto C_zeta_Z = C_zeta_x + C_Z_x * z_challenge;

            // Receive proof commitment \pi
            auto C_pi = verifier_transcript.template receive_from_prover<Commitment>("ZM:PI");

            // The prover and verifier manifests should agree
            EXPECT_EQ(prover_transcript.get_manifest(), verifier_transcript.get_manifest());

            // Construct inputs and perform pairing check to verify claimed evaluation
            // Note: The pairing check (without the degree check component X^{N_max-N-1}) can be expressed naturally as
            // e(C_{\zeta,Z}, [1]_2) = e(pi, [X - x]_2). This can be rearranged (e.g. see the plonk paper) as
            // e(C_{\zeta,Z} - x*pi, [1]_2) * e(-pi, [X]_2) = 1, or
            // e(P_0, [1]_2) * e(P_1, [X]_2) = 1
            auto P0 = C_zeta_Z + C_pi * x_challenge;
            auto P1 = -C_pi;
            verified = this->vk()->pairing_check(P0, P1);
            // EXPECT_TRUE(verified);
        }
        return verified;
    }
};

template <class Curve> class ZeroMorphWithConcatenationTest : public CommitmentTest<Curve> {
  public:
    using Fr = typename Curve::ScalarField;
    using Polynomial = barretenberg::Polynomial<Fr>;
    using Commitment = typename Curve::AffineElement;
    using GroupElement = typename Curve::Element;
    using ZeroMorphProver = ZeroMorphProver_<Curve>;
    using ZeroMorphVerifier = ZeroMorphVerifier_<Curve>;

    // Evaluate Phi_k(x) = \sum_{i=0}^k x^i using the direct inefficent formula
    Fr Phi(Fr challenge, size_t subscript)
    {
        size_t length = 1 << subscript;
        auto result = Fr(0);
        for (size_t idx = 0; idx < length; ++idx) {
            result += challenge.pow(idx);
        }
        return result;
    }

    /**
     * @brief Construct and verify ZeroMorph proof of batched multilinear evaluation with shifts and concatenation
     * @details The goal is to construct and verify a single batched multilinear evaluation proof for m polynomials f_i,
     * l polynomials h_i and o groups of polynomials where each polynomial is concatenated from several shorter
     * polynomials. It is assumed that the h_i are shifts of polynomials g_i (the "to-be-shifted" polynomials), which
     * are a subset of the f_i. This is what is encountered in practice. We accomplish this using evaluations of h_i but
     * commitments to only their unshifted counterparts g_i (which we get for "free" since commitments [g_i] are
     * contained in the set of commitments [f_i]).
     *
     */
    bool execute_zeromorph_protocol(size_t NUM_UNSHIFTED, size_t NUM_SHIFTED, size_t NUM_CONCATENATED)
    {
        bool verified = false;
        size_t concatenation_index = 2;
        size_t N = 64;
        size_t MINI_CIRCUIT_N = N / concatenation_index;
        size_t log_N = numeric::get_msb(N);

        auto u_challenge = this->random_evaluation_point(log_N);

        // Construct some random multilinear polynomials f_i and their evaluations v_i = f_i(u)
        std::vector<Polynomial> f_polynomials; // unshifted polynomials
        std::vector<Fr> v_evaluations;
        for (size_t i = 0; i < NUM_UNSHIFTED; ++i) {
            f_polynomials.emplace_back(this->random_polynomial(N));
            f_polynomials[i][0] = Fr(0); // ensure f is "shiftable"
            v_evaluations.emplace_back(f_polynomials[i].evaluate_mle(u_challenge));
        }

        // Construct some "shifted" multilinear polynomials h_i as the left-shift-by-1 of f_i
        std::vector<Polynomial> g_polynomials; // to-be-shifted polynomials
        std::vector<Polynomial> h_polynomials; // shifts of the to-be-shifted polynomials
        std::vector<Fr> w_evaluations;
        for (size_t i = 0; i < NUM_SHIFTED; ++i) {
            g_polynomials.emplace_back(f_polynomials[i]);
            h_polynomials.emplace_back(g_polynomials[i].shifted());
            w_evaluations.emplace_back(h_polynomials[i].evaluate_mle(u_challenge));
            // ASSERT_EQ(w_evaluations[i], g_polynomials[i].evaluate_mle(u_challenge, /* shift = */ true));
        }

        // Polynomials "chunks" that are concatenated in the PCS
        std::vector<std::vector<Polynomial>> concatenation_groups;

        // Concatenated polynomials
        std::vector<Polynomial> concatenated_polynomials;

        // Evaluations of concatenated polynomials
        std::vector<Fr> c_evaluations;

        // For each polynomial to be concatenated
        for (size_t i = 0; i < NUM_CONCATENATED; ++i) {
            std::vector<Polynomial> concatenation_group;
            Polynomial concatenated_polynomial(N);
            // For each chunk
            for (size_t j = 0; j < concatenation_index; j++) {
                Polynomial chunk_polynomial(N);
                // Fill the chunk polynomial with random values and appropriately fill the space in
                // concatenated_polynomial
                for (size_t k = 0; k < MINI_CIRCUIT_N; k++) {
                    // Chunks should be shiftable
                    auto tmp = Fr(0);
                    if (k > 0) {
                        tmp = Fr::random_element(this->engine);
                    }
                    chunk_polynomial[k] = tmp;
                    concatenated_polynomial[j * MINI_CIRCUIT_N + k] = tmp;
                }
                concatenation_group.emplace_back(chunk_polynomial);
            }
            // Store chunks
            concatenation_groups.emplace_back(concatenation_group);
            // Store concatenated polynomial
            concatenated_polynomials.emplace_back(concatenated_polynomial);
            // Get evaluation
            c_evaluations.emplace_back(concatenated_polynomial.evaluate_mle(u_challenge));
        }

        // Compute commitments [f_i]
        std::vector<Commitment> f_commitments;
        for (size_t i = 0; i < NUM_UNSHIFTED; ++i) {
            f_commitments.emplace_back(this->commit(f_polynomials[i]));
        }

        // Construct container of commitments of the "to-be-shifted" polynomials [g_i] (= [f_i])
        std::vector<Commitment> g_commitments;
        for (size_t i = 0; i < NUM_SHIFTED; ++i) {
            g_commitments.emplace_back(f_commitments[i]);
        }

        // Compute commitments of all polynomial chunks
        std::vector<std::vector<Commitment>> concatenation_groups_commitments;
        for (size_t i = 0; i < NUM_CONCATENATED; ++i) {
            std::vector<Commitment> concatenation_group_commitment;
            for (size_t j = 0; j < concatenation_index; j++) {
                concatenation_group_commitment.emplace_back(this->commit(concatenation_groups[i][j]));
            }
            concatenation_groups_commitments.emplace_back(concatenation_group_commitment);
        }

        // Initialize an empty BaseTranscript
        auto prover_transcript = BaseTranscript<Fr>::prover_init_empty();

        // Execute Prover protocol
        {
            auto rho = prover_transcript.get_challenge("ZM:rho");

            // Compute batching of f_i and g_i polynomials: sum_{i=0}^{m-1}\rho^i*f_i and
            // sum_{i=0}^{l-1}\rho^{m+i}*h_i, and also batched evaluation v = sum_{i=0}^{m-1}\rho^i*v_i +
            // sum_{i=0}^{l-1}\rho^{m+i}*w_i.
            auto f_batched = Polynomial(N);
            auto g_batched = Polynomial(N);
            auto concatenated_batched = Polynomial(N);
            std::vector<Polynomial> concatenation_groups_batched;
            auto v_evaluation = Fr(0);
            auto rho_pow = Fr(1);
            for (size_t i = 0; i < NUM_UNSHIFTED; ++i) {
                f_batched.add_scaled(f_polynomials[i], rho_pow);
                v_evaluation += rho_pow * v_evaluations[i];
                rho_pow *= rho;
            }
            for (size_t i = 0; i < NUM_SHIFTED; ++i) {
                g_batched.add_scaled(g_polynomials[i], rho_pow);
                v_evaluation += rho_pow * w_evaluations[i];
                rho_pow *= rho;
            }
            for (size_t i = 0; i < concatenation_index; ++i) {
                concatenation_groups_batched.push_back(Polynomial(N));
            }
            for (size_t i = 0; i < NUM_CONCATENATED; ++i) {
                concatenated_batched.add_scaled(concatenated_polynomials[i], rho_pow);
                for (size_t j = 0; j < concatenation_index; ++j) {
                    concatenation_groups_batched[j].add_scaled(concatenation_groups[i][j], rho_pow);
                }
                v_evaluation += rho_pow * c_evaluations[i];
                rho_pow *= rho;
            }

            // The new f is f_batched + g_batched.shifted() = f_batched + h_batched
            auto f_polynomial = f_batched;
            f_polynomial += g_batched.shifted();
            f_polynomial += concatenated_batched;

            // Compute the multilinear quotients q_k = q_k(X_0, ..., X_{k-1})
            auto quotients = ZeroMorphProver::compute_multilinear_quotients(f_polynomial, u_challenge);

            // Compute and send commitments C_{q_k} = [q_k], k = 0,...,d-1
            std::vector<Commitment> q_k_commitments;
            q_k_commitments.reserve(log_N);
            for (size_t idx = 0; idx < log_N; ++idx) {
                q_k_commitments[idx] = this->commit(quotients[idx]);
                std::string label = "ZM:C_q_" + std::to_string(idx);
                prover_transcript.send_to_verifier(label, q_k_commitments[idx]);
            }

            // Get challenge y
            auto y_challenge = prover_transcript.get_challenge("ZM:y");

            // Compute the batched, lifted-degree quotient \hat{q}
            auto batched_quotient = ZeroMorphProver::compute_batched_lifted_degree_quotient(quotients, y_challenge, N);

            // Compute and send the commitment C_q = [\hat{q}]
            auto q_commitment = this->commit(batched_quotient);
            prover_transcript.send_to_verifier("ZM:C_q", q_commitment);

            // Get challenges x and z
            auto [x_challenge, z_challenge] = prover_transcript.get_challenges("ZM:x", "ZM:z");

            // Compute degree check polynomial \zeta partially evaluated at x
            auto zeta_x = ZeroMorphProver::compute_partially_evaluated_degree_check_polynomial(
                batched_quotient, quotients, y_challenge, x_challenge);

            // Compute ZeroMorph identity polynomial Z partially evaluated at x
            auto Z_x = ZeroMorphProver::compute_partially_evaluated_zeromorph_identity_polynomial(
                f_batched, g_batched, quotients, v_evaluation, u_challenge, x_challenge, concatenation_groups_batched);

            // Compute batched degree and ZM-identity quotient polynomial pi
            auto pi_polynomial = ZeroMorphProver::compute_batched_evaluation_and_degree_check_quotient(
                zeta_x, Z_x, x_challenge, z_challenge);

            // Compute and send proof commitment pi
            auto pi_commitment = this->commit(pi_polynomial);
            prover_transcript.send_to_verifier("ZM:PI", pi_commitment);
        }

        auto verifier_transcript = BaseTranscript<Fr>::verifier_init_empty(prover_transcript);

        // Execute Verifier protocol
        {
            // Challenge rho
            auto rho = verifier_transcript.get_challenge("ZM:rho");

            // Construct batched evaluation v = sum_{i=0}^{m-1}\rho^i*v_i + sum_{i=0}^{l-1}\rho^{m+i}*w_i
            auto v_evaluation = Fr(0);
            auto rho_pow = Fr(1);
            for (size_t i = 0; i < NUM_UNSHIFTED; ++i) {
                v_evaluation += rho_pow * v_evaluations[i];
                rho_pow *= rho;
            }
            for (size_t i = 0; i < NUM_SHIFTED; ++i) {
                v_evaluation += rho_pow * w_evaluations[i];
                rho_pow *= rho;
            }
            for (size_t i = 0; i < NUM_CONCATENATED; ++i) {
                v_evaluation += rho_pow * c_evaluations[i];
                rho_pow *= rho;
            }
            // Receive commitments [q_k]
            std::vector<Commitment> C_q_k;
            C_q_k.reserve(log_N);
            for (size_t i = 0; i < log_N; ++i) {
                C_q_k.emplace_back(
                    verifier_transcript.template receive_from_prover<Commitment>("ZM:C_q_" + std::to_string(i)));
            }

            // Challenge y
            auto y_challenge = verifier_transcript.get_challenge("ZM:y");

            // Receive commitment C_{q}
            auto C_q = verifier_transcript.template receive_from_prover<Commitment>("ZM:C_q");

            // Challenges x, z
            auto [x_challenge, z_challenge] = verifier_transcript.get_challenges("ZM:x", "ZM:z");

            // Compute commitment C_{\zeta_x}
            auto C_zeta_x = ZeroMorphVerifier::compute_C_zeta_x(C_q, C_q_k, y_challenge, x_challenge);

            // Compute commitment C_{Z_x}
            Commitment C_Z_x = ZeroMorphVerifier::compute_C_Z_x(f_commitments,
                                                                g_commitments,
                                                                C_q_k,
                                                                rho,
                                                                v_evaluation,
                                                                x_challenge,
                                                                u_challenge,
                                                                concatenation_groups_commitments);

            // Compute commitment C_{\zeta,Z}
            auto C_zeta_Z = C_zeta_x + C_Z_x * z_challenge;

            // Receive proof commitment \pi
            auto C_pi = verifier_transcript.template receive_from_prover<Commitment>("ZM:PI");

            // The prover and verifier manifests should agree
            EXPECT_EQ(prover_transcript.get_manifest(), verifier_transcript.get_manifest());

            // Construct inputs and perform pairing check to verify claimed evaluation
            // Note: The pairing check (without the degree check component X^{N_max-N-1}) can be expressed naturally as
            // e(C_{\zeta,Z}, [1]_2) = e(pi, [X - x]_2). This can be rearranged (e.g. see the plonk paper) as
            // e(C_{\zeta,Z} - x*pi, [1]_2) * e(-pi, [X]_2) = 1, or
            // e(P_0, [1]_2) * e(P_1, [X]_2) = 1
            auto P0 = C_zeta_Z + C_pi * x_challenge;
            auto P1 = -C_pi;
            verified = this->vk()->pairing_check(P0, P1);
            // EXPECT_TRUE(verified);
        }
        return verified;
    }
};

using CurveTypes = ::testing::Types<curve::BN254>;
TYPED_TEST_SUITE(ZeroMorphTest, CurveTypes);
TYPED_TEST_SUITE(ZeroMorphWithConcatenationTest, CurveTypes);

/**
 * @brief Test method for computing q_k given multilinear f
 * @details Given f = f(X_0, ..., X_{d-1}), and (u,v) such that f(u) = v, compute q_k = q_k(X_0, ..., X_{k-1}) such that
 * the following identity holds:
 *
 *  f(X_0, ..., X_{d-1}) - v = \sum_{k=0}^{d-1} (X_k - u_k)q_k(X_0, ..., X_{k-1})
 *
 */
TYPED_TEST(ZeroMorphTest, QuotientConstruction)
{
    // Define some useful type aliases
    using ZeroMorphProver = ZeroMorphProver_<TypeParam>;
    using Fr = typename TypeParam::ScalarField;
    using Polynomial = barretenberg::Polynomial<Fr>;

    // Define size parameters
    size_t N = 16;
    size_t log_N = numeric::get_msb(N);

    // Construct a random multilinear polynomial f, and (u,v) such that f(u) = v.
    Polynomial multilinear_f = this->random_polynomial(N);
    std::vector<Fr> u_challenge = this->random_evaluation_point(log_N);
    Fr v_evaluation = multilinear_f.evaluate_mle(u_challenge);

    // Compute the multilinear quotients q_k = q_k(X_0, ..., X_{k-1})
    std::vector<Polynomial> quotients = ZeroMorphProver::compute_multilinear_quotients(multilinear_f, u_challenge);

    // Show that the q_k were properly constructed by showing that the identity holds at a random multilinear challenge
    // z, i.e. f(z) - v - \sum_{k=0}^{d-1} (z_k - u_k)q_k(z) = 0
    std::vector<Fr> z_challenge = this->random_evaluation_point(log_N);

    Fr result = multilinear_f.evaluate_mle(z_challenge);
    result -= v_evaluation;
    for (size_t k = 0; k < log_N; ++k) {
        auto q_k_eval = Fr(0);
        if (k == 0) {
            // q_0 = a_0 is a constant polynomial so it's evaluation is simply its constant coefficient
            q_k_eval = quotients[k][0];
        } else {
            // Construct (u_0, ..., u_{k-1})
            auto subrange_size = static_cast<std::ptrdiff_t>(k);
            std::vector<Fr> z_partial(z_challenge.begin(), z_challenge.begin() + subrange_size);
            q_k_eval = quotients[k].evaluate_mle(z_partial);
        }
        // result = result - (z_k - u_k) * q_k(u_0, ..., u_{k-1})
        result -= (z_challenge[k] - u_challenge[k]) * q_k_eval;
    }

    EXPECT_EQ(result, 0);
}

/**
 * @brief Test function for constructing batched lifted degree quotient \hat{q}
 *
 */
TYPED_TEST(ZeroMorphTest, BatchedLiftedDegreeQuotient)
{
    // Define some useful type aliases
    using ZeroMorphProver = ZeroMorphProver_<TypeParam>;
    using Fr = typename TypeParam::ScalarField;
    using Polynomial = barretenberg::Polynomial<Fr>;

    const size_t N = 8;

    // Define some mock q_k with deg(q_k) = 2^k - 1
    std::vector<Fr> data_0 = { 1 };
    std::vector<Fr> data_1 = { 2, 3 };
    std::vector<Fr> data_2 = { 4, 5, 6, 7 };
    Polynomial q_0(data_0);
    Polynomial q_1(data_1);
    Polynomial q_2(data_2);
    std::vector<Polynomial> quotients = { q_0, q_1, q_2 };

    auto y_challenge = Fr::random_element();

    // Compute batched quotient \hat{q} using the prover method
    auto batched_quotient = ZeroMorphProver::compute_batched_lifted_degree_quotient(quotients, y_challenge, N);

    // Now explicitly define q_k_lifted = X^{N-2^k} * q_k and compute the expected batched result
    std::array<Fr, N> data_0_lifted = { 0, 0, 0, 0, 0, 0, 0, 1 };
    std::array<Fr, N> data_1_lifted = { 0, 0, 0, 0, 0, 0, 2, 3 };
    std::array<Fr, N> data_2_lifted = { 0, 0, 0, 0, 4, 5, 6, 7 };
    Polynomial q_0_lifted(data_0_lifted);
    Polynomial q_1_lifted(data_1_lifted);
    Polynomial q_2_lifted(data_2_lifted);

    // Explicitly compute \hat{q}
    auto batched_quotient_expected = Polynomial(N);
    batched_quotient_expected += q_0_lifted;
    batched_quotient_expected.add_scaled(q_1_lifted, y_challenge);
    batched_quotient_expected.add_scaled(q_2_lifted, y_challenge * y_challenge);

    EXPECT_EQ(batched_quotient, batched_quotient_expected);
}

/**
 * @brief Test function for constructing partially evaluated quotient \zeta_x
 *
 */
TYPED_TEST(ZeroMorphTest, PartiallyEvaluatedQuotientZeta)
{
    // Define some useful type aliases
    using ZeroMorphProver = ZeroMorphProver_<TypeParam>;
    using Fr = typename TypeParam::ScalarField;
    using Polynomial = barretenberg::Polynomial<Fr>;

    const size_t N = 8;

    // Define some mock q_k with deg(q_k) = 2^k - 1
    std::vector<Fr> data_0 = { 1 };
    std::vector<Fr> data_1 = { 2, 3 };
    std::vector<Fr> data_2 = { 4, 5, 6, 7 };
    Polynomial q_0(data_0);
    Polynomial q_1(data_1);
    Polynomial q_2(data_2);
    std::vector<Polynomial> quotients = { q_0, q_1, q_2 };

    auto y_challenge = Fr::random_element();

    auto batched_quotient = ZeroMorphProver::compute_batched_lifted_degree_quotient(quotients, y_challenge, N);

    auto x_challenge = Fr::random_element();

    // Contruct zeta_x using the prover method
    auto zeta_x = ZeroMorphProver::compute_partially_evaluated_degree_check_polynomial(
        batched_quotient, quotients, y_challenge, x_challenge);

    // Now construct zeta_x explicitly
    auto zeta_x_expected = Polynomial(N);
    zeta_x_expected += batched_quotient;
    // q_batched - \sum_k q_k * y^k * x^{N - deg(q_k) - 1}
    zeta_x_expected.add_scaled(q_0, -x_challenge.pow(N - 0 - 1));
    zeta_x_expected.add_scaled(q_1, -y_challenge * x_challenge.pow(N - 1 - 1));
    zeta_x_expected.add_scaled(q_2, -y_challenge * y_challenge * x_challenge.pow(N - 3 - 1));

    EXPECT_EQ(zeta_x, zeta_x_expected);
}

/**
 * @brief Demonstrate formulas for efficiently computing \Phi_k(x) = \sum_{i=0}^{k-1}x^i
 * @details \Phi_k(x) = \sum_{i=0}^{k-1}x^i = (x^{2^k} - 1) / (x - 1)
 *
 */
TYPED_TEST(ZeroMorphTest, PhiEvaluation)
{
    using Fr = typename TypeParam::ScalarField;
    const size_t N = 8;
    size_t n = numeric::get_msb(N);

    // \Phi_n(x)
    {
        auto x_challenge = Fr::random_element();

        auto efficient = (x_challenge.pow(1 << n) - 1) / (x_challenge - 1);

        auto expected = this->Phi(x_challenge, n);

        EXPECT_EQ(efficient, expected);
    }

    // \Phi_{n-k-1}(x^{2^{k + 1}}) = (x^{2^n} - 1) / (x^{2^{k + 1}} - 1)
    {
        auto x_challenge = Fr::random_element();

        size_t k = 2;

        // x^{2^{k+1}}
        auto x_pow = x_challenge.pow(1 << (k + 1));

        auto efficient = x_challenge.pow(1 << n) - 1; // x^N - 1
        efficient = efficient / (x_pow - 1);          // (x^N - 1) / (x^{2^{k + 1}} - 1)

        auto expected = this->Phi(x_pow, n - k - 1);
        EXPECT_EQ(efficient, expected);
    }
}

/**
 * @brief Test function for constructing partially evaluated quotient Z_x
 *
 */
TYPED_TEST(ZeroMorphTest, PartiallyEvaluatedQuotientZ)
{
    // Define some useful type aliases
    using ZeroMorphProver = ZeroMorphProver_<TypeParam>;
    using Fr = typename TypeParam::ScalarField;
    using Polynomial = barretenberg::Polynomial<Fr>;

    const size_t N = 8;
    size_t log_N = numeric::get_msb(N);

    // Construct a random multilinear polynomial f, and (u,v) such that f(u) = v.
    Polynomial multilinear_f = this->random_polynomial(N);
    Polynomial multilinear_g = this->random_polynomial(N);
    multilinear_g[0] = 0;
    std::vector<Fr> u_challenge = this->random_evaluation_point(log_N);
    Fr v_evaluation = multilinear_f.evaluate_mle(u_challenge);
    Fr w_evaluation = multilinear_g.evaluate_mle(u_challenge, /* shift = */ true);

    auto rho = Fr::random_element();

    // compute batched polynomial and evaluation
    auto f_batched = multilinear_f;
    auto g_batched = multilinear_g;
    g_batched *= rho;
    auto v_batched = v_evaluation + rho * w_evaluation;

    // Define some mock q_k with deg(q_k) = 2^k - 1
    auto q_0 = this->random_polynomial(1 << 0);
    auto q_1 = this->random_polynomial(1 << 1);
    auto q_2 = this->random_polynomial(1 << 2);
    std::vector<Polynomial> quotients = { q_0, q_1, q_2 };

    auto x_challenge = Fr::random_element();

    // Construct Z_x using the prover method
    auto Z_x = ZeroMorphProver::compute_partially_evaluated_zeromorph_identity_polynomial(
        f_batched, g_batched, quotients, v_batched, u_challenge, x_challenge);

    // Compute Z_x directly
    auto Z_x_expected = g_batched;
    Z_x_expected.add_scaled(f_batched, x_challenge);
    Z_x_expected[0] -= v_batched * x_challenge * this->Phi(x_challenge, log_N);
    for (size_t k = 0; k < log_N; ++k) {
        auto x_pow_2k = x_challenge.pow(1 << k);         // x^{2^k}
        auto x_pow_2kp1 = x_challenge.pow(1 << (k + 1)); // x^{2^{k+1}}
        // x^{2^k} * \Phi_{n-k-1}(x^{2^{k+1}}) - u_k *  \Phi_{n-k}(x^{2^k})
        auto scalar = x_pow_2k * this->Phi(x_pow_2kp1, log_N - k - 1) - u_challenge[k] * this->Phi(x_pow_2k, log_N - k);
        scalar *= x_challenge;
        scalar *= Fr(-1);
        Z_x_expected.add_scaled(quotients[k], scalar);
    }

    EXPECT_EQ(Z_x, Z_x_expected);
}

/**
 * @brief Test full Prover/Verifier protocol for proving single multilinear evaluation
 *
 */
TYPED_TEST(ZeroMorphTest, ProveAndVerifySingle)
{
    size_t num_unshifted = 1;
    size_t num_shifted = 0;
    auto verified = this->execute_zeromorph_protocol(num_unshifted, num_shifted);
    EXPECT_TRUE(verified);
}

/**
 * @brief Test full Prover/Verifier protocol for proving batched multilinear evaluation with shifts
 *
 */
TYPED_TEST(ZeroMorphTest, ProveAndVerifyBatchedWithShifts)
{
    size_t num_unshifted = 3;
    size_t num_shifted = 2;
    auto verified = this->execute_zeromorph_protocol(num_unshifted, num_shifted);
    EXPECT_TRUE(verified);
}

/**
 * @brief Test full Prover/Verifier protocol for proving single multilinear evaluation
 *
 */
TYPED_TEST(ZeroMorphWithConcatenationTest, ProveAndVerify)
{
    size_t num_unshifted = 1;
    size_t num_shifted = 0;
    size_t num_concatenated = 3;
    auto verified = this->execute_zeromorph_protocol(num_unshifted, num_shifted, num_concatenated);
    EXPECT_TRUE(verified);
}
} // namespace proof_system::honk::pcs::zeromorph
