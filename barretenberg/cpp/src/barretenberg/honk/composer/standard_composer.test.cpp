#include <cstddef>
#include <cstdint>
#include <gtest/gtest.h>
#include <vector>

#include "barretenberg/honk/composer/standard_composer.hpp"
#include "barretenberg/honk/proof_system/prover.hpp"
#include "barretenberg/honk/sumcheck/sumcheck_round.hpp"
#include "barretenberg/honk/utils/grand_product_delta.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/proof_system/circuit_builder/standard_circuit_builder.hpp"
#include "barretenberg/proof_system/relations/permutation_relation.hpp"
#include "barretenberg/proof_system/relations/relation_parameters.hpp"

using namespace proof_system::honk;

#define TYPE_ALIASES                                                                                                   \
    using Flavor = TypeParam;                                                                                          \
    using FF = typename Flavor::FF;                                                                                    \
    using CircuitBuilder = proof_system::StandardCircuitBuilder_<FF>;                                                  \
    using Composer = StandardComposer_<Flavor>;

namespace test_standard_honk_composer {

template <typename Flavor> class StandardHonkComposerTests : public ::testing::Test {
  protected:
    // TODO(640): The Standard Honk on Grumpkin test suite fails unless the SRS is initialised for every test.
    virtual void SetUp()
    {
        if constexpr (proof_system::IsGrumpkinFlavor<Flavor>) {
            barretenberg::srs::init_grumpkin_crs_factory("../srs_db/grumpkin");
        } else {
            barretenberg::srs::init_crs_factory("../srs_db/ignition");
        }
    };
};

using FlavorTypes = ::testing::Types<flavor::Standard, flavor::StandardGrumpkin>;
TYPED_TEST_SUITE(StandardHonkComposerTests, FlavorTypes);

/**
 * @brief The goal of this test is to check that the sigma permutation vectors for honk are generated correctly.
 *
 * @details Specifically:
 * 1) That they are indeed a permutation of all initial indices
 * 2) That if the permutation argument is computed with witness values, the values from the identity permutation and
 * sigma permutation are equal
 */
TYPED_TEST(StandardHonkComposerTests, SigmaIDCorrectness)
{
    TYPE_ALIASES
    auto test_permutation = [](CircuitBuilder& builder, Composer& composer) {
        auto instance = composer.create_instance(builder);
        auto prover = composer.create_prover(instance);
        auto proving_key = instance->proving_key;

        const auto n = proving_key->circuit_size;

        auto public_inputs = builder.get_public_inputs();
        auto num_public_inputs = public_inputs.size();
        auto num_gates = builder.get_num_gates();

        // Using the same random beta and gamma as in the permutation argument
        FF beta = FF::random_element();
        FF gamma = FF::random_element();

        FF left = FF::one();
        FF right = FF::one();

        // Let's check that indices are the same and nothing is lost, first
        size_t wire_idx = 0;
        for (auto& sigma_polynomial : proving_key->get_sigma_polynomials()) {
            for (size_t i = 0; i < n; ++i) {
                left *= (gamma + wire_idx * n + i);
                right *= (gamma + sigma_polynomial[i]);
            }
            // Ensure that the public inputs cycles are correctly broken
            // and fix the cycle by adding the extra terms
            if (wire_idx == 0) {
                for (size_t i = 0; i < num_public_inputs; ++i) {
                    EXPECT_EQ(sigma_polynomial[i], -FF(i + 1));
                    left *= (gamma - (i + 1));
                    right *= (gamma + (n + i));
                }
            }
            ++wire_idx;
        }

        EXPECT_EQ(left, right);

        left = FF::one();
        right = FF::one();

        auto permutation_polynomials = proving_key->get_sigma_polynomials();
        auto id_polynomials = proving_key->get_id_polynomials();
        auto wire_polynomials = proving_key->get_wires();
        for (size_t j = 0; j < Composer::NUM_WIRES; ++j) {
            std::string index = std::to_string(j + 1);
            const auto& permutation_polynomial = permutation_polynomials[j];
            const auto& witness_polynomial = wire_polynomials[j];
            const auto& id_polynomial = id_polynomials[j];
            // left = ∏ᵢ,ⱼ(ωᵢ,ⱼ + β⋅ind(i,j) + γ)
            // right = ∏ᵢ,ⱼ(ωᵢ,ⱼ + β⋅σ(i,j) + γ)
            for (size_t i = 0; i < proving_key->circuit_size; ++i) {
                const auto current_witness = witness_polynomial[i];
                left *= current_witness + beta * id_polynomial[i] + gamma;
                right *= current_witness + beta * permutation_polynomial[i] + gamma;
            }
            // check that the first rows are correctly set to handle public inputs.
            for (size_t i = 0; i < num_public_inputs; ++i) {
                if ((j == 0) || (j == 1)) {
                    EXPECT_EQ(witness_polynomial[i], public_inputs[i]);
                } else {
                    EXPECT_EQ(witness_polynomial[i], 0);
                }
            }
            // Check that the last rows are all 0
            for (size_t i = num_public_inputs + num_gates; i < n; ++i) {
                EXPECT_EQ(witness_polynomial[i], 0);
            }
        }

        // test correctness of the public input delta
        auto delta = proof_system::honk::compute_public_input_delta<Flavor>(public_inputs, beta, gamma, n);
        EXPECT_EQ(left / right, delta);

        for (size_t i = 0; i < num_public_inputs; ++i) {
            left *= public_inputs[i] - beta * (i + 1) + gamma;
            right *= public_inputs[i] + beta * (n + i) + gamma;
        }
        EXPECT_EQ(left, right);
    };

    auto builder = CircuitBuilder();
    FF a = FF::one();
    uint32_t a_idx = builder.add_variable(a);
    FF b = FF::one();
    FF c = a + b;
    uint32_t b_idx = builder.add_variable(b);
    uint32_t c_idx = builder.add_variable(c);
    FF d = a + c;
    uint32_t d_idx = builder.add_public_variable(d);

    uint32_t e_idx = builder.put_constant_variable(d);
    builder.assert_equal(e_idx, d_idx, "");

    builder.create_add_gate({ a_idx, b_idx, c_idx, FF::one(), FF::one(), FF::neg_one(), FF::zero() });
    builder.create_add_gate({ d_idx, c_idx, a_idx, FF::one(), FF::neg_one(), FF::neg_one(), FF::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, FF::one(), FF::one(), FF::neg_one(), FF::zero() });
    builder.create_add_gate({ a_idx, b_idx, c_idx, FF::one(), FF::one(), FF::neg_one(), FF::zero() });
    builder.create_add_gate({ b_idx, a_idx, c_idx, FF::one(), FF::one(), FF::neg_one(), FF::zero() });
    for (size_t i = 0; i < 30; ++i) {
        builder.create_add_gate({ a_idx, b_idx, c_idx, FF::one(), FF::one(), FF::neg_one(), FF::zero() });
    }

    auto composer = Composer();
    test_permutation(builder, composer);
}

/**
 * @brief Check the correctness of lagrange polynomials generated during proving key computation
 *
 */
TYPED_TEST(StandardHonkComposerTests, LagrangeCorrectness)
{
    TYPE_ALIASES
    using Polynomial = typename Flavor::Polynomial;
    // Create a dummy circuit with a few gates
    auto builder = CircuitBuilder();
    FF a = FF::one();
    uint32_t a_idx = builder.add_variable(a);
    FF b = FF::one();
    FF c = a + b;
    FF d = a + c;
    uint32_t b_idx = builder.add_variable(b);
    uint32_t c_idx = builder.add_variable(c);
    uint32_t d_idx = builder.add_variable(d);
    for (size_t i = 0; i < 16; i++) {
        builder.create_add_gate({ a_idx, b_idx, c_idx, FF::one(), FF::one(), FF::neg_one(), FF::zero() });
        builder.create_add_gate({ d_idx, c_idx, a_idx, FF::one(), FF::neg_one(), FF::neg_one(), FF::zero() });
    }

    // Generate proving key
    auto composer = Composer();
    auto instance = composer.create_instance(builder);
    auto prover = composer.create_prover(instance);
    auto proving_key = instance->proving_key;

    // Generate a random polynomial
    Polynomial random_polynomial = Polynomial(proving_key->circuit_size);
    for (size_t i = 0; i < proving_key->circuit_size; i++) {
        random_polynomial[i] = FF::random_element();
    }
    // Compute inner product of random polynomial and the first lagrange polynomial

    Polynomial first_lagrange_polynomial = proving_key->lagrange_first;
    FF first_product(0);
    for (size_t i = 0; i < proving_key->circuit_size; i++) {
        first_product += random_polynomial[i] * first_lagrange_polynomial[i];
    }
    EXPECT_EQ(first_product, random_polynomial[0]);

    // Compute inner product of random polynomial and the last lagrange polynomial
    auto last_lagrange_polynomial = proving_key->lagrange_last;
    FF last_product(0);
    for (size_t i = 0; i < proving_key->circuit_size; i++) {
        last_product += random_polynomial[i] * last_lagrange_polynomial[i];
    }
    EXPECT_EQ(last_product, random_polynomial[proving_key->circuit_size - 1]);
}

/**
 * @brief Test that the assert_equal method in composer is working as intended
 *
 * @details We show equality of witness values through permutation arguments, so the assert_equal method changes the
 * underlying variable structure. If we bind two real variables through it, we expect their wire copy cycles to be
 * merged.
 * In this test we create two almost identical circuits. They differ because one
 */
TYPED_TEST(StandardHonkComposerTests, AssertEquals)
{
    TYPE_ALIASES

    /**
     * @brief A function that creates a simple circuit with repeated gates, leading to large permutation cycles
     *
     */
    auto create_simple_circuit = [](auto& builder) {
        FF a = FF::one();
        uint32_t a_idx = builder.add_variable(a);
        FF b = FF::one();
        FF c = a + b;
        uint32_t b_idx = builder.add_variable(b);
        uint32_t c_idx = builder.add_variable(c);

        for (size_t i = 0; i < 10; i++) {
            builder.create_add_gate({ a_idx, b_idx, c_idx, FF::one(), FF::one(), FF::neg_one(), FF::zero() });
            builder.create_add_gate({ b_idx, a_idx, c_idx, FF::one(), FF::one(), FF::neg_one(), FF::zero() });
        }
        return std::make_tuple(a_idx, b_idx);
    };
    /**
     * @brief A function that computes the largest cycle from the sigma permutation generated by the composer
     *
     */
    auto get_maximum_cycle = [](auto& builder, auto& composer) {
        // Compute the proving key for sigma polynomials
        auto instance = composer.create_instance(builder);
        auto prover = composer.create_prover(instance);
        auto proving_key = instance->proving_key;

        auto permutation_length = composer.NUM_WIRES * proving_key->circuit_size;
        auto sigma_polynomials = proving_key->get_sigma_polynomials();

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
            auto next_element_big =
                static_cast<uint256_t>(sigma_polynomials[i / proving_key->circuit_size][i % proving_key->circuit_size]);
            EXPECT_LE(next_element_big, uint256_t(UINT32_MAX));
            auto next_element = static_cast<size_t>(next_element_big.data[0]);
            size_t cycle_length = 1;
            visited_indices[i] = true;

            // Jump through the cycle untill we reach the start or the permutation length exceeds the possible
            // maximum
            while ((next_element != starting_element) && cycle_length < (permutation_length + 1)) {
                // Update cycle length and visited index infromation
                cycle_length++;
                visited_indices[next_element] = true;
                // Get next index
                next_element_big = static_cast<uint256_t>(sigma_polynomials[next_element / proving_key->circuit_size]
                                                                           [next_element % proving_key->circuit_size]);
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
    auto builder_no_assert_equal = CircuitBuilder();
    auto builder_with_assert_equal = CircuitBuilder();

    // Construct circuits
    create_simple_circuit(builder_no_assert_equal);
    auto assert_eq_params = create_simple_circuit(builder_with_assert_equal);

    // Use assert_equal on one of them
    builder_with_assert_equal.assert_equal(std::get<0>(assert_eq_params),
                                           std::get<1>(assert_eq_params),
                                           "Equality asssertion in standard honk composer test");

    // Check that the maximum cycle in the one, where we used assert_equal, is twice as long
    auto composer_no_assert_equal = Composer();
    auto composer_with_assert_equal = Composer();
    EXPECT_EQ(get_maximum_cycle(builder_with_assert_equal, composer_with_assert_equal),
              get_maximum_cycle(builder_no_assert_equal, composer_no_assert_equal) * 2);
}

TYPED_TEST(StandardHonkComposerTests, VerificationKeyCreation)
{
    TYPE_ALIASES

    // Create a composer and a dummy circuit with a few gates
    auto builder = CircuitBuilder();
    FF a = FF::one();
    uint32_t a_idx = builder.add_variable(a);
    FF b = FF::one();
    FF c = a + b;
    FF d = a + c;
    uint32_t b_idx = builder.add_variable(b);
    uint32_t c_idx = builder.add_variable(c);
    uint32_t d_idx = builder.add_variable(d);
    for (size_t i = 0; i < 16; i++) {
        builder.create_add_gate({ a_idx, b_idx, c_idx, FF::one(), FF::one(), FF::neg_one(), FF::zero() });
        builder.create_add_gate({ d_idx, c_idx, a_idx, FF::one(), FF::neg_one(), FF::neg_one(), FF::zero() });
    }

    auto composer = Composer();
    auto instance = composer.create_instance(builder);
    auto verification_key = instance->compute_verification_key();
    // There is nothing we can really check apart from the fact that constraint selectors and permutation selectors
    // were committed to, we simply check that the verification key now contains the appropriate number of
    // constraint and permutation selector commitments. This method should work with any future arithemtization.
    EXPECT_EQ(verification_key->size(), builder.selectors.size() + composer.NUM_WIRES * 2 + 2);
}

TYPED_TEST(StandardHonkComposerTests, BaseCase)
{
    TYPE_ALIASES
    auto builder = CircuitBuilder();
    FF a = 1;
    builder.add_variable(a);

    auto composer = Composer();
    auto instance = composer.create_instance(builder);
    auto prover = composer.create_prover(instance);
    auto proof = prover.construct_proof();
    auto verifier = composer.create_verifier(instance);
    bool verified = verifier.verify_proof(proof);
    ASSERT_TRUE(verified);
}

TYPED_TEST(StandardHonkComposerTests, TwoGates)
{
    TYPE_ALIASES
    auto run_test = [](bool expect_verified) {
        if constexpr (proof_system::IsGrumpkinFlavor<Flavor>) {
            barretenberg::srs::init_grumpkin_crs_factory("../srs_db/grumpkin");
        }
        auto builder = CircuitBuilder();
        // 1 + 1 - 2 = 0
        uint32_t w_l_1_idx;
        if (expect_verified) {
            w_l_1_idx = builder.add_variable(1);
        } else {
            w_l_1_idx = builder.add_variable(0);
        }
        uint32_t w_r_1_idx = builder.add_variable(1);
        uint32_t w_o_1_idx = builder.add_variable(2);
        builder.create_add_gate({ w_l_1_idx, w_r_1_idx, w_o_1_idx, 1, 1, -1, 0 });

        // 2 * 2 - 4 = 0
        uint32_t w_l_2_idx = builder.add_variable(2);
        uint32_t w_r_2_idx = builder.add_variable(2);
        uint32_t w_o_2_idx = builder.add_variable(4);
        builder.create_mul_gate({ w_l_2_idx, w_r_2_idx, w_o_2_idx, 1, -1, 0 });

        auto composer = Composer();
        auto instance = composer.create_instance(builder);
        auto prover = composer.create_prover(instance);
        auto proof = prover.construct_proof();

        auto verifier = composer.create_verifier(instance);
        bool verified = verifier.verify_proof(proof);

        EXPECT_EQ(verified, expect_verified);
    };

    run_test(/* expect_verified=*/true);
    run_test(/* expect_verified=*/false);
}

TYPED_TEST(StandardHonkComposerTests, SumcheckEvaluations)
{
    TYPE_ALIASES
    auto run_test = [](bool expected_result) {
        if constexpr (proof_system::IsGrumpkinFlavor<Flavor>) {
            barretenberg::srs::init_grumpkin_crs_factory("../srs_db/grumpkin");
        } else {
            barretenberg::srs::init_crs_factory("../srs_db/ignition");
        }
        auto builder = CircuitBuilder();
        FF a = FF::one();
        // Construct a small but non-trivial circuit
        uint32_t a_idx = builder.add_public_variable(a);
        FF b = FF::one();
        FF c = a + b;
        FF d = a + c;

        if (expected_result == false) {
            d += 1;
        };

        uint32_t b_idx = builder.add_variable(b);
        uint32_t c_idx = builder.add_variable(c);
        uint32_t d_idx = builder.add_variable(d);
        for (size_t i = 0; i < 4; i++) {
            builder.create_add_gate({ a_idx, b_idx, c_idx, FF::one(), FF::one(), FF::neg_one(), FF::zero() });
            builder.create_add_gate({ d_idx, c_idx, a_idx, FF::one(), FF::neg_one(), FF::neg_one(), FF::zero() });
        }

        auto composer = Composer();
        auto instance = composer.create_instance(builder);
        auto prover = composer.create_prover(instance);
        auto proof = prover.construct_proof();
        auto verifier = composer.create_verifier(instance);
        bool verified = verifier.verify_proof(proof);
        ASSERT_EQ(verified, expected_result);
    };
    run_test(/*expected_result=*/true);
    run_test(/*expected_result=*/false);
}
} // namespace test_standard_honk_composer
