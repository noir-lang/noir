#pragma once
#include <array>
#include <vector>
#include <plonk/composer/plookup_tables/plookup_tables.hpp>
#include <plonk/composer/plookup_composer.hpp>

#include <stdlib/primitives/field/field.hpp>

namespace plonk {
namespace stdlib {

template <typename Composer> class plookup_base {
    typedef field_t<Composer> field_pt;

  public:
    static field_pt read_from_table(const waffle::PlookupMultiTableId id,
                                    const field_pt key_a,
                                    const field_pt key_b = 0);

    static std::pair<field_pt, field_pt> read_pair_from_table(const waffle::PlookupMultiTableId id,
                                                              const field_pt& key);

    static field_pt read_from_2_to_1_table(const waffle::PlookupMultiTableId id,
                                           const field_pt& key_a,
                                           const field_pt& key_b);
    static field_pt read_from_1_to_2_table(const waffle::PlookupMultiTableId id, const field_pt& key_a);

    static std::array<std::vector<field_pt>, 3> read_sequence_from_table(const waffle::PlookupMultiTableId id,
                                                                         const field_pt& key_a,
                                                                         const field_pt& key_b = 0,
                                                                         const bool is_2_to_1_lookup = false);
};

extern template class plookup_base<waffle::PlookupComposer>;

typedef plookup_base<waffle::PlookupComposer> plookup;
} // namespace stdlib
} // namespace plonk
