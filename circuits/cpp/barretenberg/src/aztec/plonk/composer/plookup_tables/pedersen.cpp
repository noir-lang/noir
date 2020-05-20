#include "./pedersen.hpp"

#include <crypto/pedersen/pedersen.hpp>

namespace waffle {

namespace pedersen_tables {

namespace {
static std::array<std::vector<grumpkin::g1::element>, NUM_LOOKUPS_PER_HASH> pedersen_generator_table_1;
static std::array<std::vector<grumpkin::g1::element>, NUM_LOOKUPS_PER_HASH> pedersen_generator_table_2;
static bool initialized_generator_tables = false;

void initialize_generator_table(std::vector<grumpkin::g1::element>& generator_table,
                                const grumpkin::g1::element& generator,
                                const size_t table_size)
{
    typedef grumpkin::g1::element element;

    const element d1 = generator.dbl();

    const size_t midpoint = table_size / 2;

    generator_table.resize(table_size);

    generator_table[midpoint] = generator;
    for (size_t i = midpoint + 1; i < table_size; ++i) {
        generator_table[i] = generator_table[i - 1] + d1;
    }

    for (size_t i = 0; i < midpoint; ++i) {
        generator_table[i] = -generator_table[table_size - 1 - i];
    }
    grumpkin::g1::element::batch_normalize(&generator_table[0], table_size);
}

void initialize_skew_table(std::vector<grumpkin::g1::element>& skew_table,
                           const grumpkin::g1::element& generator,
                           const grumpkin::g1::element& skew)
{
    typedef grumpkin::g1::element element;

    // the most significant wnaf slice cannot be negative, but is 1 larger than the number of remaining bits in the
    // scalar (as the previous wnaf entry could have generated a carry that is propagated to this slice)

    // we also want to deal with the scalar multiplier's 'skew' in this slice - whether the scalar is even or odd.
    // This either requires an explicit lookup for a single bit, or will double the size of one of the wnaf lookup
    // tables. As the msb wnaf is smaller than the rest it is natural to aggregate the skew into this table. We also
    // cannot incorporate skew into the least significant wnaf table, as this would generate a point at infinity for one
    // of the table entries (1 - skew = 0)
    const size_t num_end_bits = (BITS_PER_SCALAR_MULTIPLIER % BITS_PER_LOOKUP) + 1;
    const size_t table_size = (1 << num_end_bits) * 2;

    const element d1 = generator.dbl();

    skew_table.resize(table_size);
    skew_table[0] = generator - skew;
    skew_table[1] = generator;

    for (size_t i = 2; i < table_size; i += 2) {
        skew_table[i] = skew_table[i - 2] + d1;
        skew_table[i + 1] = skew_table[i - 1] + d1;
    }
    grumpkin::g1::element::batch_normalize(&skew_table[0], table_size);
}

void initialize_generator_tables()
{
    if (initialized_generator_tables) {
        return;
    }
    const size_t basic_table_size = (1 << BITS_PER_LOOKUP);

    grumpkin::g1::element generator1(crypto::pedersen::get_generator(0));
    grumpkin::g1::element generator2(crypto::pedersen::get_generator(1));

    grumpkin::g1::element skew_generator1(crypto::pedersen::get_generator(0));
    grumpkin::g1::element skew_generator2(crypto::pedersen::get_generator(1));

    for (size_t i = 0; i < NUM_LOOKUPS_PER_HASH - 1; ++i) {

        initialize_generator_table(
            pedersen_generator_table_1[NUM_LOOKUPS_PER_HASH - 1 - i], generator1, basic_table_size);
        initialize_generator_table(
            pedersen_generator_table_2[NUM_LOOKUPS_PER_HASH - 1 - i], generator2, basic_table_size);
        for (size_t j = 0; j < BITS_PER_LOOKUP; ++j) {
            generator1 = generator1.dbl();
            generator2 = generator2.dbl();
        }
    }

    initialize_skew_table(pedersen_generator_table_1[0], generator1, skew_generator1);
    initialize_skew_table(pedersen_generator_table_2[0], generator2, skew_generator2);
    initialized_generator_tables = true;
}
} // namespace

grumpkin::g1::affine_element get_generator_value(const size_t generator_index,
                                                 const size_t lookup_index,
                                                 const size_t wnaf_value)
{
    initialize_generator_tables();

    uint64_t raw_value = wnaf_value & 0xffffff;
    uint64_t negative = (wnaf_value >> 31) & 0x01;
    uint64_t midpoint = pedersen_generator_table_1[lookup_index].size() / 2;
    size_t index;
    if (negative) {
        index = midpoint - 1 - raw_value;
    } else {
        index = midpoint + raw_value;
    }
    ASSERT(index < pedersen_generator_table_1[lookup_index].size());
    switch (generator_index) {
    case 0: {
        return pedersen_generator_table_1[lookup_index][index];
    }
    case 1: {
        return pedersen_generator_table_2[lookup_index][index];
    }
    default: {
        throw;
    }
    }
}

grumpkin::g1::affine_element get_skew_generator_value(const size_t generator_index,
                                                      const size_t wnaf_value,
                                                      const bool skew)
{
    initialize_generator_tables();

    uint64_t raw_value = wnaf_value & 0xffffff;
    size_t index = raw_value * 2 + !skew;

    ASSERT(index < pedersen_generator_table_1[0].size());
    switch (generator_index) {
    case 0: {
        return pedersen_generator_table_1[0][index];
    }
    case 1: {
        return pedersen_generator_table_2[0][index];
    }
    default: {
        throw;
    }
    }
}

inline std::array<barretenberg::fr, 2> get_values_from_key_stub(const std::array<uint64_t, 2>)
{
    return { 0, 0 };
}

PLookupBasicTable generate_pedersen_table(const size_t generator_index,
                                          const size_t slice_index,
                                          PLookupBasicTableId id,
                                          const size_t table_index)
{
    initialize_generator_tables();

    PLookupBasicTable table;
    table.id = id;
    table.table_index = table_index;
    if (slice_index == 0) {
        table.size = (1U << ((BITS_PER_SCALAR_MULTIPLIER % BITS_PER_LOOKUP) + 1)) * 2;
    } else {
        table.size = (1U << BITS_PER_LOOKUP);
    }
    table.use_twin_keys = false;

    table.size = pedersen_generator_table_1[slice_index].size();
    uint64_t midpoint = table.size / 2;

    if (slice_index != 0) {
        for (uint64_t i = 0; i < midpoint; ++i) {
            uint64_t wnaf_value = i + (1ULL << 31ULL);
            int scalar_value = -static_cast<int>(i * 2 + 1);
            table.column_1.emplace_back(scalar_value);
            const auto value = get_generator_value(generator_index, slice_index, wnaf_value);
            table.column_2.emplace_back(value.x);
            table.column_3.emplace_back(value.y);
        }

        for (uint64_t i = midpoint; i < table.size; ++i) {
            uint64_t wnaf_value = i - midpoint;
            int scalar_value = static_cast<int>(i * 2 + 1);
            table.column_1.emplace_back(scalar_value);
            const auto value = get_generator_value(generator_index, slice_index, wnaf_value);
            table.column_2.emplace_back(value.x);
            table.column_3.emplace_back(value.y);
        }
    } else {
        barretenberg::fr skew_factor = -barretenberg::fr(uint256_t(1) << BITS_PER_SCALAR_MULTIPLIER).invert();
        for (uint64_t i = 0; i < table.size; i += 2) {
            uint64_t wnaf_value = i / 2;
            int scalar_value = -static_cast<int>(i * 2 + 1);
            table.column_1.emplace_back(skew_factor + scalar_value);
            auto value = get_skew_generator_value(generator_index, wnaf_value, true);
            table.column_2.emplace_back(value.x);
            table.column_3.emplace_back(value.y);
            table.column_1.emplace_back(skew_factor);
            value = get_skew_generator_value(generator_index, wnaf_value, false);
            table.column_2.emplace_back(value.x);
            table.column_3.emplace_back(value.y);
        }
    }

    table.get_values_from_key = &get_values_from_key_stub;

    table.column_1_step_size = barretenberg::fr(uint256_t(1) << BITS_PER_SCALAR_MULTIPLIER);
    table.column_2_step_size = 0;
    table.column_3_step_size = 0;

    return table;
}

PLookupMultiTable generate_pedersen_multi_table(const PLookupMultiTableId id)
{
    const size_t num_entries = BITS_PER_LOOKUP;

    PLookupMultiTable table(1 << BITS_PER_LOOKUP, 0, 0, num_entries);

    table.id = id;
    table.slice_sizes = { (1 << BITS_PER_LOOKUP), (1 << BITS_PER_LOOKUP), (1 << BITS_PER_LOOKUP),
                          (1 << BITS_PER_LOOKUP), (1 << BITS_PER_LOOKUP), (1 << BITS_PER_LOOKUP),
                          (1 << BITS_PER_LOOKUP), (1 << BITS_PER_LOOKUP), (1 << BITS_PER_LOOKUP),
                          (1 << BITS_PER_LOOKUP) };
    table.get_table_values = { &get_values_from_key_stub, &get_values_from_key_stub, &get_values_from_key_stub,
                               &get_values_from_key_stub, &get_values_from_key_stub, &get_values_from_key_stub,
                               &get_values_from_key_stub, &get_values_from_key_stub, &get_values_from_key_stub,
                               &get_values_from_key_stub };

    if (id == PEDERSEN_1) {
        table.lookup_ids = { PEDERSEN_1_1, PEDERSEN_1_2, PEDERSEN_1_3, PEDERSEN_1_4, PEDERSEN_1_5,
                             PEDERSEN_1_6, PEDERSEN_1_7, PEDERSEN_1_8, PEDERSEN_1_9, PEDERSEN_1_10 };
    } else {
        table.lookup_ids = { PEDERSEN_2_1, PEDERSEN_2_2, PEDERSEN_2_3, PEDERSEN_2_4, PEDERSEN_2_5,
                             PEDERSEN_2_6, PEDERSEN_2_7, PEDERSEN_2_8, PEDERSEN_2_9, PEDERSEN_2_10 };
    }
    return table;
}
} // namespace pedersen_tables
} // namespace waffle
