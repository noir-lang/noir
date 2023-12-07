#include "toy_avm_circuit_builder.hpp"
#include "barretenberg/crypto/generators/generator_data.hpp"
#include <gtest/gtest.h>

using namespace barretenberg;

namespace {
auto& engine = numeric::random::get_debug_engine();
}

namespace toy_avm_circuit_builder_tests {

/**
 * @brief A test explaining the work of the permutations in Toy AVM
 *
 */
TEST(ToyAVMCircuitBuilder, BaseCase)
{

    using FF = proof_system::honk::flavor::ToyAVM::FF;
    const size_t circuit_size = 16;
    proof_system::ToyAVMCircuitBuilder<proof_system::honk::flavor::ToyAVM> circuit_builder;

    // Sample 2*16 random elements for the tuple permutation example
    std::vector<FF> column_0;
    std::vector<FF> column_1;
    for (size_t i = 0; i < circuit_size; i++) {
        column_0.emplace_back(FF::random_element());
        column_1.emplace_back(FF::random_element());
    }

    // Sample 8 random elements for the single column permutation
    std::vector<FF> column_2;
    for (size_t i = 0; i < circuit_size / 2; i++) {
        column_2.emplace_back(FF::random_element());
    }

    for (size_t i = 0; i < circuit_size; i++) {
        // We put the same tuple of values in the first 2 wires and in the next 2 to at different rows
        // We also put the same value in the self_permutation column in 2 consecutive rows
        circuit_builder.add_row({ column_0[i], column_1[i], column_0[15 - i], column_1[15 - i], column_2[i / 2] });
    }

    // Test that permutations with correct values work
    bool result = circuit_builder.check_circuit();
    EXPECT_EQ(result, true);

    // Store value temporarily
    FF tmp = circuit_builder.wires[0][5];

    // Replace one of the values in a tuple permutation column with a random one, breaking the permutation
    circuit_builder.wires[0][5] = FF::random_element();

    // Check that it fails
    result = circuit_builder.check_circuit();
    EXPECT_EQ(result, false);

    // Restore value
    circuit_builder.wires[0][5] = tmp;

    // Check circuit passes
    result = circuit_builder.check_circuit();
    EXPECT_EQ(result, true);

    // Break single-column permutation
    circuit_builder.wires[circuit_builder.wires.size() - 1][0] = FF::random_element();
    result = circuit_builder.check_circuit();
    EXPECT_EQ(result, false);
}
} // namespace toy_avm_circuit_builder_tests