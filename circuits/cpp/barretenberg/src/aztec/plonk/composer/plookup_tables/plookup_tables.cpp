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
}
} // namespace

const PLookupMultiTable& get_multi_table(const PLookupMultiTableId id)
{
    if (!inited) {
        init_multi_tables();
        inited = true;
    }
    return MULTI_TABLES[id];
}

PLookupReadData get_multi_table_values(const PLookupMultiTableId id, const fr& key)
{
    const auto& multi_table = get_multi_table(id);

    const size_t num_lookups = multi_table.lookup_ids.size();

    PLookupReadData result;

    const auto keys = numeric::slice_input(key, multi_table.slice_sizes);

    std::vector<fr> column_1_raw_values;
    std::vector<fr> column_2_raw_values;
    std::vector<fr> column_3_raw_values;

    for (size_t i = 0; i < num_lookups; ++i) {
        // PLookupTable& table = get_table(multi_table.lookup_ids[i]);

        const auto values = multi_table.get_table_values[i]({ keys[i], 0 });
        column_1_raw_values.emplace_back(keys[i]);
        column_2_raw_values.emplace_back(values[0]);
        column_3_raw_values.emplace_back(values[1]);

        const PLookupTable::KeyEntry key_entry{ { keys[i], 0 }, values };
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