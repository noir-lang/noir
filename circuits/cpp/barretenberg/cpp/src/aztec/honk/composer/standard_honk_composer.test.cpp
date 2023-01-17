#include "standard_honk_composer.hpp"
#include "numeric/uint256/uint256.hpp"
#include "plonk/proof_system/types/polynomial_manifest.hpp"
#include <cstdint>
#include <honk/proof_system/prover.hpp>
#include <honk/sumcheck/polynomials/multivariates.hpp>
#include <gtest/gtest.h>

using namespace honk;

namespace test_standard_honk_composer {
/**
 * @brief The goal of this test is to check that the sigma permutation vectors for honk are generated correctly.
 *
 * @details Specifically:
 * 1) That they are indeed a permutation of all initial indices
 * 2) That if the permutation argument is computed with witness values, the values from the identity permutation and
 * sigma permutation are equal
 */
TEST(standard_honk_composer, test_sigma_and_id_correctness)
{
    StandardHonkComposer composer = StandardHonkComposer();
    fr a = fr::one();
    uint32_t a_idx = composer.add_variable(a);
    fr b = fr::one();
    fr c = a + b;
    fr d = a + c;
    uint32_t b_idx = composer.add_variable(b);
    uint32_t c_idx = composer.add_variable(c);
    uint32_t d_idx = composer.add_variable(d);
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });

    composer.create_add_gate({ d_idx, c_idx, a_idx, fr::one(), fr::neg_one(), fr::neg_one(), fr::zero() });

    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ b_idx, a_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });

    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });

    auto proving_key = composer.compute_proving_key();
    barretenberg::fr left = barretenberg::fr::one();
    barretenberg::fr right = barretenberg::fr::one();
    // Let's check that indices are the same and nothing is lost, first
    for (size_t i = 0; i < composer.program_width; i++) {

        std::string index = std::to_string(i + 1);
        auto polynomial = proving_key->polynomial_cache.get("sigma_" + index + "_lagrange");
        for (size_t j = 0; j < proving_key->n; j++) {
            left *= i * (proving_key->n) + j;
            right *= polynomial[j];
        }
    }

    EXPECT_EQ(left, right);
    // Using the same random beta and gamma as in the permutation argument
    barretenberg::fr beta = barretenberg::fr::random_element();
    barretenberg::fr gamma = barretenberg::fr::random_element();
    left = barretenberg::fr::one();
    right = barretenberg::fr::one();

    // Now let's check that witness values correspond to the permutation
    composer.compute_witness();
    for (size_t i = 0; i < composer.program_width; i++) {

        std::string index = std::to_string(i + 1);
        auto permutation_polynomial = proving_key->polynomial_cache.get("sigma_" + index + "_lagrange");
        auto witness_polynomial = proving_key->polynomial_cache.get("w_" + index + "_lagrange");
        auto id_polynomial = proving_key->polynomial_cache.get("id_" + index + "_lagrange");
        // left = ∏ᵢ,ⱼ(ωᵢ,ⱼ + β⋅ind(i,j) + γ)
        // right = ∏ᵢ,ⱼ(ωᵢ,ⱼ + β⋅σ(i,j) + γ)
        for (size_t j = 0; j < proving_key->n; j++) {
            auto current_witness = witness_polynomial[j];
            left *= current_witness + beta * id_polynomial[j] + gamma;
            right *= current_witness + beta * permutation_polynomial[j] + gamma;
        }
    }
    EXPECT_EQ(left, right);
}

/**
 * @brief Check the correctness of lagrange polynomials generated during proving key computation
 *
 */
TEST(standard_honk_composer, test_lagrange_polynomial_correctness)
{
    // Create a composer and a dummy circuit with a few gates
    StandardHonkComposer composer = StandardHonkComposer();
    fr a = fr::one();
    uint32_t a_idx = composer.add_variable(a);
    fr b = fr::one();
    fr c = a + b;
    fr d = a + c;
    uint32_t b_idx = composer.add_variable(b);
    uint32_t c_idx = composer.add_variable(c);
    uint32_t d_idx = composer.add_variable(d);
    for (size_t i = 0; i < 16; i++) {
        composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
        composer.create_add_gate({ d_idx, c_idx, a_idx, fr::one(), fr::neg_one(), fr::neg_one(), fr::zero() });
    }
    // Generate proving key
    auto proving_key = composer.compute_proving_key();
    // Generate a random polynomial
    barretenberg::polynomial random_polynomial = barretenberg::polynomial(proving_key->n, proving_key->n);
    for (size_t i = 0; i < proving_key->n; i++) {
        random_polynomial[i] = barretenberg::fr::random_element();
    }
    // Compute inner product of random polynomial and the first lagrange polynomial
    barretenberg::polynomial first_lagrange_polynomial = proving_key->polynomial_cache.get("L_first_lagrange");
    barretenberg::fr first_product(0);
    for (size_t i = 0; i < proving_key->n; i++) {
        first_product += random_polynomial[i] * first_lagrange_polynomial[i];
    }
    EXPECT_EQ(first_product, random_polynomial[0]);

    // Compute inner product of random polynomial and the last lagrange polynomial
    barretenberg::polynomial last_lagrange_polynomial = proving_key->polynomial_cache.get("L_last_lagrange");
    barretenberg::fr last_product(0);
    for (size_t i = 0; i < proving_key->n; i++) {
        last_product += random_polynomial[i] * last_lagrange_polynomial[i];
    }
    EXPECT_EQ(last_product, random_polynomial[proving_key->n - 1]);
}

