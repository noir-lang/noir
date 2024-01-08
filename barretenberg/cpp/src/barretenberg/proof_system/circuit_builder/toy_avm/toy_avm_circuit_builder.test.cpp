#include "../generated/Toy_circuit_builder.hpp"
#include "barretenberg/crypto/generators/generator_data.hpp"
#include "barretenberg/flavor/generated/Toy_flavor.hpp"
#include "barretenberg/proof_system/circuit_builder/generated/Toy_circuit_builder.hpp"

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

    using FF = proof_system::honk::flavor::ToyFlavor::FF;
    using Builder = proof_system::ToyCircuitBuilder;
    using Row = Builder::Row;
    Builder circuit_builder;

    const size_t circuit_size = 16;
    std::vector<Row> rows;

    // Sample 2*16 random elements for the tuple permutation example
    for (size_t i = 0; i < circuit_size; i++) {
        Row row{
            .toy_q_tuple_set = 1,
            .toy_set_1_column_1 = FF::random_element(),
            .toy_set_1_column_2 = FF::random_element(),
        };
        rows.push_back(row);
    }

    for (size_t i = 0; i < circuit_size; i++) {
        // We put the same tuple of values in the first 2 wires and in the next 2 to at different rows
        // We also put the same value in the self_permutation column in 2 consecutive rows
        Row& front_row = rows[i];
        Row& back_row = rows[circuit_size - (i + 1)];

        back_row.toy_set_2_column_1 = front_row.toy_set_1_column_1;
        back_row.toy_set_2_column_2 = front_row.toy_set_1_column_2;
    }

    // Test that permutations with correct values work
    circuit_builder.set_trace(std::move(rows));
    bool result = circuit_builder.check_circuit();
    EXPECT_EQ(result, true);

    // Store value temporarily
    FF tmp = circuit_builder.rows[5].toy_set_1_column_1;

    // Replace one of the values in a tuple permutation column with a random one, breaking the permutation
    circuit_builder.rows[5].toy_set_1_column_1 = FF::random_element();

    // Check that it fails
    result = circuit_builder.check_circuit();
    EXPECT_EQ(result, false);

    // Restore value
    circuit_builder.rows[5].toy_set_1_column_1 = tmp;

    // Check circuit passes
    result = circuit_builder.check_circuit();
    EXPECT_EQ(result, true);
}
} // namespace toy_avm_circuit_builder_tests