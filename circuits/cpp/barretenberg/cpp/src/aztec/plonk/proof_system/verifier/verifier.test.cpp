#include "../prover/prover.hpp"
#include "../proving_key/proving_key.hpp"
#include "../utils/linearizer.hpp"
#include "../utils/permutation.hpp"
#include "../widgets/transition_widgets/arithmetic_widget.hpp"
#include "../../transcript/transcript.hpp"
#include "verifier.hpp"
#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include <gtest/gtest.h>
#include <srs/reference_string/file_reference_string.hpp>
#include <polynomials/polynomial_arithmetic.hpp>
#include <plonk/proof_system/types/polynomial_manifest.hpp>
#include <plonk/proof_system/commitment_scheme/kate_commitment_scheme.hpp>

namespace verifier_helpers {

transcript::Manifest create_manifest(const size_t num_public_inputs = 0)
{
    constexpr size_t g1_size = 64;
    constexpr size_t fr_size = 32;
    const size_t public_input_size = fr_size * num_public_inputs;
    const transcript::Manifest output = transcript::Manifest(
        { transcript::Manifest::RoundManifest(
              { { "circuit_size", 4, true }, { "public_input_size", 4, true } }, "init", 1),
          transcript::Manifest::RoundManifest({}, "eta", 0),
          transcript::Manifest::RoundManifest({ { "public_inputs", public_input_size, false },
                                                { "W_1", g1_size, false },
                                                { "W_2", g1_size, false },
                                                { "W_3", g1_size, false } },
                                              "beta",
                                              2),
          transcript::Manifest::RoundManifest({ { "Z_PERM", g1_size, false } }, "alpha", 1),
          transcript::Manifest::RoundManifest(
              { { "T_1", g1_size, false }, { "T_2", g1_size, false }, { "T_3", g1_size, false } }, "z", 1),
          transcript::Manifest::RoundManifest(
              {
                  { "w_1", fr_size, false, 0 },
                  { "w_2", fr_size, false, 1 },
                  { "w_3", fr_size, false, 2 },
                  { "sigma_1", fr_size, false, 3 },
                  { "sigma_2", fr_size, false, 4 },
                  { "z_perm_omega", fr_size, false, -1 },
              },
              "nu",
              7,
              true),
          transcript::Manifest::RoundManifest(
              { { "PI_Z", g1_size, false }, { "PI_Z_OMEGA", g1_size, false } }, "separator", 1) });
    return output;
}

using namespace barretenberg;
using namespace waffle;

waffle::Verifier generate_verifier(std::shared_ptr<proving_key> circuit_proving_key)
{
    std::array<fr*, 8> poly_coefficients;
    poly_coefficients[0] = circuit_proving_key->polynomial_cache.get("q_1").get_coefficients();
    poly_coefficients[1] = circuit_proving_key->polynomial_cache.get("q_2").get_coefficients();
    poly_coefficients[2] = circuit_proving_key->polynomial_cache.get("q_3").get_coefficients();
    poly_coefficients[3] = circuit_proving_key->polynomial_cache.get("q_m").get_coefficients();
    poly_coefficients[4] = circuit_proving_key->polynomial_cache.get("q_c").get_coefficients();
    poly_coefficients[5] = circuit_proving_key->polynomial_cache.get("sigma_1").get_coefficients();
    poly_coefficients[6] = circuit_proving_key->polynomial_cache.get("sigma_2").get_coefficients();
    poly_coefficients[7] = circuit_proving_key->polynomial_cache.get("sigma_3").get_coefficients();

    std::vector<barretenberg::g1::affine_element> commitments;
    scalar_multiplication::pippenger_runtime_state state(circuit_proving_key->n);
    commitments.resize(8);

    for (size_t i = 0; i < 8; ++i) {
        commitments[i] =
            g1::affine_element(scalar_multiplication::pippenger(poly_coefficients[i],
                                                                circuit_proving_key->reference_string->get_monomials(),
                                                                circuit_proving_key->n,
                                                                state));
    }

    auto crs = std::make_shared<waffle::VerifierFileReferenceString>("../srs_db/ignition");
    std::shared_ptr<verification_key> circuit_verification_key = std::make_shared<verification_key>(
        circuit_proving_key->n, circuit_proving_key->num_public_inputs, crs, circuit_proving_key->composer_type);

    circuit_verification_key->constraint_selectors.insert({ "Q_1", commitments[0] });
    circuit_verification_key->constraint_selectors.insert({ "Q_2", commitments[1] });
    circuit_verification_key->constraint_selectors.insert({ "Q_3", commitments[2] });
    circuit_verification_key->constraint_selectors.insert({ "Q_M", commitments[3] });
    circuit_verification_key->constraint_selectors.insert({ "Q_C", commitments[4] });

    circuit_verification_key->permutation_selectors.insert({ "SIGMA_1", commitments[5] });
    circuit_verification_key->permutation_selectors.insert({ "SIGMA_2", commitments[6] });
    circuit_verification_key->permutation_selectors.insert({ "SIGMA_3", commitments[7] });

    Verifier verifier(circuit_verification_key, create_manifest());

    std::unique_ptr<KateCommitmentScheme<standard_settings>> kate_commitment_scheme =
        std::make_unique<KateCommitmentScheme<standard_settings>>();
    verifier.commitment_scheme = std::move(kate_commitment_scheme);

    // std::unique_ptr<waffle::VerifierArithmeticWidget> widget = std::make_unique<waffle::VerifierArithmeticWidget>();
    // verifier.verifier_widgets.emplace_back(std::move(widget));
    return verifier;
}

waffle::Prover generate_test_data(const size_t n)
{
    // state.random_widgets.emplace_back(std::make_unique<waffle::ProverArithmeticWidget>(n));

    // create some constraints that satisfy our arithmetic circuit relation

    // even indices = mul gates, odd incides = add gates

    auto crs = std::make_shared<waffle::FileReferenceString>(n + 1, "../srs_db/ignition");
    std::shared_ptr<proving_key> key = std::make_shared<proving_key>(n, 0, crs, waffle::STANDARD);

    polynomial w_l;
    polynomial w_r;
    polynomial w_o;
    polynomial q_l;
    polynomial q_r;
    polynomial q_o;
    polynomial q_c;
    polynomial q_m;

    w_l.resize(n);
    w_r.resize(n);
    w_o.resize(n);
    q_l.resize(n);
    q_r.resize(n);
    q_o.resize(n);
    q_m.resize(n);
    q_c.resize(n);
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

    polynomial sigma_1(key->n);
    polynomial sigma_2(key->n);
    polynomial sigma_3(key->n);

    waffle::compute_permutation_lagrange_base_single<standard_settings>(sigma_1, sigma_1_mapping, key->small_domain);
    waffle::compute_permutation_lagrange_base_single<standard_settings>(sigma_2, sigma_2_mapping, key->small_domain);
    waffle::compute_permutation_lagrange_base_single<standard_settings>(sigma_3, sigma_3_mapping, key->small_domain);

    polynomial sigma_1_lagrange_base(sigma_1, key->n);
    polynomial sigma_2_lagrange_base(sigma_2, key->n);
    polynomial sigma_3_lagrange_base(sigma_3, key->n);

    key->polynomial_cache.put("sigma_1_lagrange", std::move(sigma_1_lagrange_base));
    key->polynomial_cache.put("sigma_2_lagrange", std::move(sigma_2_lagrange_base));
    key->polynomial_cache.put("sigma_3_lagrange", std::move(sigma_3_lagrange_base));

    sigma_1.ifft(key->small_domain);
    sigma_2.ifft(key->small_domain);
    sigma_3.ifft(key->small_domain);
    constexpr size_t width = 4;
    polynomial sigma_1_fft(sigma_1, key->n * width);
    polynomial sigma_2_fft(sigma_2, key->n * width);
    polynomial sigma_3_fft(sigma_3, key->n * width);

    sigma_1_fft.coset_fft(key->large_domain);
    sigma_2_fft.coset_fft(key->large_domain);
    sigma_3_fft.coset_fft(key->large_domain);

    key->polynomial_cache.put("sigma_1", std::move(sigma_1));
    key->polynomial_cache.put("sigma_2", std::move(sigma_2));
    key->polynomial_cache.put("sigma_3", std::move(sigma_3));

    key->polynomial_cache.put("sigma_1_fft", std::move(sigma_1_fft));
    key->polynomial_cache.put("sigma_2_fft", std::move(sigma_2_fft));
    key->polynomial_cache.put("sigma_3_fft", std::move(sigma_3_fft));

    key->polynomial_cache.put("w_1_lagrange", std::move(w_l));
    key->polynomial_cache.put("w_2_lagrange", std::move(w_r));
    key->polynomial_cache.put("w_3_lagrange", std::move(w_o));

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

    key->polynomial_cache.put("q_1", std::move(q_l));
    key->polynomial_cache.put("q_2", std::move(q_r));
    key->polynomial_cache.put("q_3", std::move(q_o));
    key->polynomial_cache.put("q_m", std::move(q_m));
    key->polynomial_cache.put("q_c", std::move(q_c));

    key->polynomial_cache.put("q_1_fft", std::move(q_1_fft));
    key->polynomial_cache.put("q_2_fft", std::move(q_2_fft));
    key->polynomial_cache.put("q_3_fft", std::move(q_3_fft));
    key->polynomial_cache.put("q_m_fft", std::move(q_m_fft));
    key->polynomial_cache.put("q_c_fft", std::move(q_c_fft));

    std::unique_ptr<waffle::ProverPermutationWidget<3>> permutation_widget =
        std::make_unique<waffle::ProverPermutationWidget<3>>(key.get());

    std::unique_ptr<waffle::ProverArithmeticWidget<waffle::standard_settings>> widget =
        std::make_unique<waffle::ProverArithmeticWidget<waffle::standard_settings>>(key.get());

    std::unique_ptr<KateCommitmentScheme<standard_settings>> kate_commitment_scheme =
        std::make_unique<KateCommitmentScheme<standard_settings>>();

    waffle::Prover state = waffle::Prover(std::move(key), create_manifest());
    state.random_widgets.emplace_back(std::move(permutation_widget));
    state.transition_widgets.emplace_back(std::move(widget));
    state.commitment_scheme = std::move(kate_commitment_scheme);
    return state;
}
} // namespace verifier_helpers

TEST(verifier, verify_arithmetic_proof_small)
{
    size_t n = 8;

    waffle::Prover state = verifier_helpers::generate_test_data(n);

    waffle::Verifier verifier = verifier_helpers::generate_verifier(state.key);

    // construct proof
    waffle::plonk_proof proof = state.construct_proof();

    // verify proof
    bool result = verifier.verify_proof(proof);

    EXPECT_EQ(result, true);
}

TEST(verifier, verify_arithmetic_proof)
{
    size_t n = 1 << 14;

    waffle::Prover state = verifier_helpers::generate_test_data(n);

    waffle::Verifier verifier = verifier_helpers::generate_verifier(state.key);

    // construct proof
    waffle::plonk_proof proof = state.construct_proof();

    // verify proof
    bool result = verifier.verify_proof(proof);

    EXPECT_EQ(result, true);
}

TEST(verifier, verify_damaged_proof)
{
    size_t n = 8;

    waffle::Prover state = verifier_helpers::generate_test_data(n);

    waffle::Verifier verifier = verifier_helpers::generate_verifier(state.key);

    // Create empty proof
    waffle::plonk_proof proof = {};

    // verify proof
    EXPECT_ANY_THROW(verifier.verify_proof(proof));
}