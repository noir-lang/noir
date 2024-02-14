#include "../generated/toy_circuit_builder.hpp"
#include "barretenberg/crypto/generators/generator_data.hpp"
#include "barretenberg/flavor/generated/toy_flavor.hpp"
#include "barretenberg/proof_system/circuit_builder/generated/toy_circuit_builder.hpp"

#include <gtest/gtest.h>

using namespace bb;

namespace {
auto& engine = numeric::get_debug_randomness();
}

/**
 * @brief A test explaining the work of the permutations in Toy AVM
 *
 */
TEST(ToyAVMCircuitBuilder, BaseCase)
{
    using FF = ToyFlavor::FF;
    using Builder = ToyCircuitBuilder;
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

/**
 * @brief Investigate circuit builder / proving issue
 *
 */
TEST(ToyAVMCircuitBuilder, MultiLookup)
{
    using FF = ToyFlavor::FF;
    using Builder = ToyCircuitBuilder;
    using Row = Builder::Row;
    Builder circuit_builder;

    const size_t circuit_size = 16;
    std::vector<Row> rows;
    // init empty rows
    for (size_t i = 0; i < circuit_size; i++) {
        Row row{};
        rows.push_back(row);
    }

    // LOOKUPS
    // Create clk mem access lookup table;
    // We only want to turn on the mem write when clk is 1
    Row& row_1 = rows[0];
    row_1.toy_q_err = FF(1);
    row_1.toy_clk = FF(1);
    // Below we lookup two occurances, so our counts is 2
    row_1.lookup_err_counts = FF(2);

    // Set the mem read on two different rows, we will then lookup into the clk
    row_1.toy_m_clk = FF(1);
    row_1.toy_q_err_check = FF(1);

    Row& row_3 = rows[2];
    row_3.toy_m_clk = FF(1);
    row_3.toy_q_err_check = FF(1);

    // Check circuit passes
    circuit_builder.set_trace(std::move(rows));
    EXPECT_EQ(circuit_builder.check_circuit(), true);

    // Turn off row_3 lookup selector, expect failure
    circuit_builder.rows[2].toy_m_clk = FF(0);
    EXPECT_EQ(circuit_builder.check_circuit(), false);
}

TEST(ToyAVMCircuitBuilder, EmptyLookups)
{
    using Builder = ToyCircuitBuilder;
    using Row = Builder::Row;
    Builder circuit_builder;

    const size_t circuit_size = 16;
    std::vector<Row> rows;
    // init empty rows
    for (size_t i = 0; i < circuit_size; i++) {
        Row row{};
        rows.push_back(row);
    }

    circuit_builder.set_trace(std::move(rows));
    EXPECT_EQ(circuit_builder.check_circuit(), true);
}

TEST(ToyAVMCircuitBuilder, SparsePermutation)
{
    // Test sparse permutation, where the permutation check is not active on all rows
    using FF = ToyFlavor::FF;
    using Builder = ToyCircuitBuilder;
    using Row = Builder::Row;
    Builder circuit_builder;

    const size_t circuit_size = 16;
    std::vector<Row> rows;
    // init empty rows
    for (size_t i = 0; i < circuit_size; i++) {
        Row row{};
        rows.push_back(row);
    }

    // Activate lhs on row 1
    Row& row_1 = rows[0];
    row_1.toy_sparse_lhs = FF(1);
    row_1.toy_sparse_column_1 = FF(420);

    // Activate rhs on row 5
    Row& row_5 = rows[4];
    row_5.toy_sparse_rhs = FF(1);
    row_5.toy_sparse_column_2 = FF(420);

    circuit_builder.set_trace(std::move(rows));
    EXPECT_EQ(circuit_builder.check_circuit(), true);

    // Expect it to break after changing row5
    circuit_builder.rows[4].toy_sparse_column_2 = FF(421);
    EXPECT_EQ(circuit_builder.check_circuit(), false);
}