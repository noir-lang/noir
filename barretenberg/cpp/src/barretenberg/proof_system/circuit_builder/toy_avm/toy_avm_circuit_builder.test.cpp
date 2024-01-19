#include "../generated/Toy_circuit_builder.hpp"
#include "barretenberg/crypto/generators/generator_data.hpp"
#include "barretenberg/flavor/generated/Toy_flavor.hpp"
#include "barretenberg/proof_system/circuit_builder/generated/Toy_circuit_builder.hpp"

#include <gtest/gtest.h>

using namespace bb;

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

    using FF = bb::honk::flavor::ToyFlavor::FF;
    using Builder = bb::ToyCircuitBuilder;
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
    EXPECT_EQ(circuit_builder.check_circuit(), true);

    // Store value temporarily
    FF tmp = circuit_builder.rows[5].toy_set_1_column_1;

    // Replace one of the values in a tuple permutation column with a random one, breaking the permutation
    circuit_builder.rows[5].toy_set_1_column_1 = FF::random_element();

    // Check that it fails
    EXPECT_EQ(circuit_builder.check_circuit(), false);

    // Restore value
    circuit_builder.rows[5].toy_set_1_column_1 = tmp;

    // Check circuit passes
    EXPECT_EQ(circuit_builder.check_circuit(), true);

    // LOOKUPS
    // Create xor lookup table, from 0 to 16;
    for (size_t i = 0; i < circuit_size; i++) {
        Row& row = circuit_builder.rows[i];
        size_t a = i;
        size_t b = circuit_size - i;
        size_t c = a ^ b;

        row.toy_q_xor_table = FF(1);
        row.toy_table_xor_a = FF(a);
        row.toy_table_xor_b = FF(b);
        row.toy_table_xor_c = FF(c);
    }

    // Perform a lookup every other row
    for (size_t i = 0; i < circuit_size; i += 2) {
        Row& row = circuit_builder.rows[i];
        size_t a = i;
        size_t b = circuit_size - i;
        size_t c = a ^ b;

        row.toy_q_xor = FF(1);
        row.toy_xor_a = FF(a);
        row.toy_xor_b = FF(b);
        row.toy_xor_c = FF(c);

        // Add a count for this row
        row.lookup_xor_counts = FF(1);
    }

    // Check circuit passes
    EXPECT_EQ(circuit_builder.check_circuit(), true);

    // Break lookup by changing count
    tmp = circuit_builder.rows[5].lookup_xor_counts;
    circuit_builder.rows[5].lookup_xor_counts = FF::random_element();

    EXPECT_EQ(circuit_builder.check_circuit(), false);

    circuit_builder.rows[5].lookup_xor_counts = tmp;
    EXPECT_EQ(circuit_builder.check_circuit(), true);

    // Break lookup by changing lookup value
    tmp = circuit_builder.rows[2].toy_xor_a;
    circuit_builder.rows[2].toy_xor_a = FF::random_element();

    EXPECT_EQ(circuit_builder.check_circuit(), false);

    circuit_builder.rows[2].toy_xor_a = tmp;
    EXPECT_EQ(circuit_builder.check_circuit(), true);
}
} // namespace toy_avm_circuit_builder_tests