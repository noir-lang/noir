#pragma once
#include "../../constants.hpp"
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/crypto/pedersen_hash/pedersen.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"

namespace join_split_example::proofs::notes::native::claim {

inline auto compute_nullifier(grumpkin::fq const& note_commitment)
{
    return crypto::pedersen_hash::hash({ note_commitment }, GeneratorIndex::CLAIM_NOTE_NULLIFIER);
}

} // namespace join_split_example::proofs::notes::native::claim
