#include "barretenberg/plonk/proof_system/types/prover_settings.hpp"
#include "prover.hpp"
#include "prover_library.hpp"
#include "barretenberg/polynomials/polynomial.hpp"

#include "barretenberg/srs/reference_string/file_reference_string.hpp"
#include <array>
#include <string>
#include <vector>
#include <cstddef>
#include <gtest/gtest.h>

using namespace proof_system::honk;
namespace prover_library_tests {

template <class FF> class ProverLibraryTests : public testing::Test {

    using Polynomial = barretenberg::Polynomial<FF>;

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

    /**
     * @brief Check consistency of the computation of the permutation grand product polynomial z_permutation.
     * @details This test compares a simple, unoptimized, easily readable calculation of the grand product z_permutation
     * to the optimized implementation used by the prover. It's purpose is to provide confidence that some optimization
     * introduced into the calculation has not changed the result.
     * @note This test does confirm the correctness of z_permutation, only that the two implementations yield an
     * identical result.
     */
    template <size_t program_width> static void test_permutation_grand_product_construction()
    {
        // Define some mock inputs for proving key constructor
        static const size_t num_gates = 8;
        static const size_t num_public_inputs = 0;
        auto reference_string = std::make_shared<FileReferenceString>(num_gates + 1, "../srs_db/ignition");

        // Instatiate a proving_key and make a pointer to it. This will be used to instantiate a Prover.
        auto proving_key = std::make_shared<plonk::proving_key>(
            num_gates, num_public_inputs, reference_string, ComposerType::STANDARD_HONK);

        // static const size_t program_width = StandardProver::settings_::program_width;

        // Construct mock wire and permutation polynomials.
        // Note: for the purpose of checking the consistency between two methods of computing z_perm, these polynomials
        // can simply be random. We're not interested in the particular properties of the result.
        std::vector<Polynomial> wires;
        std::vector<Polynomial> sigmas;
        std::vector<Polynomial> ids;
        for (size_t i = 0; i < program_width; ++i) {
            wires.emplace_back(get_random_polynomial(num_gates));
            sigmas.emplace_back(get_random_polynomial(num_gates));
            ids.emplace_back(get_random_polynomial(num_gates));

            // Add sigma/ID polys to proving_key; to be used by the prover in constructing it's own z_perm
            std::string sigma_label = "sigma_" + std::to_string(i + 1) + "_lagrange";
            proving_key->polynomial_store.put(sigma_label, Polynomial{ sigmas[i] });
            std::string id_label = "id_" + std::to_string(i + 1) + "_lagrange";
            proving_key->polynomial_store.put(id_label, Polynomial{ ids[i] });
        }

        // Get random challenges
        auto beta = FF::random_element();
        auto gamma = FF::random_element();

        // Method 1: Compute z_perm using 'compute_grand_product_polynomial' as the prover would in practice
        Polynomial z_permutation =
            prover_library::compute_permutation_grand_product<program_width>(proving_key, wires, beta, gamma);

        // Method 2: Compute z_perm locally using the simplest non-optimized syntax possible. The comment below,
        // which describes the computation in 4 steps, is adapted from a similar comment in
        // compute_grand_product_polynomial.
        /*
         * Assume program_width 3. Z_perm may be defined in terms of its values
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
         * Step 1) Compute the 2*program_width length-n polynomials A_i and B_i
         * Step 2) Compute the 2*program_width length-n polynomials ∏ A_i(j) and ∏ B_i(j)
         * Step 3) Compute the two length-n polynomials defined by
         *          numer[i] = ∏ A_1(j)⋅A_2(j)⋅A_3(j), and denom[i] = ∏ B_1(j)⋅B_2(j)⋅B_3(j)
         * Step 4) Compute Z_perm[i+1] = numer[i]/denom[i] (recall: Z_perm[0] = 1)
         */

        // Make scratch space for the numerator and denominator accumulators.
        std::array<std::array<FF, num_gates>, program_width> numererator_accum;
        std::array<std::array<FF, num_gates>, program_width> denominator_accum;

        // Step (1)
        for (size_t i = 0; i < proving_key->circuit_size; ++i) {
            for (size_t k = 0; k < program_width; ++k) {
                numererator_accum[k][i] = wires[k][i] + (ids[k][i] * beta) + gamma;    // w_k(i) + β.id_k(i) + γ
                denominator_accum[k][i] = wires[k][i] + (sigmas[k][i] * beta) + gamma; // w_k(i) + β.σ_k(i) + γ
            }
        }

        // Step (2)
        for (size_t k = 0; k < program_width; ++k) {
            for (size_t i = 0; i < proving_key->circuit_size - 1; ++i) {
                numererator_accum[k][i + 1] *= numererator_accum[k][i];
                denominator_accum[k][i + 1] *= denominator_accum[k][i];
            }
        }

        // Step (3)
        for (size_t i = 0; i < proving_key->circuit_size; ++i) {
            for (size_t k = 1; k < program_width; ++k) {
                numererator_accum[0][i] *= numererator_accum[k][i];
                denominator_accum[0][i] *= denominator_accum[k][i];
            }
        }

        // Step (4)
        Polynomial z_permutation_expected(proving_key->circuit_size);
        z_permutation_expected[0] = FF::zero(); // Z_0 = 1
        // Note: in practice, we replace this expensive element-wise division with Montgomery batch inversion
        for (size_t i = 0; i < proving_key->circuit_size - 1; ++i) {
            z_permutation_expected[i + 1] = numererator_accum[0][i] / denominator_accum[0][i];
        }

        // Check consistency between locally computed z_perm and the one computed by the prover library
        EXPECT_EQ(z_permutation, z_permutation_expected);
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
        auto reference_string = std::make_shared<FileReferenceString>(circuit_size + 1, "../srs_db/ignition");

        // Instatiate a proving_key and make a pointer to it. This will be used to instantiate a Prover.
        auto proving_key = std::make_shared<plonk::proving_key>(
            circuit_size, num_public_inputs, reference_string, ComposerType::STANDARD_HONK);

        // Construct mock wire and permutation polynomials.
        // Note: for the purpose of checking the consistency between two methods of computing z_perm, these polynomials
        // can simply be random. We're not interested in the particular properties of the result.
        std::vector<Polynomial> wires;
        for (size_t i = 0; i < 3; ++i) {
            wires.emplace_back(get_random_polynomial(circuit_size));
        }
        std::vector<Polynomial> tables;
        for (size_t i = 0; i < 4; ++i) {
            tables.emplace_back(get_random_polynomial(circuit_size));
            std::string label = "table_value_" + std::to_string(i + 1) + "_lagrange";
            proving_key->polynomial_store.put(label, Polynomial{ tables[i] });
        }

        auto s_lagrange = get_random_polynomial(circuit_size);
        auto column_1_step_size = get_random_polynomial(circuit_size);
        auto column_2_step_size = get_random_polynomial(circuit_size);
        auto column_3_step_size = get_random_polynomial(circuit_size);
        auto lookup_index_selector = get_random_polynomial(circuit_size);
        auto lookup_selector = get_random_polynomial(circuit_size);

        proving_key->polynomial_store.put("s_lagrange", Polynomial{ s_lagrange });
        proving_key->polynomial_store.put("q_2_lagrange", Polynomial{ column_1_step_size });
        proving_key->polynomial_store.put("q_m_lagrange", Polynomial{ column_2_step_size });
        proving_key->polynomial_store.put("q_c_lagrange", Polynomial{ column_3_step_size });
        proving_key->polynomial_store.put("q_3_lagrange", Polynomial{ lookup_index_selector });
        proving_key->polynomial_store.put("table_type_lagrange", Polynomial{ lookup_selector });

        // Get random challenges
        auto beta = FF::random_element();
        auto gamma = FF::random_element();
        auto eta = FF::random_element();

        // Method 1: Compute z_lookup using the prover library method
        Polynomial z_lookup =
            prover_library::compute_lookup_grand_product(proving_key, wires, s_lagrange, eta, beta, gamma);

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
            accumulators[3][i] = s_lagrange[i] + beta * s_lagrange[shift_idx] + gamma * (FF::one() + beta);

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

        EXPECT_EQ(z_lookup, z_lookup_expected);
    };

    /**
     * @brief Check consistency of the computation of the sorted list accumulator
     * @details This test compares a simple, unoptimized, easily readable calculation of the sorted list accumulator
     * to the optimized implementation used by the prover. It's purpose is to provide confidence that some optimization
     * introduced into the calculation has not changed the result.
     * @note This test does confirm the correctness of the sorted list accumulator, only that the two implementations
     * yield an identical result.
     */
    static void test_sorted_list_accumulator_construction()
    {
        // Construct a proving_key
        static const size_t circuit_size = 8;
        static const size_t num_public_inputs = 0;
        auto reference_string = std::make_shared<FileReferenceString>(circuit_size + 1, "../srs_db/ignition");
        auto proving_key = std::make_shared<plonk::proving_key>(
            circuit_size, num_public_inputs, reference_string, ComposerType::STANDARD_HONK);

        // Get random challenge eta
        auto eta = FF::random_element();

        // Construct mock sorted list polynomials.
        std::vector<Polynomial> sorted_list_polynomials;
        for (size_t i = 0; i < 4; ++i) {
            sorted_list_polynomials.emplace_back(get_random_polynomial(circuit_size));
        }

        // Method 1: computed sorted list accumulator polynomial using prover library method
        Polynomial sorted_list_accumulator =
            prover_library::compute_sorted_list_accumulator(proving_key, sorted_list_polynomials, eta);

        // Method 2: Compute local sorted list accumulator simply and inefficiently
        const FF eta_sqr = eta.sqr();
        const FF eta_cube = eta_sqr * eta;

        // Compute s = s_1 + η*s_2 + η²*s_3 + η³*s_4
        Polynomial sorted_list_accumulator_expected{ sorted_list_polynomials[0] };
        for (size_t i = 0; i < circuit_size; ++i) {
            sorted_list_accumulator_expected[i] += sorted_list_polynomials[1][i] * eta +
                                                   sorted_list_polynomials[2][i] * eta_sqr +
                                                   sorted_list_polynomials[3][i] * eta_cube;
        }

        EXPECT_EQ(sorted_list_accumulator, sorted_list_accumulator_expected);
    };
};

typedef testing::Types<barretenberg::fr> FieldTypes;
TYPED_TEST_SUITE(ProverLibraryTests, FieldTypes);

TYPED_TEST(ProverLibraryTests, PermutationGrandProduct)
{
    TestFixture::template test_permutation_grand_product_construction<plonk::standard_settings::program_width>();
    TestFixture::template test_permutation_grand_product_construction<plonk::ultra_settings::program_width>();
}

TYPED_TEST(ProverLibraryTests, LookupGrandProduct)
{
    TestFixture::test_lookup_grand_product_construction();
}

TYPED_TEST(ProverLibraryTests, SortedListAccumulator)
{
    TestFixture::test_sorted_list_accumulator_construction();
}

} // namespace prover_library_tests
