#pragma once
#include <common/serialize.hpp>
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include "bridge_id.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace native {
namespace claim {

struct claim_note_tx_data {
    uint256_t deposit_value;
    uint256_t bridge_id;
    uint256_t note_secret;
    uint32_t defi_interaction_nonce;
};

inline bool operator==(claim_note_tx_data const& lhs, claim_note_tx_data const& rhs)
{
    return lhs.bridge_id == rhs.bridge_id && lhs.deposit_value == rhs.deposit_value &&
           lhs.note_secret == rhs.note_secret && lhs.defi_interaction_nonce == rhs.defi_interaction_nonce;
}

inline void read(uint8_t const*& it, claim_note_tx_data& note)
{
    using serialize::read;
    read(it, note.deposit_value);
    read(it, note.bridge_id);
    read(it, note.note_secret);
    read(it, note.defi_interaction_nonce);
}

inline void write(std::vector<uint8_t>& buf, claim_note_tx_data const& note)
{
    using serialize::write;
    write(buf, note.deposit_value);
    write(buf, note.bridge_id);
    write(buf, note.note_secret);
    write(buf, note.defi_interaction_nonce);
}

} // namespace claim
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup