
#include <gtest/gtest.h>

#include "barretenberg/numeric/random/engine.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"
#include "rom_table.hpp"

namespace test_stdlib_rom_array {
using namespace bb;
using namespace bb::plonk;

// Defining ultra-specific types for local testing.
using Builder = bb::UltraCircuitBuilder;
using field_ct = stdlib::field_t<Builder>;
using witness_ct = stdlib::witness_t<Builder>;
using rom_table_ct = stdlib::rom_table<Builder>;

namespace {
auto& engine = numeric::random::get_debug_engine();
}

TEST(rom_table, rom_table_read_write_consistency)
{
    Builder builder;

    std::vector<field_ct> table_values;
    const size_t table_size = 10;
    for (size_t i = 0; i < table_size; ++i) {
        table_values.emplace_back(witness_ct(&builder, bb::fr::random_element()));
    }

    rom_table_ct table(table_values);

    field_ct result(0);
    bb::fr expected(0);

    for (size_t i = 0; i < 10; ++i) {
        field_ct index(witness_ct(&builder, (uint64_t)i));

        if (i % 2 == 0) {
            const auto before_n = builder.num_gates;
            const auto to_add = table[index];
            const auto after_n = builder.num_gates;
            // should cost 1 gates (the ROM read adds 1 extra gate when the proving key is constructed)
            // (but not for 1st entry, the 1st ROM read also builts the ROM table, which will cost table_size * 2 gates)
            if (i != 0) {
                EXPECT_EQ(after_n - before_n, 1ULL);
            }
            result += to_add; // variable lookup
        } else {
            const auto before_n = builder.num_gates;
            const auto to_add = table[i]; // constant lookup
            const auto after_n = builder.num_gates;
            // should cost 0 gates. Constant lookups are free
            EXPECT_EQ(after_n - before_n, 0ULL);
            result += to_add;
        }
        expected += table_values[i].get_value();
    }

    EXPECT_EQ(result.get_value(), expected);

    bool verified = builder.check_circuit();
    EXPECT_EQ(verified, true);
}

} // namespace test_stdlib_rom_array
