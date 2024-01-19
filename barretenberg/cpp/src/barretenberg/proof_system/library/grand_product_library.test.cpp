
#include "grand_product_library.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/flavor/ultra.hpp"
#include "barretenberg/polynomials/polynomial.hpp"

#include "barretenberg/srs/factories/file_crs_factory.hpp"
#include <gtest/gtest.h>

using namespace bb::honk;
namespace grand_product_library_tests {

template <class FF> class GrandProductTests : public testing::Test {

    using Polynomial = bb::Polynomial<FF>;

  public:
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
        // Define some mock inputs for proving key constructor
        static const size_t num_gates = 8;
        static const size_t num_public_inputs = 0;

        // Instatiate a proving_key and make a pointer to it. This will be used to instantiate a Prover.
        auto proving_key = std::make_shared<typename Flavor::ProvingKey>(num_gates, num_public_inputs);

        // static const size_t program_width = StandardProver::settings_::program_width;

        // Construct mock wire and permutation polynomials.
        // Note: for the purpose of checking the consistency between two methods of computing z_perm, these polynomials
        // can simply be random. We're not interested in the particular properties of the result.
        std::vector<Polynomial> wires;
        std::vector<Polynomial> sigmas;
        std::vector<Polynomial> ids;

        auto wire_polynomials = proving_key->get_wires();
        auto sigma_polynomials = proving_key->get_sigma_polynomials();
        auto id_polynomials = proving_key->get_id_polynomials();
        for (size_t i = 0; i < Flavor::NUM_WIRES; ++i) {
            wires.emplace_back(get_random_polynomial(num_gates));
            sigmas.emplace_back(get_random_polynomial(num_gates));
            ids.emplace_back(get_random_polynomial(num_gates));

            populate_span(wire_polynomials[i], wires[i]);
            populate_span(sigma_polynomials[i], sigmas[i]);
            populate_span(id_polynomials[i], ids[i]);
        }

        // Get random challenges
        auto beta = FF::random_element();
        auto gamma = FF::random_element();

        bb::RelationParameters<FF> params{
            .eta = 0,
            .beta = beta,
            .gamma = gamma,
            .public_input_delta = 1,
            .lookup_grand_product_delta = 1,
        };

        typename Flavor::ProverPolynomials prover_polynomials;
        for (auto [prover_poly, key_poly] : zip_view(prover_polynomials.get_unshifted(), proving_key->get_all())) {
            ASSERT(bb::flavor_get_label(prover_polynomials, prover_poly) ==
                   bb::flavor_get_label(*proving_key, key_poly));
            prover_poly = key_poly.share();
        }

        // Method 1: Compute z_perm using 'compute_grand_product_polynomial' as the prover would in practice
        constexpr size_t PERMUTATION_RELATION_INDEX = 0;
        using LHS =
            typename std::tuple_element<PERMUTATION_RELATION_INDEX, typename Flavor::GrandProductRelations>::type;
        ASSERT(Flavor::NUM_WIRES == 4);
        using RHS = typename bb::UltraPermutationRelation<FF>;
        static_assert(std::same_as<LHS, RHS>);
        grand_product_library::compute_grand_product<Flavor, RHS>(
            proving_key->circuit_size, prover_polynomials, params);

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
        std::array<std::array<FF, num_gates>, Flavor::NUM_WIRES> numerator_accum;
        std::array<std::array<FF, num_gates>, Flavor::NUM_WIRES> denominator_accum;

        // Step (1)
        for (size_t i = 0; i < proving_key->circuit_size; ++i) {
            for (size_t k = 0; k < Flavor::NUM_WIRES; ++k) {
                numerator_accum[k][i] = wires[k][i] + (ids[k][i] * beta) + gamma;      // w_k(i) + β.id_k(i) + γ
                denominator_accum[k][i] = wires[k][i] + (sigmas[k][i] * beta) + gamma; // w_k(i) + β.σ_k(i) + γ
            }
        }

        // Step (2)
        for (size_t k = 0; k < Flavor::NUM_WIRES; ++k) {
            for (size_t i = 0; i < proving_key->circuit_size - 1; ++i) {
                numerator_accum[k][i + 1] *= numerator_accum[k][i];
                denominator_accum[k][i + 1] *= denominator_accum[k][i];
            }
        }

        // Step (3)
        for (size_t i = 0; i < proving_key->circuit_size; ++i) {
            for (size_t k = 1; k < Flavor::NUM_WIRES; ++k) {
                numerator_accum[0][i] *= numerator_accum[k][i];
                denominator_accum[0][i] *= denominator_accum[k][i];
            }
        }

        // Step (4)
        Polynomial z_permutation_expected(proving_key->circuit_size);
        z_permutation_expected[0] = FF::zero(); // Z_0 = 1
        // Note: in practice, we replace this expensive element-wise division with Montgomery batch inversion
        for (size_t i = 0; i < proving_key->circuit_size - 1; ++i) {
            z_permutation_expected[i + 1] = numerator_accum[0][i] / denominator_accum[0][i];
        }

        // Check consistency between locally computed z_perm and the one computed by the prover library
        EXPECT_EQ(proving_key->z_perm, z_permutation_expected);
    };

