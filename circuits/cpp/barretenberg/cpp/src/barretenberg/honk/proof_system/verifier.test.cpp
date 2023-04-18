#include "barretenberg/numeric/bitop/get_msb.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/honk/flavor/flavor.hpp"
#include "prover.hpp"
#include "barretenberg/plonk/proof_system/proving_key/proving_key.hpp"
#include "barretenberg/transcript/transcript.hpp"
#include "verifier.hpp"
#include "barretenberg/ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp"
#include <gtest/gtest.h>
#include "barretenberg/srs/reference_string/file_reference_string.hpp"
#include "barretenberg/polynomials/polynomial_arithmetic.hpp"
#include "barretenberg/plonk/proof_system/commitment_scheme/kate_commitment_scheme.hpp"
#include "barretenberg/proof_system/composer/permutation_helper.hpp"
#include <vector>

using namespace barretenberg;
using namespace proof_system::honk;

namespace test_honk_verifier {

template <class FF> class VerifierTests : public testing::Test {
  public:
    static transcript::Manifest create_manifest(const size_t num_public_inputs, const size_t num_sumcheck_rounds)
    {
        return honk::StandardHonk::create_manifest(num_public_inputs, num_sumcheck_rounds);
    }

    static StandardVerifier generate_verifier(std::shared_ptr<plonk::proving_key> circuit_proving_key)
    {
        std::array<fr*, 8> poly_coefficients;
        poly_coefficients[0] = circuit_proving_key->polynomial_store.get("q_1_lagrange").get_coefficients();
        poly_coefficients[1] = circuit_proving_key->polynomial_store.get("q_2_lagrange").get_coefficients();
        poly_coefficients[2] = circuit_proving_key->polynomial_store.get("q_3_lagrange").get_coefficients();
        poly_coefficients[3] = circuit_proving_key->polynomial_store.get("q_m_lagrange").get_coefficients();
        poly_coefficients[4] = circuit_proving_key->polynomial_store.get("q_c_lagrange").get_coefficients();
        poly_coefficients[5] = circuit_proving_key->polynomial_store.get("sigma_1_lagrange").get_coefficients();
        poly_coefficients[6] = circuit_proving_key->polynomial_store.get("sigma_2_lagrange").get_coefficients();
        poly_coefficients[7] = circuit_proving_key->polynomial_store.get("sigma_3_lagrange").get_coefficients();

        std::vector<barretenberg::g1::affine_element> commitments;
        scalar_multiplication::pippenger_runtime_state prover(circuit_proving_key->circuit_size);
        commitments.resize(8);

        for (size_t i = 0; i < 8; ++i) {
            commitments[i] = g1::affine_element(
                scalar_multiplication::pippenger(poly_coefficients[i],
                                                 circuit_proving_key->reference_string->get_monomial_points(),
                                                 circuit_proving_key->circuit_size,
                                                 prover));
        }

        auto crs = std::make_shared<VerifierFileReferenceString>("../srs_db/ignition");
        auto circuit_verification_key =
            std::make_shared<plonk::verification_key>(circuit_proving_key->circuit_size,
                                                      circuit_proving_key->num_public_inputs,
                                                      crs,
                                                      circuit_proving_key->composer_type);

        circuit_verification_key->commitments.insert({ "Q_1", commitments[0] });
        circuit_verification_key->commitments.insert({ "Q_2", commitments[1] });
        circuit_verification_key->commitments.insert({ "Q_3", commitments[2] });
        circuit_verification_key->commitments.insert({ "Q_M", commitments[3] });
        circuit_verification_key->commitments.insert({ "Q_C", commitments[4] });

        circuit_verification_key->commitments.insert({ "SIGMA_1", commitments[5] });
        circuit_verification_key->commitments.insert({ "SIGMA_2", commitments[6] });
        circuit_verification_key->commitments.insert({ "SIGMA_3", commitments[7] });

        StandardVerifier verifier(circuit_verification_key);

        // std::unique_ptr<KateCommitmentScheme<standard_settings>> kate_commitment_scheme =
        //     std::make_unique<KateCommitmentScheme<standard_settings>>();
        // verifier.commitment_scheme = std::move(kate_commitment_scheme);

        return verifier;
    }

    // Note: this example is adapted from a corresponding PlonK verifier test.
    static StandardProver generate_test_data(const size_t n)
    {
        // Create some constraints that satisfy our arithmetic circuit relation
        // even indices = mul gates, odd incides = add gates

        auto crs = std::make_shared<FileReferenceString>(n + 1, "../srs_db/ignition");
        std::shared_ptr<plonk::proving_key> proving_key =
            std::make_shared<plonk::proving_key>(n, 0, crs, ComposerType::STANDARD_HONK);

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
            q_l.at(2 * i) = fr::one();
            q_r.at(2 * i) = fr::one();
            q_o.at(2 * i) = fr::neg_one();
            q_c.at(2 * i) = fr::one();
            q_m.at(2 * i) = fr::one();

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

        polynomial sigma_1(proving_key->circuit_size);
        polynomial sigma_2(proving_key->circuit_size);
        polynomial sigma_3(proving_key->circuit_size);

        // plonk::compute_permutation_lagrange_base_single<standard_settings>(sigma_1, sigma_1_mapping,
        // proving_key->small_domain); plonk::compute_permutation_lagrange_base_single<standard_settings>(sigma_2,
        // sigma_2_mapping, proving_key->small_domain);
        // plonk::compute_permutation_lagrange_base_single<standard_settings>(sigma_3, sigma_3_mapping,
        // proving_key->small_domain);

        polynomial sigma_1_lagrange_base(sigma_1, proving_key->circuit_size);
        polynomial sigma_2_lagrange_base(sigma_2, proving_key->circuit_size);
        polynomial sigma_3_lagrange_base(sigma_3, proving_key->circuit_size);

        proving_key->polynomial_store.put("sigma_1_lagrange", std::move(sigma_1_lagrange_base));
        proving_key->polynomial_store.put("sigma_2_lagrange", std::move(sigma_2_lagrange_base));
        proving_key->polynomial_store.put("sigma_3_lagrange", std::move(sigma_3_lagrange_base));

        compute_standard_honk_id_polynomials<3>(proving_key);
        compute_first_and_last_lagrange_polynomials(proving_key);

        proving_key->polynomial_store.put("w_1_lagrange", std::move(w_l));
        proving_key->polynomial_store.put("w_2_lagrange", std::move(w_r));
        proving_key->polynomial_store.put("w_3_lagrange", std::move(w_o));

        proving_key->polynomial_store.put("q_1_lagrange", std::move(q_l));
        proving_key->polynomial_store.put("q_2_lagrange", std::move(q_r));
        proving_key->polynomial_store.put("q_3_lagrange", std::move(q_o));
        proving_key->polynomial_store.put("q_m_lagrange", std::move(q_m));
        proving_key->polynomial_store.put("q_c_lagrange", std::move(q_c));

        // TODO(#223)(Cody): This should be more generic
        std::vector<barretenberg::polynomial> witness_polynomials;
        auto prover = StandardProver(std::move(witness_polynomials), proving_key);

        std::unique_ptr<pcs::kzg::CommitmentKey> kate_commitment_key =
            std::make_unique<pcs::kzg::CommitmentKey>(proving_key->circuit_size, "../srs_db/ignition");

        return prover;
    }
};

using FieldTypes = testing::Types<barretenberg::fr>;
TYPED_TEST_SUITE(VerifierTests, FieldTypes);

// This test is modeled after a corresponding test for the Plonk Verifier. As is the case there, this test relies on
// valid proof construction which makes the scope quite large. Not really a unit test but a nice test nonetheless.
// TODO(#223)(Luke/Cody): Make this a meaningful test (or remove altogether)
TYPED_TEST(VerifierTests, VerifyArithmeticProofSmall)
{
    GTEST_SKIP() << "It's good to have a standalone test, but for now we just rely on composer tests.";
    size_t n = 8;

    StandardProver prover = TestFixture::generate_test_data(n);

    StandardVerifier verifier = TestFixture::generate_verifier(prover.key);

    // construct proof
    plonk::proof proof = prover.construct_proof();

    // verify proof
    bool result = verifier.verify_proof(proof);

    EXPECT_EQ(result, true);
}

} // namespace test_honk_verifier
