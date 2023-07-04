#include <gtest/gtest.h>

#include "barretenberg/numeric/random/engine.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"
#include "ram_table.hpp"

namespace test_stdlib_ram_table {

using namespace proof_system::plonk;
// Defining ultra-specific types for local testing.
using Composer = proof_system::UltraCircuitBuilder;
using field_ct = stdlib::field_t<Composer>;
using witness_ct = stdlib::witness_t<Composer>;
using ram_table_ct = stdlib::ram_table<Composer>;

namespace {
auto& engine = numeric::random::get_debug_engine();
}

TEST(ram_table, ram_table_init_read_consistency)
{
    Composer composer;

    std::vector<field_ct> table_values;
    const size_t table_size = 10;
    for (size_t i = 0; i < table_size; ++i) {
        table_values.emplace_back(witness_ct(&composer, barretenberg::fr::random_element()));
    }

    ram_table_ct table(table_values);

    field_ct result(0);
    barretenberg::fr expected(0);

    for (size_t i = 0; i < 10; ++i) {
        field_ct index(witness_ct(&composer, (uint64_t)i));

        if (i % 2 == 0) {
            const auto to_add = table.read(index);
            result += to_add; // variable lookup
        } else {
            const auto to_add = table.read(i); // constant lookup
            result += to_add;
        }
        expected += table_values[i].get_value();
    }

    EXPECT_EQ(result.get_value(), expected);

    bool verified = composer.check_circuit();
    EXPECT_EQ(verified, true);
}

TEST(ram_table, ram_table_read_write_consistency)
{
    Composer composer;
    const size_t table_size = 10;

    std::vector<barretenberg::fr> table_values(table_size);

    ram_table_ct table(&composer, table_size);

    for (size_t i = 0; i < table_size; ++i) {
        table.write(i, 0);
    }
    field_ct result(0);
    barretenberg::fr expected(0);

    const auto update = [&]() {
        for (size_t i = 0; i < table_size / 2; ++i) {
            table_values[2 * i] = barretenberg::fr::random_element();
            table_values[2 * i + 1] = barretenberg::fr::random_element();

            // init with both constant and variable values
            table.write(2 * i, table_values[2 * i]);
            table.write(2 * i + 1, witness_ct(&composer, table_values[2 * i + 1]));
        }
    };

    const auto read = [&]() {
        for (size_t i = 0; i < table_size / 2; ++i) {
            const size_t index = table_size - 2 - (i * 2); // access in something other than basic incremental order

            result += table.read(witness_ct(&composer, index));
            result += table.read(index + 1);

            expected += table_values[index];
            expected += table_values[index + 1];
        }
    };

    update();
    read();
    update();
    read();
    update();

    EXPECT_EQ(result.get_value(), expected);

    bool verified = composer.check_circuit();
    EXPECT_EQ(verified, true);
}
} // namespace test_stdlib_ram_table