/**
 * @brief Test that the assert_equal method in composer is working as intended
 *
 * @details We show equality of witness values through permutation arguments, so the assert_equal method changes the
 * underlying variable structure. If we bind two real variables through it, we expect their wire copy cycles to be
 * merged.
 * In this test we create two almost identical circuits. They differ because one
 */
TEST(standard_honk_composer, test_assert_equal)
{

    /**
     * @brief A function that creates a simple circuit with repeated gates, leading to large permutation cycles
     *
     */
    auto create_simple_circuit = [](auto& composer) {
        fr a = fr::one();
        uint32_t a_idx = composer.add_variable(a);
        fr b = fr::one();
        fr c = a + b;
        uint32_t b_idx = composer.add_variable(b);
        uint32_t c_idx = composer.add_variable(c);

        for (size_t i = 0; i < 10; i++) {
            composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
            composer.create_add_gate({ b_idx, a_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
        }
        return std::make_tuple(a_idx, b_idx);
    };
    /**
     * @brief A function that computes the largest cycle from the sigma permutation generated by the composer
     *
     */
    auto get_maximum_cycle = [](auto& composer) {
        // Compute the proving key for sigma polynomials
        auto proving_key = composer.compute_proving_key();
        auto permutation_length = composer.program_width * proving_key->n;
        std::vector<polynomial> sigma_polynomials;

        // Put the sigma polynomials into a vector for easy access
        for (size_t i = 0; i < composer.program_width; i++) {
            std::string index = std::to_string(i + 1);
            sigma_polynomials.push_back(proving_key->polynomial_cache.get("sigma_" + index + "_lagrange"));
        }

        // Let's compute the maximum cycle
        size_t maximum_cycle = 0;

        std::vector<bool> visited_indices;

        visited_indices.resize(permutation_length, false);

        for (size_t i = 0; i < permutation_length;) {
            // Jump to first unvisited member in the cycle
            // We check that i is limited by permutation_length
            while (visited_indices[i] && (i < permutation_length)) {
                i++;
            }
            if (i >= permutation_length) {
                break;
            }
            auto starting_element = i;
            auto next_element_big = static_cast<uint256_t>(sigma_polynomials[i / proving_key->n][i % proving_key->n]);
            EXPECT_LE(next_element_big, uint256_t(UINT32_MAX));
            auto next_element = static_cast<size_t>(next_element_big.data[0]);
            size_t cycle_length = 1;
            visited_indices[i] = true;

            // Jump through the cycle untill we reach the start or the permutation length exceeds the possible maximum
            while ((next_element != starting_element) && cycle_length < (permutation_length + 1)) {
                // Update cycle length and visited index infromation
                cycle_length++;
                visited_indices[next_element] = true;
                // Get next index
                next_element_big = static_cast<uint256_t>(
                    sigma_polynomials[next_element / proving_key->n][next_element % proving_key->n]);
                EXPECT_LE(next_element_big, uint256_t(UINT32_MAX));
                next_element = static_cast<size_t>(next_element_big.data[0]);
            }
            // If cycle_length is larger than permutation length, then instead of just a cycle we have a runway,too,
            // which is incorrect
            EXPECT_LE(cycle_length, permutation_length);

            // Update the maximum cycle
            if (cycle_length > maximum_cycle) {
                maximum_cycle = cycle_length;
            }
        }
        return maximum_cycle;
    };

    // Get 2 circuits
    StandardHonkComposer composer_no_assert_equal = StandardHonkComposer();
    StandardHonkComposer composer_with_assert_equal = StandardHonkComposer();

    // Construct circuits
    create_simple_circuit(composer_no_assert_equal);
    auto assert_eq_params = create_simple_circuit(composer_with_assert_equal);

    // Use assert_equal on one of them
    composer_with_assert_equal.assert_equal(std::get<0>(assert_eq_params),
                                            std::get<1>(assert_eq_params),
                                            "Equality asssertion in standard honk composer test");

    // Check that the maximum cycle in the one, where we used assert_equal, is twice as long
    EXPECT_EQ(get_maximum_cycle(composer_with_assert_equal), get_maximum_cycle(composer_no_assert_equal) * 2);
}

TEST(StandarHonkComposer, BaseCase)
{
    auto composer = StandardHonkComposer();
    fr a = fr::one();
    composer.circuit_constructor.add_public_variable(a);

    auto prover = composer.create_unrolled_prover();
    // waffle::Verifier verifier = composer.create_verifier();
    auto multivariates = honk::sumcheck::Multivariates<fr, waffle::STANDARD_HONK_MANIFEST_SIZE>(prover.proving_key);
    (void)multivariates;
    waffle::plonk_proof proof = prover.construct_proof();

    // bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
    // EXPECT_EQ(result, true);
}
} // namespace test_standard_honk_composer
