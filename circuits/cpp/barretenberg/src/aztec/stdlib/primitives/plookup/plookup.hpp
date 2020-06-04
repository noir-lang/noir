#pragma once
#include <array>
#include <vector>
#include <plonk/composer/plookup_tables/plookup_tables.hpp>
#include <plonk/composer/plookup_composer.hpp>

#include <stdlib/primitives/field/field.hpp>

namespace plonk {
namespace stdlib {
namespace plookup {
template <typename Composer>
plonk::stdlib::field_t<Composer> read_from_table(const waffle::PLookupMultiTableId id,
                                                 const plonk::stdlib::field_t<Composer> key_a,
                                                 const plonk::stdlib::field_t<Composer> key_b = 0);

template <typename Composer>
std::pair<plonk::stdlib::field_t<Composer>, plonk::stdlib::field_t<Composer>> read_pair_from_table(
    const waffle::PLookupMultiTableId id, const plonk::stdlib::field_t<Composer>& key);

template <typename Composer>
std::array<std::vector<plonk::stdlib::field_t<Composer>>, 3> read_sequence_from_table(
    const waffle::PLookupMultiTableId id, const plonk::stdlib::field_t<Composer>& key);

extern template plonk::stdlib::field_t<waffle::PLookupComposer> read_from_table(
    const waffle::PLookupMultiTableId id,
    const plonk::stdlib::field_t<waffle::PLookupComposer> key_a,
    const plonk::stdlib::field_t<waffle::PLookupComposer> key_b);

extern template std::pair<plonk::stdlib::field_t<waffle::PLookupComposer>,
                          plonk::stdlib::field_t<waffle::PLookupComposer>>
read_pair_from_table(const waffle::PLookupMultiTableId id, const plonk::stdlib::field_t<waffle::PLookupComposer>& key);

extern template std::array<std::vector<plonk::stdlib::field_t<waffle::PLookupComposer>>, 3> read_sequence_from_table(
    const waffle::PLookupMultiTableId id, const plonk::stdlib::field_t<waffle::PLookupComposer>& key);

} // namespace plookup
} // namespace stdlib
} // namespace plonk