    /**
     * @brief Check consistency of the computation of the lookup grand product polynomial z_lookup.
     * @details This test compares a simple, unoptimized, easily readable calculation of the grand product z_lookup
     * to the optimized implementation used by the prover. It's purpose is to provide confidence that some optimization
     * introduced into the calculation has not changed the result.
     * @note This test does confirm the correctness of z_lookup, only that the two implementations yield an
     * identical result.
     */
    static void test_lookup_grand_product_construction()
    {
        // Define some mock inputs for proving key constructor
        static const size_t circuit_size = 8;
        static const size_t num_public_inputs = 0;

        // Instatiate a proving_key and make a pointer to it. This will be used to instantiate a Prover.
        using Flavor = flavor::Ultra;
        auto proving_key = std::make_shared<typename Flavor::ProvingKey>(circuit_size, num_public_inputs);

        // Construct mock wire and permutation polynomials.
        // Note: for the purpose of checking the consistency between two methods of computing z_lookup, these
        // polynomials can simply be random. We're not interested in the particular properties of the result.
        std::vector<Polynomial> wires;
        auto wire_polynomials = proving_key->get_wires();
        // Note(luke): Use of 3 wires is fundamental to the structure of the tables and should not be tied to NUM_WIRES
        // for now
        for (size_t i = 0; i < 3; ++i) { // TODO(Cody): will this test ever generalize?
            Polynomial random_polynomial = get_random_polynomial(circuit_size);
            random_polynomial[0] = 0; // when computing shifts, 1st element needs to be 0
            wires.emplace_back(random_polynomial);
            populate_span(wire_polynomials[i], random_polynomial);
        }

        std::vector<Polynomial> tables;
        auto table_polynomials = proving_key->get_table_polynomials();
        for (auto& table_polynomial : table_polynomials) {
            Polynomial random_polynomial = get_random_polynomial(circuit_size);
            random_polynomial[0] = 0; // when computing shifts, 1st element needs to be 0
            tables.emplace_back(random_polynomial);
            populate_span(table_polynomial, random_polynomial);
        }

        auto sorted_batched = get_random_polynomial(circuit_size);
        sorted_batched[0] = 0; // when computing shifts, 1st element needs to be 0
        auto column_1_step_size = get_random_polynomial(circuit_size);
        auto column_2_step_size = get_random_polynomial(circuit_size);
        auto column_3_step_size = get_random_polynomial(circuit_size);
        auto lookup_index_selector = get_random_polynomial(circuit_size);
        auto lookup_selector = get_random_polynomial(circuit_size);

        proving_key->sorted_accum = sorted_batched.share();
        populate_span(proving_key->q_r, column_1_step_size);
        populate_span(proving_key->q_m, column_2_step_size);
        populate_span(proving_key->q_c, column_3_step_size);
        populate_span(proving_key->q_o, lookup_index_selector);
        populate_span(proving_key->q_lookup, lookup_selector);

        // Get random challenges
        auto beta = FF::random_element();
        auto gamma = FF::random_element();
        auto eta = FF::random_element();

        bb::RelationParameters<FF> params{
            .eta = eta,
            .beta = beta,
            .gamma = gamma,
            .public_input_delta = 1,
            .lookup_grand_product_delta = 1,
        };

        typename Flavor::ProverPolynomials prover_polynomials;
        for (auto [prover_poly, key_poly] : zip_view(prover_polynomials.get_unshifted(), proving_key->get_all())) {
            ASSERT(bb::flavor_get_label(prover_polynomials, prover_poly) ==
                   bb::flavor_get_label(*proving_key, key_poly));
            prover_poly = key_poly.share();
        }
        for (auto [prover_poly, key_poly] :
             zip_view(prover_polynomials.get_shifted(), proving_key->get_to_be_shifted())) {
            ASSERT(bb::flavor_get_label(prover_polynomials, prover_poly) ==
                   bb::flavor_get_label(*proving_key, key_poly) + "_shift");
            prover_poly = key_poly.shifted();
        }
        // Test a few assignments
        EXPECT_EQ(&proving_key->z_lookup[0], &prover_polynomials.z_lookup[0]);
        EXPECT_EQ(&proving_key->sigma_1[0], &prover_polynomials.sigma_1[0]);
        EXPECT_EQ(&proving_key->lagrange_last[0], &prover_polynomials.lagrange_last[0]);

        // Method 1: Compute z_lookup using the prover library method
        constexpr size_t LOOKUP_RELATION_INDEX = 1;
        using LHS = typename std::tuple_element<LOOKUP_RELATION_INDEX, typename Flavor::GrandProductRelations>::type;
        using RHS = bb::LookupRelation<FF>;
        static_assert(std::same_as<LHS, RHS>);
        grand_product_library::compute_grand_product<Flavor, RHS>(
            proving_key->circuit_size, prover_polynomials, params);

        // Method 2: Compute the lookup grand product polynomial Z_lookup:
        //
        //                   ∏(1 + β) ⋅ ∏(q_lookup*f_k + γ) ⋅ ∏(t_k + βt_{k+1} + γ(1 + β))
        // Z_lookup(X_j) = -----------------------------------------------------------------
        //                                   ∏(s_k + βs_{k+1} + γ(1 + β))
        //
        // in a way that is simple to read (but inefficient). See prover library method for more details.
        const FF eta_sqr = eta.sqr();
        const FF eta_cube = eta_sqr * eta;

        std::array<Polynomial, 4> accumulators;
        for (size_t i = 0; i < 4; ++i) {
            accumulators[i] = Polynomial{ circuit_size };
        }

        // Step (1)

        // Note: block_mask is used for efficient modulus, i.e. i % N := i & (N-1), for N = 2^k
        const size_t block_mask = circuit_size - 1;
        // Initialize 't(X)' to be used in an expression of the form t(X) + β*t(Xω)
        FF table_i = tables[0][0] + tables[1][0] * eta + tables[2][0] * eta_sqr + tables[3][0] * eta_cube;
        for (size_t i = 0; i < circuit_size; ++i) {
            size_t shift_idx = (i + 1) & block_mask;

            // f = (w_1 + q_2*w_1(Xω)) + η(w_2 + q_m*w_2(Xω)) + η²(w_3 + q_c*w_3(Xω)) + η³q_index.
            FF f_i = (wires[0][i] + wires[0][shift_idx] * column_1_step_size[i]) +
                     (wires[1][i] + wires[1][shift_idx] * column_2_step_size[i]) * eta +
                     (wires[2][i] + wires[2][shift_idx] * column_3_step_size[i]) * eta_sqr +
                     eta_cube * lookup_index_selector[i];

            // q_lookup * f + γ
            accumulators[0][i] = lookup_selector[i] * f_i + gamma;

            // t = t_1 + ηt_2 + η²t_3 + η³t_4
            FF table_i_plus_1 = tables[0][shift_idx] + eta * tables[1][shift_idx] + eta_sqr * tables[2][shift_idx] +
                                eta_cube * tables[3][shift_idx];

            // t + βt(Xω) + γ(1 + β)
            accumulators[1][i] = table_i + table_i_plus_1 * beta + gamma * (FF::one() + beta);

            // (1 + β)
            accumulators[2][i] = FF::one() + beta;

            // s + βs(Xω) + γ(1 + β)
            accumulators[3][i] = sorted_batched[i] + beta * sorted_batched[shift_idx] + gamma * (FF::one() + beta);

            // Set t(X_i) for next iteration
            table_i = table_i_plus_1;
        }

        // Step (2)
        for (auto& accum : accumulators) {
            for (size_t i = 0; i < circuit_size - 1; ++i) {
                accum[i + 1] *= accum[i];
            }
        }

        // Step (3)
        Polynomial z_lookup_expected(circuit_size);
        z_lookup_expected[0] = FF::zero(); // Z_lookup_0 = 0

        // Compute the numerator in accumulators[0]; The denominator is in accumulators[3]
        for (size_t i = 0; i < circuit_size - 1; ++i) {
            accumulators[0][i] *= accumulators[1][i] * accumulators[2][i];
        }
        // Compute Z_lookup_i, i = [1, n-1]
        for (size_t i = 0; i < circuit_size - 1; ++i) {
            z_lookup_expected[i + 1] = accumulators[0][i] / accumulators[3][i];
        }

        EXPECT_EQ(proving_key->z_lookup, z_lookup_expected);
    };
};

using FieldTypes = testing::Types<bb::fr>;
TYPED_TEST_SUITE(GrandProductTests, FieldTypes);

TYPED_TEST(GrandProductTests, GrandProductPermutation)
{
    TestFixture::template test_permutation_grand_product_construction<flavor::Ultra>();
}

TYPED_TEST(GrandProductTests, GrandProductLookup)
{
    TestFixture::test_lookup_grand_product_construction();
}

} // namespace grand_product_library_tests
