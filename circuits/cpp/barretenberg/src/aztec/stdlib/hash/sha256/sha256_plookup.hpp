#pragma once
#include <array>
#include <plonk/composer/plookup_tables.hpp>
#include <plonk/composer/composer_base.hpp>

#include "../../primitives/field/field.hpp"

namespace waffle {
class PLookupComposer;
} // namespace waffle

namespace plonk {
namespace stdlib {

struct sparse_ch_value {
    field_t<waffle::PLookupComposer> normal;
    field_t<waffle::PLookupComposer> sparse;
    field_t<waffle::PLookupComposer> rot6;
    field_t<waffle::PLookupComposer> rot11;
    field_t<waffle::PLookupComposer> rot25;
};

struct sparse_maj_value {
    field_t<waffle::PLookupComposer> normal;
    field_t<waffle::PLookupComposer> sparse;
    field_t<waffle::PLookupComposer> rot2;
    field_t<waffle::PLookupComposer> rot13;
    field_t<waffle::PLookupComposer> rot22;
};

template <uint64_t base, uint64_t num_bits>
field_t<waffle::PLookupComposer> normalize_sparse_form(
    const field_t<waffle::PLookupComposer>& input,
    waffle::LookupTableId table_id = waffle::LookupTableId::SHA256_BASE7_NORMALIZE);

sparse_maj_value convert_into_sparse_maj_form(const field_t<waffle::PLookupComposer>& a);
sparse_ch_value convert_into_sparse_ch_form(const field_t<waffle::PLookupComposer>& e);

field_t<waffle::PLookupComposer> choose(const sparse_ch_value& e, const sparse_ch_value& f, const sparse_ch_value& g);
field_t<waffle::PLookupComposer> majority(const sparse_maj_value& a,
                                          const sparse_maj_value& b,
                                          const sparse_maj_value& c);

std::array<field_t<waffle::PLookupComposer>, 8> sha256_inner_block(
    const std::array<field_t<waffle::PLookupComposer>, 64>& w);

// std::array<uint32<waffle::PLookupComposer>, 8> sha256_block(const std::array<uint32<waffle::PLookupComposer>, 8>&
// h_init,
//                                                     const std::array<uint32<waffle::PLookupComposer>, 16>& input);

// byte_array<waffle::PLookupComposer> sha256_block(const byte_array<waffle::PLookupComposer>& input);

// bit_array<waffle::PLookupComposer> sha256(const bit_array<waffle::PLookupComposer>& input);

} // namespace stdlib
} // namespace plonk
