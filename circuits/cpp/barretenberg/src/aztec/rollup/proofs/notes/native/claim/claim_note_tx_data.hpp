#pragma once
#include <common/serialize.hpp>
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include "../bridge_id.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace native {
namespace claim {

struct claim_note_tx_data {
    uint256_t deposit_value;
    uint256_t bridge_id;
    grumpkin::g1::affine_element owner;
    uint32_t owner_nonce;
    uint256_t note_secret;

    bool operator==(claim_note_tx_data const&) const = default;
};

inline std::ostream& operator<<(std::ostream& os, claim_note_tx_data const& note)
{
    return os << "{ value: " << note.deposit_value << ", bridge_id: " << note.bridge_id << ", owner: " << note.owner
              << ", owner_nonce: " << note.owner_nonce << ", secret: " << note.note_secret << " }";
}

inline void read(uint8_t const*& it, claim_note_tx_data& note)
{
    using serialize::read;
    read(it, note.deposit_value);
    read(it, note.bridge_id);
    read(it, note.owner);
    read(it, note.owner_nonce);
    read(it, note.note_secret);
}

inline void write(std::vector<uint8_t>& buf, claim_note_tx_data const& note)
{
    using serialize::write;
    write(buf, note.deposit_value);
    write(buf, note.bridge_id);
    write(buf, note.owner);
    write(buf, note.owner_nonce);
    write(buf, note.note_secret);
}

} // namespace claim
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup