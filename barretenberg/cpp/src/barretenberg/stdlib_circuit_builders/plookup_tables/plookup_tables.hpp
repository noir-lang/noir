#pragma once
#include "barretenberg/common/throw_or_abort.hpp"
#include "barretenberg/stdlib_circuit_builders/plookup_tables/types.hpp"

namespace bb::plookup {

const MultiTable& create_table(MultiTableId id);

ReadData<bb::fr> get_lookup_accumulators(MultiTableId id,
                                         const bb::fr& key_a,
                                         const bb::fr& key_b = 0,
                                         bool is_2_to_1_lookup = false);

BasicTable create_basic_table(BasicTableId id, size_t index);
} // namespace bb::plookup
