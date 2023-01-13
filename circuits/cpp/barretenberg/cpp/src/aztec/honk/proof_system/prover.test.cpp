#include "../../srs/reference_string/file_reference_string.hpp"
#include "./prover.hpp"

#include <array>
#include <vector>
#include <cstddef>
#include <gtest/gtest.h>

using namespace honk;
namespace honk_prover_tests {

// field is named Fscalar here because of clash with the Fr
template <class Fscalar> class ProverTests : public testing::Test {

  public:
    /**
     * @brief Simple test of Prover instantiation. This is handy while were in the Honk PoC building phase but may
     * eventually be unnecessary.
     *
     */
    static void test_prover_instantiation()
    {
        // Define some mock inputs for ProvingKey constructor
        size_t num_gates = 8;
        size_t num_public_inputs = 0;
        auto reference_string = std::make_shared<waffle::FileReferenceString>(num_gates + 1, "../srs_db/ignition");

        // Instatiate a proving_key and make a pointer to it
        auto proving_key =
            std::make_shared<waffle::proving_key>(num_gates, num_public_inputs, reference_string, waffle::STANDARD);

        // Instantiate a Prover with the proving_key pointer
        auto honk_prover = StandardProver(proving_key);
    };

    /**
     * @brief Test the correctness of the computation of the permutation grand product polynomial z_permutation
     * @details This test compares a simple, unoptimized, easily readable calculation of the grand product z_permutation
     * to the optimized implementation used by the prover. It's purpose is to provide confidence that some optimization
     * introduced into the calculation has not changed the result.
     */
    static void test_grand_product_construction()
    {
        using barretenberg::polynomial;

        // Define some mock inputs for proving key constructor
        static const size_t num_gates = 8;
        static const size_t num_public_inputs = 0;
        auto reference_string = std::make_shared<waffle::FileReferenceString>(num_gates + 1, "../srs_db/ignition");

        // Instatiate a proving_key and make a pointer to it. This will be used to instantiate a Prover.
        auto proving_key =
            std::make_shared<waffle::proving_key>(num_gates, num_public_inputs, reference_string, waffle::STANDARD);

        static const size_t program_width = StandardProver::settings_::program_width;

        // Construct mock wire and permutation polynomials and add them to the proving_key.
        // Note: for the purpose of checking the consistency between two methods of computing z_perm, these polynomials
        // can simply be random. We're not interested in the particular properties of the result.
        std::vector<polynomial> wires;
        std::vector<polynomial> sigmas;
        for (size_t i = 0; i < program_width; ++i) {
            polynomial wire_poly(proving_key->n, proving_key->n);
            polynomial sigma_poly(proving_key->n, proving_key->n);
            for (size_t j = 0; j < proving_key->n; ++j) {
                wire_poly[j] = Fscalar::random_element();
                sigma_poly[j] = Fscalar::random_element();
            }
            // Copy the wires and sigmas for use in constructing the test-owned z_perm
            wires.emplace_back(wire_poly);
            sigmas.emplace_back(sigma_poly);

            // Add polys to proving_key; to be used by the prover in constructing it's own z_perm
            std::string wire_id = "w_" + std::to_string(i + 1) + "_lagrange";
            std::string sigma_id = "sigma_" + std::to_string(i + 1) + "_lagrange";
            proving_key->polynomial_cache.put(wire_id, std::move(wire_poly));
            proving_key->polynomial_cache.put(sigma_id, std::move(sigma_poly));
        }

        // Instantiate a Prover with pointer to the proving_key just constructed
        auto honk_prover = StandardProver(proving_key);

        // Method 1: Compute z_perm using 'compute_grand_product_polynomial' as the prover would in practice
        honk_prover.compute_grand_product_polynomial();

        // Method 2: Compute z_perm locally using the simplest non-optimized syntax possible. The comment below,
        // which describes the computation in 4 steps, is adapted from a similar comment in
        // compute_grand_product_polynomial.
        /*
         * Assume program_width 3. Z_perm may be defined in terms of its values
         * on X_i = 0,1,...,n-1 as Z_perm[0] = 1 and for i = 1:n-1
         *
         *                  (w_1(j) + β⋅id_1(j) + γ) ⋅ (w_2(j) + β⋅id_2(j) + γ) ⋅ (w_3(j) + β⋅id_3(j) + γ)
         * Z_perm[i] = ∏ --------------------------------------------------------------------------------
         *                  (w_1(j) + β⋅σ_1(j) + γ) ⋅ (w_2(j) + β⋅σ_2(j) + γ) ⋅ (w_3(j) + β⋅σ_3(j) + γ)
         *
         * where ∏ := ∏_{j=0:i-1} and id_i(X) = id(X) + n*(i-1). These evaluations are constructed over the
         * course of three steps. For expositional simplicity, write Z_perm[i] as
         *
         *                A_1(j) ⋅ A_2(j) ⋅ A_3(j)
         * Z_perm[i] = ∏ --------------------------
         *                B_1(j) ⋅ B_2(j) ⋅ B_3(j)
         *
         * Step 1) Compute the 2*program_width length-n polynomials A_i and B_i
         * Step 2) Compute the 2*program_width length-n polynomials ∏ A_i(j) and ∏ B_i(j)
         * Step 3) Compute the two length-n polynomials defined by
         *          numer[i] = ∏ A_1(j)⋅A_2(j)⋅A_3(j), and denom[i] = ∏ B_1(j)⋅B_2(j)⋅B_3(j)
         * Step 4) Compute Z_perm[i+1] = numer[i]/denom[i] (recall: Z_perm[0] = 1)
         */

        // Make scratch space for the numerator and denominator accumulators.
        std::array<std::array<Fscalar, num_gates>, program_width> numererator_accum;
        std::array<std::array<Fscalar, num_gates>, program_width> denominator_accum;

        // Get random challenges
        // (TODO(luke): set these up to come from a transcript. Must match actual implementation
        Fscalar beta = Fscalar::one();
        Fscalar gamma = Fscalar::one();

        // Step (1)
        for (size_t i = 0; i < proving_key->n; ++i) {
            for (size_t k = 0; k < program_width; ++k) {
                Fscalar idx = k * proving_key->n + i;                                  // id_k[i]
                numererator_accum[k][i] = wires[k][i] + (idx * beta) + gamma;          // w_k(i) + β.(k*n+i) + γ
                denominator_accum[k][i] = wires[k][i] + (sigmas[k][i] * beta) + gamma; // w_k(i) + β.σ_k(i) + γ
            }
        }

        // Step (2)
        for (size_t k = 0; k < program_width; ++k) {
            for (size_t i = 0; i < proving_key->n - 1; ++i) {
                numererator_accum[k][i + 1] *= numererator_accum[k][i];
                denominator_accum[k][i + 1] *= denominator_accum[k][i];
            }
        }

        // Step (3)
        for (size_t i = 0; i < proving_key->n; ++i) {
            for (size_t k = 1; k < program_width; ++k) {
                numererator_accum[0][i] *= numererator_accum[k][i];
                denominator_accum[0][i] *= denominator_accum[k][i];
            }
        }

        // Step (4)
        polynomial z_perm(proving_key->n, proving_key->n);
        z_perm[0] = Fscalar::one(); // Z_0 = 1
        // Note: in practice, we replace this expensive element-wise division with Montgomery batch inversion
        for (size_t i = 0; i < proving_key->n - 1; ++i) {
            z_perm[i + 1] = numererator_accum[0][i] / denominator_accum[0][i];
        }

        // Check consistency between locally computed z_perm and the one computed by the prover
        EXPECT_EQ(z_perm, honk_prover.proving_key->polynomial_cache.get("z_perm"));
    };
};

typedef testing::Types<barretenberg::fr> FieldTypes;
TYPED_TEST_SUITE(ProverTests, FieldTypes);

TYPED_TEST(ProverTests, prover_instantiation)
{
    TestFixture::test_prover_instantiation();
}

TYPED_TEST(ProverTests, grand_product_construction)
{
    TestFixture::test_grand_product_construction();
}

} // namespace honk_prover_tests