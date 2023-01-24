#include "plookup_tables.hpp"

namespace waffle {
namespace plookup {

using namespace barretenberg;

namespace {
static std::array<PlookupMultiTable, PlookupMultiTableId::NUM_MULTI_TABLES> MULTI_TABLES;
static bool inited = false;

void init_multi_tables()
{
    MULTI_TABLES[PlookupMultiTableId::SHA256_CH_INPUT] =
        sha256_tables::get_choose_input_table(PlookupMultiTableId::SHA256_CH_INPUT);
    MULTI_TABLES[PlookupMultiTableId::SHA256_MAJ_INPUT] =
        sha256_tables::get_majority_input_table(PlookupMultiTableId::SHA256_MAJ_INPUT);
    MULTI_TABLES[PlookupMultiTableId::SHA256_WITNESS_INPUT] =
        sha256_tables::get_witness_extension_input_table(PlookupMultiTableId::SHA256_WITNESS_INPUT);
    MULTI_TABLES[PlookupMultiTableId::SHA256_CH_OUTPUT] =
        sha256_tables::get_choose_output_table(PlookupMultiTableId::SHA256_CH_OUTPUT);
    MULTI_TABLES[PlookupMultiTableId::SHA256_MAJ_OUTPUT] =
        sha256_tables::get_majority_output_table(PlookupMultiTableId::SHA256_MAJ_OUTPUT);
    MULTI_TABLES[PlookupMultiTableId::SHA256_WITNESS_OUTPUT] =
        sha256_tables::get_witness_extension_output_table(PlookupMultiTableId::SHA256_WITNESS_OUTPUT);
    MULTI_TABLES[PlookupMultiTableId::AES_NORMALIZE] =
        aes128_tables::get_aes_normalization_table(PlookupMultiTableId::AES_NORMALIZE);
    MULTI_TABLES[PlookupMultiTableId::AES_INPUT] = aes128_tables::get_aes_input_table(PlookupMultiTableId::AES_INPUT);
    MULTI_TABLES[PlookupMultiTableId::AES_SBOX] = aes128_tables::get_aes_sbox_table(PlookupMultiTableId::AES_SBOX);
    MULTI_TABLES[PlookupMultiTableId::PEDERSEN_LEFT] =
        pedersen_tables::get_pedersen_left_table(PlookupMultiTableId::PEDERSEN_LEFT);
    MULTI_TABLES[PlookupMultiTableId::PEDERSEN_RIGHT] =
        pedersen_tables::get_pedersen_right_table(PlookupMultiTableId::PEDERSEN_RIGHT);
    MULTI_TABLES[PlookupMultiTableId::UINT32_XOR] = uint_tables::get_uint32_xor_table(PlookupMultiTableId::UINT32_XOR);
    MULTI_TABLES[PlookupMultiTableId::UINT32_AND] = uint_tables::get_uint32_and_table(PlookupMultiTableId::UINT32_AND);
}
} // namespace

const PlookupMultiTable& create_table(const PlookupMultiTableId id)
{
    if (!inited) {
        init_multi_tables();
        inited = true;
    }
    return MULTI_TABLES[id];
}

PlookupReadData get_table_values(const PlookupMultiTableId id,
                                 const fr& key_a,
                                 const fr& key_b,
                                 const bool is_2_to_1_lookup)
{
    const auto& multi_table = create_table(id);

    const size_t num_lookups = multi_table.lookup_ids.size();

    PlookupReadData result;

    const auto key_a_slices = numeric::slice_input_using_variable_bases(key_a, multi_table.slice_sizes);
    const auto key_b_slices = numeric::slice_input_using_variable_bases(key_b, multi_table.slice_sizes);

    std::vector<fr> column_1_raw_values;
    std::vector<fr> column_2_raw_values;
    std::vector<fr> column_3_raw_values;

    for (size_t i = 0; i < num_lookups; ++i) {
        const auto values = multi_table.get_table_values[i]({ key_a_slices[i], key_b_slices[i] });
        column_1_raw_values.emplace_back(key_a_slices[i]);
        column_2_raw_values.emplace_back(is_2_to_1_lookup ? key_b_slices[i] : values[0]);
        column_3_raw_values.emplace_back(is_2_to_1_lookup ? values[0] : values[1]);

        const PlookupBasicTable::KeyEntry key_entry{ { key_a_slices[i], key_b_slices[i] }, values };
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