#include "./plookup.hpp"
#include <plonk/composer/plookup_composer.hpp>

namespace waffle {
class PLookupComposer;
} // namespace waffle

namespace plonk {
namespace stdlib {
namespace plookup {

using namespace barretenberg;

template <typename Composer>
std::array<std::vector<plonk::stdlib::field_t<Composer>>, 3> read_sequence_from_table(
    const waffle::PLookupMultiTableId id, const field_t<Composer>& key)
{
    Composer* ctx = key.get_context();

    const auto sequence_data = ctx->get_multi_table_values(id, key.get_value());

    std::array<std::vector<plonk::stdlib::field_t<Composer>>, 3> sequence_values;
    if (key.witness_index == UINT32_MAX) {
        for (size_t i = 0; i < sequence_data.column_1_accumulator_values.size(); ++i) {
            sequence_values[0].emplace_back(field_t<Composer>(ctx, sequence_data.column_1_accumulator_values[i]));
            sequence_values[1].emplace_back(field_t<Composer>(ctx, sequence_data.column_2_accumulator_values[i]));
            sequence_values[2].emplace_back(field_t<Composer>(ctx, sequence_data.column_3_accumulator_values[i]));
        }
    } else {
        const auto sequence_indices = ctx->read_sequence_from_multi_table(id, sequence_data, key.witness_index);
        for (size_t i = 0; i < sequence_data.column_1_accumulator_values.size(); ++i) {
            sequence_values[0].emplace_back(field_t<Composer>::from_witness_index(ctx, sequence_indices[0][i]));
            sequence_values[1].emplace_back(field_t<Composer>::from_witness_index(ctx, sequence_indices[1][i]));
            sequence_values[2].emplace_back(field_t<Composer>::from_witness_index(ctx, sequence_indices[2][i]));
        }
    }
    return sequence_values;
}

template <typename Composer>
std::pair<field_t<Composer>, field_t<Composer>> read_pair_from_table(const waffle::PLookupMultiTableId id,
                                                                     const field_t<Composer>& key)
{
    const auto sequence_elements = read_sequence_from_table(id, key);

    return { sequence_elements[1][0], sequence_elements[2][0] };
}

template <typename Composer>
field_t<Composer> read_from_table(const waffle::PLookupMultiTableId id,
                                  const field_t<Composer> key_a,
                                  const field_t<Composer>)
{
    const auto sequence_elements = read_sequence_from_table(id, key_a);

    return sequence_elements[1][0];
}

template plonk::stdlib::field_t<waffle::PLookupComposer> read_from_table(
    const waffle::PLookupMultiTableId id,
    const plonk::stdlib::field_t<waffle::PLookupComposer> key_a,
    const plonk::stdlib::field_t<waffle::PLookupComposer> key_b);

template std::pair<plonk::stdlib::field_t<waffle::PLookupComposer>, plonk::stdlib::field_t<waffle::PLookupComposer>>
read_pair_from_table(const waffle::PLookupMultiTableId id, const plonk::stdlib::field_t<waffle::PLookupComposer>& key);

template std::array<std::vector<plonk::stdlib::field_t<waffle::PLookupComposer>>, 3> read_sequence_from_table(
    const waffle::PLookupMultiTableId id, const plonk::stdlib::field_t<waffle::PLookupComposer>& key);

} // namespace plookup
} // namespace stdlib
} // namespace plonk
