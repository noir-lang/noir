
#include "barretenberg/plonk_honk_shared/library/grand_product_library.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_flavor.hpp"

#include "barretenberg/srs/factories/file_crs_factory.hpp"
#include <gtest/gtest.h>
using namespace bb;

template <class FF> class GrandProductTests : public testing::Test {

    using Polynomial = bb::Polynomial<FF>;

  public:
    void SetUp() { srs::init_crs_factory("../srs_db/ignition"); }
    /**
     * @brief Get a random polynomial
     *
     * @param size
     * @return Polynomial
     */
    static constexpr Polynomial get_random_polynomial(size_t size)
    {
        Polynomial random_polynomial{ size };
        for (auto& coeff : random_polynomial) {
            coeff = FF::random_element();
        }
        return random_polynomial;
    }

    static void populate_span(auto& polynomial_view, const auto& polynomial)
    {
        ASSERT(polynomial_view.size() <= polynomial.size());
        for (size_t idx = 0; idx < polynomial.size(); idx++) {
            polynomial_view[idx] = polynomial[idx];
        }
    };

    /**
     * @brief Check consistency of the computation of the permutation grand product polynomial z_permutation.
     * @details This test compares a simple, unoptimized, easily readable calculation of the grand product z_permutation
     * to the optimized implementation used by the prover. It's purpose is to provide confidence that some optimization
     * introduced into the calculation has not changed the result.
     * @note This test does confirm the correctness of z_permutation, only that the two implementations yield an
     * identical result.
     */
    template <typename Flavor> static void test_permutation_grand_product_construction()
    {
        using ProverPolynomials = typename Flavor::ProverPolynomials;

        // Set a mock circuit size
        static const size_t circuit_size = 8;

        // Construct a ProverPolynomials object with completely random polynomials
        ProverPolynomials prover_polynomials;
        for (auto& poly : prover_polynomials.get_all()) {
            poly = get_random_polynomial(circuit_size);
        }

        // Get random challenges
        auto beta = FF::random_element();
        auto gamma = FF::random_element();

        RelationParameters<FF> params{
            .eta = 0,
            .beta = beta,
            .gamma = gamma,
            .public_input_delta = 1,
            .lookup_grand_product_delta = 1,
        };

        // Method 1: Compute z_perm using 'compute_grand_product_polynomial' as the prover would in practice
        constexpr size_t PERMUTATION_RELATION_INDEX = 0;
        using LHS =
            typename std::tuple_element<PERMUTATION_RELATION_INDEX, typename Flavor::GrandProductRelations>::type;
        ASSERT(Flavor::NUM_WIRES == 4);
        using RHS = typename bb::UltraPermutationRelation<FF>;
        static_assert(std::same_as<LHS, RHS>);
        compute_grand_product<Flavor, RHS>(prover_polynomials, params);

        // Method 2: Compute z_perm locally using the simplest non-optimized syntax possible. The comment below,
        // which describes the computation in 4 steps, is adapted from a similar comment in
        // compute_grand_product_polynomial.
        /*
         * Assume Flavor::NUM_WIRES 3. Z_perm may be defined in terms of its values
         * on X_i = 0,1,...,n-1 as Z_perm[0] = 0 and for i = 1:n-1
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
         * Step 1) Compute the 2*Flavor::NUM_WIRES length-n polynomials A_i and B_i
         * Step 2) Compute the 2*Flavor::NUM_WIRES length-n polynomials ∏ A_i(j) and ∏ B_i(j)
         * Step 3) Compute the two length-n polynomials defined by
         *          numer[i] = ∏ A_1(j)⋅A_2(j)⋅A_3(j), and denom[i] = ∏ B_1(j)⋅B_2(j)⋅B_3(j)
         * Step 4) Compute Z_perm[i+1] = numer[i]/denom[i] (recall: Z_perm[0] = 1)
         */

        // Make scratch space for the numerator and denominator accumulators.
        std::array<std::array<FF, circuit_size>, Flavor::NUM_WIRES> numerator_accum;
        std::array<std::array<FF, circuit_size>, Flavor::NUM_WIRES> denominator_accum;

        auto wires = prover_polynomials.get_wires();
        auto sigmas = prover_polynomials.get_sigmas();
        auto ids = prover_polynomials.get_ids();
        // Step (1)
        for (size_t i = 0; i < circuit_size; ++i) {
            for (size_t k = 0; k < Flavor::NUM_WIRES; ++k) {
                numerator_accum[k][i] = wires[k][i] + (ids[k][i] * beta) + gamma;      // w_k(i) + β.id_k(i) + γ
                denominator_accum[k][i] = wires[k][i] + (sigmas[k][i] * beta) + gamma; // w_k(i) + β.σ_k(i) + γ
            }
        }

        // Step (2)
        for (size_t k = 0; k < Flavor::NUM_WIRES; ++k) {
            for (size_t i = 0; i < circuit_size - 1; ++i) {
                numerator_accum[k][i + 1] *= numerator_accum[k][i];
                denominator_accum[k][i + 1] *= denominator_accum[k][i];
            }
        }

        // Step (3)
        for (size_t i = 0; i < circuit_size; ++i) {
            for (size_t k = 1; k < Flavor::NUM_WIRES; ++k) {
                numerator_accum[0][i] *= numerator_accum[k][i];
                denominator_accum[0][i] *= denominator_accum[k][i];
            }
        }

        // Step (4)
        Polynomial z_permutation_expected(circuit_size);
        z_permutation_expected[0] = FF::zero(); // Z_0 = 1
        // Note: in practice, we replace this expensive element-wise division with Montgomery batch inversion
        for (size_t i = 0; i < circuit_size - 1; ++i) {
            z_permutation_expected[i + 1] = numerator_accum[0][i] / denominator_accum[0][i];
        }

        // Check consistency between locally computed z_perm and the one computed by the prover library
        EXPECT_EQ(prover_polynomials.z_perm, z_permutation_expected);
    };
};

using FieldTypes = testing::Types<bb::fr>;
TYPED_TEST_SUITE(GrandProductTests, FieldTypes);

TYPED_TEST(GrandProductTests, GrandProductPermutation)
{
    TestFixture::template test_permutation_grand_product_construction<UltraFlavor>();
}
