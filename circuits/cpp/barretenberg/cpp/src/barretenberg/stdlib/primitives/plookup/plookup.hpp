#pragma once
#include <array>
#include <vector>
#include "barretenberg/proof_system/plookup_tables/plookup_tables.hpp"
#include "barretenberg/plonk/composer/ultra_composer.hpp"
#include "barretenberg/proof_system/plookup_tables/types.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"

namespace proof_system::plonk {
namespace stdlib {

template <typename Composer> class plookup_ {
    typedef field_t<Composer> field_pt;

  public:
    static std::pair<field_pt, field_pt> read_pair_from_table(const plookup::MultiTableId id, const field_pt& key);

    static field_pt read_from_2_to_1_table(const plookup::MultiTableId id,
                                           const field_pt& key_a,
                                           const field_pt& key_b);
    static field_pt read_from_1_to_2_table(const plookup::MultiTableId id, const field_pt& key_a);

    static plookup::ReadData<field_pt> get_lookup_accumulators(const plookup::MultiTableId id,
                                                               const field_pt& key_a,
                                                               const field_pt& key_b = 0,
                                                               const bool is_2_to_1_lookup = false);
};

extern template class plookup_<plonk::UltraComposer>;

typedef plookup_<plonk::UltraComposer> plookup_read;
} // namespace stdlib
} // namespace proof_system::plonk
