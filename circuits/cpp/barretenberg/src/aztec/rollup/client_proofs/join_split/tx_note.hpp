#pragma once
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <common/net.hpp>

namespace rollup {
namespace client_proofs {
namespace join_split {

using namespace barretenberg;

struct tx_note {
    grumpkin::g1::affine_element owner;
    uint32_t value;
    barretenberg::fr secret;
};

grumpkin::g1::affine_element encrypt_note(const tx_note& plaintext);

inline std::ostream& operator<<(std::ostream& os, tx_note const& note)
{
    os << "{ owner_x: " << note.owner.x << ", owner_y: " << note.owner.y << ", view_key: " << note.secret
       << ", value: " << note.value << " }";
    return os;
}

inline tx_note deserialize_tx_note(uint8_t* buf)
{
    tx_note note;
    note.owner = grumpkin::g1::affine_element::serialize_from_buffer(buf);
    note.value = ntohl(*reinterpret_cast<uint32_t*>(buf + 64));
    note.secret = fr::serialize_from_buffer(buf + 68);
    return note;
}

inline void serialize_tx_note(tx_note const& note, uint8_t* buf)
{
    grumpkin::g1::affine_element::serialize_to_buffer(note.owner, buf);
    *reinterpret_cast<uint32_t*>(buf + 64) = htonl(note.value);
    fr::serialize_to_buffer(note.secret, buf + 68);
}

} // namespace join_split
} // namespace client_proofs
} // namespace rollup