#pragma once
#include <stdlib/types/turbo.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {
namespace claim {

using namespace plonk::stdlib::types::turbo;

inline field_ct compute_nullifier(field_ct const& note_commitment)
{
    // TODO: check if this is ok with Ariel!
    return pedersen::compress(std::vector<field_ct>{ note_commitment }, true, GeneratorIndex::CLAIM_NOTE_NULLIFIER);
}

} // namespace claim
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup