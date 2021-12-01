#pragma once
#include <stdlib/types/turbo.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {
namespace value {

using namespace plonk::stdlib::types::turbo;

inline auto complete_partial_commitment(field_ct const& value_note_partial_commitment,
                                        suint_ct const& value,
                                        suint_ct const& asset_id,
                                        field_ct const& input_nullifier)
{
    return pedersen::compress({ value_note_partial_commitment, value.value, asset_id.value, input_nullifier },
                              true,
                              GeneratorIndex::VALUE_NOTE_COMMITMENT);
}

} // namespace value
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup