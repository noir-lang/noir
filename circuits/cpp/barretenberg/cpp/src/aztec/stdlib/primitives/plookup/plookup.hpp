#pragma once
#include <array>
#include <vector>
#include <plonk/composer/plookup_tables/plookup_tables.hpp>
#include <plonk/composer/ultra_composer.hpp>
#include <plonk/composer/plookup_tables/types.hpp>
#include <stdlib/primitives/field/field.hpp>

namespace plonk {
namespace stdlib {

using namespace plookup;

template <typename Composer> class plookup_ {
    typedef field_t<Composer> field_pt;

  public:
    static std::pair<field_pt, field_pt> read_pair_from_table(const MultiTableId id, const field_pt& key);

    static field_pt read_from_2_to_1_table(const MultiTableId id, const field_pt& key_a, const field_pt& key_b);
    static field_pt read_from_1_to_2_table(const MultiTableId id, const field_pt& key_a);

    static ReadData<field_pt> get_lookup_accumulators(const MultiTableId id,
                                                      const field_pt& key_a,
                                                      const field_pt& key_b = 0,
                                                      const bool is_2_to_1_lookup = false);
};

extern template class plookup_<waffle::UltraComposer>;

typedef plookup_<waffle::UltraComposer> plookup_read;
} // namespace stdlib
} // namespace plonk
