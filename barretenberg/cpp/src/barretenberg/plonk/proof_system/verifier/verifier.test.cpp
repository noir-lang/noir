#include "../prover/prover.hpp"
#include "../utils/permutation.hpp"
#include "../widgets/transition_widgets/arithmetic_widget.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/scalar_multiplication/scalar_multiplication.hpp"
#include "barretenberg/plonk/composer/standard_composer.hpp"
#include "barretenberg/plonk/proof_system/commitment_scheme/kate_commitment_scheme.hpp"
#include "barretenberg/plonk/proof_system/proving_key/proving_key.hpp"
#include "barretenberg/plonk/transcript/transcript.hpp"
#include "barretenberg/polynomials/polynomial_arithmetic.hpp"
#include "barretenberg/srs/factories/file_crs_factory.hpp"
#include <gtest/gtest.h>

namespace verifier_helpers {

using namespace bb;
using namespace bb::plonk;

plonk::Verifier generate_verifier(std::shared_ptr<proving_key> circuit_proving_key)
{
    std::array<std::shared_ptr<fr[]>, 8> poly_coefficients;
    poly_coefficients[0] = circuit_proving_key->polynomial_store.get("q_1").data();
    poly_coefficients[1] = circuit_proving_key->polynomial_store.get("q_2").data();
    poly_coefficients[2] = circuit_proving_key->polynomial_store.get("q_3").data();
    poly_coefficients[3] = circuit_proving_key->polynomial_store.get("q_m").data();
    poly_coefficients[4] = circuit_proving_key->polynomial_store.get("q_c").data();
    poly_coefficients[5] = circuit_proving_key->polynomial_store.get("sigma_1").data();
    poly_coefficients[6] = circuit_proving_key->polynomial_store.get("sigma_2").data();
    poly_coefficients[7] = circuit_proving_key->polynomial_store.get("sigma_3").data();

    std::vector<g1::affine_element> commitments;
    scalar_multiplication::pippenger_runtime_state<curve::BN254> state(circuit_proving_key->circuit_size);
    commitments.resize(8);

    for (size_t i = 0; i < 8; ++i) {
        commitments[i] = g1::affine_element(
            scalar_multiplication::pippenger<curve::BN254>(poly_coefficients[i].get(),
                                                           circuit_proving_key->reference_string->get_monomial_points(),
                                                           circuit_proving_key->circuit_size,
                                                           state));
    }

    auto crs = std::make_shared<bb::srs::factories::FileVerifierCrs<curve::BN254>>("../srs_db/ignition");
    std::shared_ptr<verification_key> circuit_verification_key =
        std::make_shared<verification_key>(circuit_proving_key->circuit_size,
                                           circuit_proving_key->num_public_inputs,
                                           crs,
                                           circuit_proving_key->circuit_type);

    circuit_verification_key->commitments.insert({ "Q_1", commitments[0] });
    circuit_verification_key->commitments.insert({ "Q_2", commitments[1] });
    circuit_verification_key->commitments.insert({ "Q_3", commitments[2] });
    circuit_verification_key->commitments.insert({ "Q_M", commitments[3] });
    circuit_verification_key->commitments.insert({ "Q_C", commitments[4] });

    circuit_verification_key->commitments.insert({ "SIGMA_1", commitments[5] });
    circuit_verification_key->commitments.insert({ "SIGMA_2", commitments[6] });
    circuit_verification_key->commitments.insert({ "SIGMA_3", commitments[7] });

    Verifier verifier(circuit_verification_key, plonk::StandardComposer::create_manifest(0));

    std::unique_ptr<KateCommitmentScheme<standard_settings>> kate_commitment_scheme =
        std::make_unique<KateCommitmentScheme<standard_settings>>();
    verifier.commitment_scheme = std::move(kate_commitment_scheme);

    // std::unique_ptr<plonk::VerifierArithmeticWidget> widget = std::make_unique<plonk::VerifierArithmeticWidget>();
    // verifier.verifier_widgets.emplace_back(std::move(widget));
    return verifier;
}

plonk::Prover generate_test_data(const size_t n)
{
    // state.random_widgets.emplace_back(std::make_unique<plonk::ProverArithmeticWidget>(n));

    // create some constraints that satisfy our arithmetic circuit relation

    // even indices = mul gates, odd incides = add gates

    auto crs = std::make_shared<bb::srs::factories::FileProverCrs<curve::BN254>>(n + 1, "../srs_db/ignition");
    std::shared_ptr<proving_key> key = std::make_shared<proving_key>(n, 0, crs, CircuitType::STANDARD);

    polynomial w_l(n);
    polynomial w_r(n);
    polynomial w_o(n);
    polynomial q_l(n);
    polynomial q_r(n);
    polynomial q_o(n);
    polynomial q_c(n);
    polynomial q_m(n);

    fr T0;
    for (size_t i = 0; i < n / 4; ++i) {
        w_l.at(2 * i) = fr::random_element();
        w_r.at(2 * i) = fr::random_element();
        w_o.at(2 * i) = w_l.at(2 * i) * w_r.at(2 * i);
        w_o[2 * i] = w_o[2 * i] + w_l[2 * i];
        w_o[2 * i] = w_o[2 * i] + w_r[2 * i];
        w_o[2 * i] = w_o[2 * i] + fr::one();
        fr::__copy(fr::one(), q_l.at(2 * i));
        fr::__copy(fr::one(), q_r.at(2 * i));
        fr::__copy(fr::neg_one(), q_o.at(2 * i));
        fr::__copy(fr::one(), q_c.at(2 * i));
        fr::__copy(fr::one(), q_m.at(2 * i));

        w_l.at(2 * i + 1) = fr::random_element();
        w_r.at(2 * i + 1) = fr::random_element();
        w_o.at(2 * i + 1) = fr::random_element();

        T0 = w_l.at(2 * i + 1) + w_r.at(2 * i + 1);
        q_c.at(2 * i + 1) = T0 + w_o.at(2 * i + 1);
        q_c.at(2 * i + 1).self_neg();
        q_l.at(2 * i + 1) = fr::one();
        q_r.at(2 * i + 1) = fr::one();
        q_o.at(2 * i + 1) = fr::one();
        q_m.at(2 * i + 1) = fr::zero();
    }
    size_t shift = n / 2;
    polynomial_arithmetic::copy_polynomial(&w_l.at(0), &w_l.at(shift), shift, shift);
    polynomial_arithmetic::copy_polynomial(&w_r.at(0), &w_r.at(shift), shift, shift);
    polynomial_arithmetic::copy_polynomial(&w_o.at(0), &w_o.at(shift), shift, shift);
    polynomial_arithmetic::copy_polynomial(&q_m.at(0), &q_m.at(shift), shift, shift);
    polynomial_arithmetic::copy_polynomial(&q_l.at(0), &q_l.at(shift), shift, shift);
    polynomial_arithmetic::copy_polynomial(&q_r.at(0), &q_r.at(shift), shift, shift);
    polynomial_arithmetic::copy_polynomial(&q_o.at(0), &q_o.at(shift), shift, shift);
    polynomial_arithmetic::copy_polynomial(&q_c.at(0), &q_c.at(shift), shift, shift);

    std::vector<uint32_t> sigma_1_mapping;
    std::vector<uint32_t> sigma_2_mapping;
    std::vector<uint32_t> sigma_3_mapping;
    // create basic permutation - second half of witness vector is a copy of the first half
    sigma_1_mapping.resize(n);
    sigma_2_mapping.resize(n);
    sigma_3_mapping.resize(n);

    for (size_t i = 0; i < n / 2; ++i) {
        sigma_1_mapping[shift + i] = (uint32_t)i;
        sigma_2_mapping[shift + i] = (uint32_t)i + (1U << 30U);
        sigma_3_mapping[shift + i] = (uint32_t)i + (1U << 31U);
        sigma_1_mapping[i] = (uint32_t)(i + shift);
        sigma_2_mapping[i] = (uint32_t)(i + shift) + (1U << 30U);
        sigma_3_mapping[i] = (uint32_t)(i + shift) + (1U << 31U);
    }
    // make last permutation the same as identity permutation
    // we are setting the permutation in the last 4 gates as identity permutation since
    // we are cutting out 4 roots as of now.
    size_t num_roots_cut_out_of_the_vanishing_polynomial = 4;
    for (uint32_t j = 0; j < num_roots_cut_out_of_the_vanishing_polynomial; ++j) {
        sigma_1_mapping[shift - 1 - j] = (uint32_t)shift - 1 - j;
        sigma_2_mapping[shift - 1 - j] = (uint32_t)shift - 1 - j + (1U << 30U);
        sigma_3_mapping[shift - 1 - j] = (uint32_t)shift - 1 - j + (1U << 31U);
        sigma_1_mapping[n - 1 - j] = (uint32_t)n - 1 - j;
        sigma_2_mapping[n - 1 - j] = (uint32_t)n - 1 - j + (1U << 30U);
        sigma_3_mapping[n - 1 - j] = (uint32_t)n - 1 - j + (1U << 31U);
    }

    polynomial sigma_1(key->circuit_size);
    polynomial sigma_2(key->circuit_size);
    polynomial sigma_3(key->circuit_size);

    plonk::compute_permutation_lagrange_base_single<standard_settings>(sigma_1, sigma_1_mapping, key->small_domain);
    plonk::compute_permutation_lagrange_base_single<standard_settings>(sigma_2, sigma_2_mapping, key->small_domain);
    plonk::compute_permutation_lagrange_base_single<standard_settings>(sigma_3, sigma_3_mapping, key->small_domain);

    polynomial sigma_1_lagrange_base(sigma_1, key->circuit_size);
    polynomial sigma_2_lagrange_base(sigma_2, key->circuit_size);
    polynomial sigma_3_lagrange_base(sigma_3, key->circuit_size);

    key->polynomial_store.put("sigma_1_lagrange", std::move(sigma_1_lagrange_base));
    key->polynomial_store.put("sigma_2_lagrange", std::move(sigma_2_lagrange_base));
    key->polynomial_store.put("sigma_3_lagrange", std::move(sigma_3_lagrange_base));

    sigma_1.ifft(key->small_domain);
    sigma_2.ifft(key->small_domain);
    sigma_3.ifft(key->small_domain);
    constexpr size_t width = 4;
    polynomial sigma_1_fft(sigma_1, key->circuit_size * width);
    polynomial sigma_2_fft(sigma_2, key->circuit_size * width);
    polynomial sigma_3_fft(sigma_3, key->circuit_size * width);

    sigma_1_fft.coset_fft(key->large_domain);
    sigma_2_fft.coset_fft(key->large_domain);
    sigma_3_fft.coset_fft(key->large_domain);

    key->polynomial_store.put("sigma_1", std::move(sigma_1));
    key->polynomial_store.put("sigma_2", std::move(sigma_2));
    key->polynomial_store.put("sigma_3", std::move(sigma_3));

    key->polynomial_store.put("sigma_1_fft", std::move(sigma_1_fft));
    key->polynomial_store.put("sigma_2_fft", std::move(sigma_2_fft));
    key->polynomial_store.put("sigma_3_fft", std::move(sigma_3_fft));

    key->polynomial_store.put("w_1_lagrange", std::move(w_l));
    key->polynomial_store.put("w_2_lagrange", std::move(w_r));
    key->polynomial_store.put("w_3_lagrange", std::move(w_o));

    q_l.ifft(key->small_domain);
    q_r.ifft(key->small_domain);
    q_o.ifft(key->small_domain);
    q_m.ifft(key->small_domain);
    q_c.ifft(key->small_domain);

    polynomial q_1_fft(q_l, n * 4);
    polynomial q_2_fft(q_r, n * 4);
    polynomial q_3_fft(q_o, n * 4);
    polynomial q_m_fft(q_m, n * 4);
    polynomial q_c_fft(q_c, n * 4);

    q_1_fft.coset_fft(key->large_domain);
    q_2_fft.coset_fft(key->large_domain);
    q_3_fft.coset_fft(key->large_domain);
    q_m_fft.coset_fft(key->large_domain);
    q_c_fft.coset_fft(key->large_domain);

    key->polynomial_store.put("q_1", std::move(q_l));
    key->polynomial_store.put("q_2", std::move(q_r));
    key->polynomial_store.put("q_3", std::move(q_o));
    key->polynomial_store.put("q_m", std::move(q_m));
    key->polynomial_store.put("q_c", std::move(q_c));

    key->polynomial_store.put("q_1_fft", std::move(q_1_fft));
    key->polynomial_store.put("q_2_fft", std::move(q_2_fft));
    key->polynomial_store.put("q_3_fft", std::move(q_3_fft));
    key->polynomial_store.put("q_m_fft", std::move(q_m_fft));
    key->polynomial_store.put("q_c_fft", std::move(q_c_fft));

    std::unique_ptr<plonk::ProverPermutationWidget<3>> permutation_widget =
        std::make_unique<plonk::ProverPermutationWidget<3>>(key.get());

    std::unique_ptr<plonk::ProverArithmeticWidget<plonk::standard_settings>> widget =
        std::make_unique<plonk::ProverArithmeticWidget<plonk::standard_settings>>(key.get());

    std::unique_ptr<KateCommitmentScheme<standard_settings>> kate_commitment_scheme =
        std::make_unique<KateCommitmentScheme<standard_settings>>();

    plonk::Prover state = plonk::Prover(std::move(key), plonk::StandardComposer::create_manifest(0));
    state.random_widgets.emplace_back(std::move(permutation_widget));
    state.transition_widgets.emplace_back(std::move(widget));
    state.commitment_scheme = std::move(kate_commitment_scheme);
    return state;
}
} // namespace verifier_helpers

TEST(verifier, verify_arithmetic_proof_small)
{
    size_t n = 8;

    plonk::Prover state = verifier_helpers::generate_test_data(n);

    auto verifier = verifier_helpers::generate_verifier(state.key);

    // construct proof
    plonk::proof proof = state.construct_proof();

    // verify proof
    bool result = verifier.verify_proof(proof);

    EXPECT_EQ(result, true);
}

// TODO(Cody): This test is suddenly very slow to run?
TEST(verifier, verify_arithmetic_proof)
{
    size_t n = 1 << 14;

    plonk::Prover state = verifier_helpers::generate_test_data(n);

    auto verifier = verifier_helpers::generate_verifier(state.key);

    // construct proof
    plonk::proof proof = state.construct_proof();

    // verify proof
    bool result = verifier.verify_proof(proof);

    EXPECT_EQ(result, true);
}

#if !defined(__wasm__)
TEST(verifier, verify_damaged_proof)
{
    size_t n = 8;

    plonk::Prover state = verifier_helpers::generate_test_data(n);

    auto verifier = verifier_helpers::generate_verifier(state.key);

    // Create empty proof
    plonk::proof proof = {};

    // verify proof
    EXPECT_ANY_THROW(verifier.verify_proof(proof));
}
#endif
