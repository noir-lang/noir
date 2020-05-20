#include "plookup_tables.hpp"

namespace waffle {
namespace plookup {

using namespace barretenberg;

namespace {
static std::array<PLookupMultiTable, PLookupMultiTableId::NUM_MULTI_TABES> MULTI_TABLES;
static bool inited = false;

void init_multi_tables()
{
    MULTI_TABLES[PLookupMultiTableId::SHA256_CH_INPUT] =
        sha256_tables::get_choose_input_table(PLookupMultiTableId::SHA256_CH_INPUT);
    MULTI_TABLES[PLookupMultiTableId::SHA256_MAJ_INPUT] =
        sha256_tables::get_majority_input_table(PLookupMultiTableId::SHA256_MAJ_INPUT);
    MULTI_TABLES[PLookupMultiTableId::SHA256_WITNESS_INPUT] =
        sha256_tables::get_witness_extension_input_table(PLookupMultiTableId::SHA256_WITNESS_INPUT);
    MULTI_TABLES[PLookupMultiTableId::SHA256_CH_OUTPUT] =
        sha256_tables::get_choose_output_table(PLookupMultiTableId::SHA256_CH_OUTPUT);
    MULTI_TABLES[PLookupMultiTableId::SHA256_MAJ_OUTPUT] =
        sha256_tables::get_majority_output_table(PLookupMultiTableId::SHA256_MAJ_OUTPUT);
    MULTI_TABLES[PLookupMultiTableId::SHA256_WITNESS_OUTPUT] =
        sha256_tables::get_witness_extension_output_table(PLookupMultiTableId::SHA256_WITNESS_OUTPUT);
    MULTI_TABLES[PLookupMultiTableId::AES_NORMALIZE] =
        aes128_tables::get_aes_normalization_table(PLookupMultiTableId::AES_NORMALIZE);
    MULTI_TABLES[PLookupMultiTableId::AES_INPUT] = aes128_tables::get_aes_input_table(PLookupMultiTableId::AES_INPUT);
    MULTI_TABLES[PLookupMultiTableId::AES_SBOX] = aes128_tables::get_aes_sbox_table(PLookupMultiTableId::AES_SBOX);
    MULTI_TABLES[PLookupMultiTableId::PEDERSEN_1] =
        pedersen_tables::generate_pedersen_multi_table(PLookupMultiTableId::PEDERSEN_1);
    MULTI_TABLES[PLookupMultiTableId::PEDERSEN_2] =
        pedersen_tables::generate_pedersen_multi_table(PLookupMultiTableId::PEDERSEN_2);
}
} // namespace

const PLookupMultiTable& create_table(const PLookupMultiTableId id)
{
    if (!inited) {
        init_multi_tables();
        inited = true;
    }
    return MULTI_TABLES[id];
}

PLookupReadData get_wnaf_table_values(const PLookupMultiTableId id, const fr& key)
{
    const auto& multi_table = create_table(id);

    const size_t num_lookups = multi_table.lookup_ids.size();

    PLookupReadData result;

    constexpr size_t num_wnaf_bits = 13;
    constexpr uint64_t bits_per_scalar = 128;
    constexpr uint64_t num_wnaf_entries = (bits_per_scalar + num_wnaf_bits - 1) / num_wnaf_bits;
    uint64_t wnaf_entries[num_wnaf_entries] = { 0 };

    fr scalar_multiplier = key.from_montgomery_form();
    bool skew = false;
    barretenberg::wnaf::fixed_wnaf<128, 1, num_wnaf_bits>(&scalar_multiplier.data[0], &wnaf_entries[0], skew, 0);

    const auto keys = numeric::slice_input(key, multi_table.slice_sizes);

    const size_t generator_index = (id == PLookupMultiTableId::PEDERSEN_1) ? 0 : 1;
    std::vector<fr> column_1_raw_values;
    std::vector<fr> column_2_raw_values;
    std::vector<fr> column_3_raw_values;

    for (size_t i = 0; i < num_lookups; ++i) {
        uint64_t raw_value = wnaf_entries[i] & 0xffffff;
        uint64_t negative = (wnaf_entries[i] >> 31) & 0x01;
        int scalar = static_cast<int>(2 * raw_value + 1) * (1 - 2 * static_cast<int>(negative));
        fr key(scalar);
        grumpkin::g1::element value;
        if (i == 0) {
            value = pedersen_tables::get_generator_value(generator_index, i, wnaf_entries[i]);
        } else {
            value = pedersen_tables::get_skew_generator_value(generator_index, wnaf_entries[i], skew);
        }
        column_1_raw_values.emplace_back(key);
        column_2_raw_values.emplace_back(value.x);
        column_3_raw_values.emplace_back(value.y);

        const PLookupBasicTable::KeyEntry key_entry{ { key, 0 }, { value.x, value.y } };
        result.key_entries.emplace_back(key_entry);
    }

    result.column_1_accumulator_values.resize(num_lookups);
    result.column_2_accumulator_values.resize(num_lookups);
    result.column_3_accumulator_values.resize(num_lookups);

    result.column_1_accumulator_values[num_lookups - 1] = column_1_raw_values[num_lookups - 1];
    result.column_2_accumulator_values[num_lookups - 1] = column_2_raw_values[num_lookups - 1];
    result.column_3_accumulator_values[num_lookups - 1] = column_3_raw_values[num_lookups - 1];

    for (size_t i = 1; i < num_lookups; ++i) {
        const auto& previous_1 = result.column_1_accumulator_values[num_lookups - i];
        const auto& previous_2 = result.column_2_accumulator_values[num_lookups - i];
        const auto& previous_3 = result.column_3_accumulator_values[num_lookups - i];

        auto& current_1 = result.column_1_accumulator_values[num_lookups - 1 - i];
        auto& current_2 = result.column_2_accumulator_values[num_lookups - 1 - i];
        auto& current_3 = result.column_3_accumulator_values[num_lookups - 1 - i];

        const auto& raw_1 = column_1_raw_values[num_lookups - 1 - i];
        const auto& raw_2 = column_2_raw_values[num_lookups - 1 - i];
        const auto& raw_3 = column_3_raw_values[num_lookups - 1 - i];

        current_1 = raw_1 + previous_1 * multi_table.column_1_step_sizes[num_lookups - i];
        current_2 = raw_2 + previous_2 * multi_table.column_2_step_sizes[num_lookups - i];
        current_3 = raw_3 + previous_3 * multi_table.column_3_step_sizes[num_lookups - i];
    }
    return result;
}

PLookupReadData get_table_values(const PLookupMultiTableId id, const fr& key)
{
    if (id == PLookupMultiTableId::PEDERSEN_1 || id == PLookupMultiTableId::PEDERSEN_2) {
        return get_wnaf_table_values(id, key);
    }
    const auto& multi_table = create_table(id);

    const size_t num_lookups = multi_table.lookup_ids.size();

    PLookupReadData result;

    const auto keys = numeric::slice_input(key, multi_table.slice_sizes);

    std::vector<fr> column_1_raw_values;
    std::vector<fr> column_2_raw_values;
    std::vector<fr> column_3_raw_values;

    for (size_t i = 0; i < num_lookups; ++i) {
        const auto values = multi_table.get_table_values[i]({ keys[i], 0 });
        column_1_raw_values.emplace_back(keys[i]);
        column_2_raw_values.emplace_back(values[0]);
        column_3_raw_values.emplace_back(values[1]);

        const PLookupBasicTable::KeyEntry key_entry{ { keys[i], 0 }, values };
        result.key_entries.emplace_back(key_entry);
    }
    result.column_1_accumulator_values.resize(num_lookups);
    result.column_2_accumulator_values.resize(num_lookups);
    result.column_3_accumulator_values.resize(num_lookups);

    result.column_1_accumulator_values[num_lookups - 1] = column_1_raw_values[num_lookups - 1];
    result.column_2_accumulator_values[num_lookups - 1] = column_2_raw_values[num_lookups - 1];
    result.column_3_accumulator_values[num_lookups - 1] = column_3_raw_values[num_lookups - 1];

    for (size_t i = 1; i < num_lookups; ++i) {
        const auto& previous_1 = result.column_1_accumulator_values[num_lookups - i];
        const auto& previous_2 = result.column_2_accumulator_values[num_lookups - i];
        const auto& previous_3 = result.column_3_accumulator_values[num_lookups - i];

        auto& current_1 = result.column_1_accumulator_values[num_lookups - 1 - i];
        auto& current_2 = result.column_2_accumulator_values[num_lookups - 1 - i];
        auto& current_3 = result.column_3_accumulator_values[num_lookups - 1 - i];

        const auto& raw_1 = column_1_raw_values[num_lookups - 1 - i];
        const auto& raw_2 = column_2_raw_values[num_lookups - 1 - i];
        const auto& raw_3 = column_3_raw_values[num_lookups - 1 - i];

        current_1 = raw_1 + previous_1 * multi_table.column_1_step_sizes[num_lookups - i];
        current_2 = raw_2 + previous_2 * multi_table.column_2_step_sizes[num_lookups - i];
        current_3 = raw_3 + previous_3 * multi_table.column_3_step_sizes[num_lookups - i];
    }
    return result;
}

} // namespace plookup
} // namespace waffle