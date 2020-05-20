#pragma once

#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>

#include "types.hpp"

namespace waffle {

namespace pedersen_tables {

static constexpr size_t BITS_PER_LOOKUP = 13; // TODO support changing this
static constexpr size_t BITS_PER_SCALAR_MULTIPLIER = 128;
static constexpr size_t NUM_LOOKUPS_PER_HASH =
    BITS_PER_SCALAR_MULTIPLIER / BITS_PER_LOOKUP + (BITS_PER_SCALAR_MULTIPLIER % BITS_PER_LOOKUP != 0);

grumpkin::g1::affine_element get_generator_value(const size_t generator_index,
                                                 const size_t lookup_index,
                                                 const size_t wnaf_value);
grumpkin::g1::affine_element get_skew_generator_value(const size_t generator_index,
                                                      const size_t wnaf_value,
                                                      const bool skew);

PLookupMultiTable generate_pedersen_multi_table(const PLookupMultiTableId id = PEDERSEN_1);

PLookupBasicTable generate_pedersen_table(const size_t generator_index,
                                          const size_t slice_index,
                                          PLookupBasicTableId id,
                                          const size_t table_index);

} // namespace pedersen_tables
} // namespace waffle