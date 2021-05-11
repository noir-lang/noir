#pragma once
#include <common/serialize.hpp>
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>

namespace rollup {
namespace proofs {
namespace notes {
namespace native {
namespace value {

using namespace barretenberg;

struct value_note {
    uint256_t value;
    uint32_t asset_id;
    uint32_t nonce;
    grumpkin::g1::affine_element owner;
    barretenberg::fr secret;
};

inline bool operator==(value_note const& lhs, value_note const& rhs)
{
    return lhs.owner == rhs.owner && lhs.value == rhs.value && lhs.secret == rhs.secret && lhs.nonce == rhs.nonce;
}

inline std::ostream& operator<<(std::ostream& os, value_note const& note)
{
    os << "{ owner_x: " << note.owner.x << ", owner_y: " << note.owner.y << ", view_key: " << note.secret
       << ", value: " << note.value << ", asset_id: " << note.asset_id << ", nonce: " << note.nonce << " }";
    return os;
}

inline void read(uint8_t const*& it, value_note& note)
{
    using serialize::read;
    read(it, note.value);
    read(it, note.asset_id);
    read(it, note.nonce);
    read(it, note.owner);
    read(it, note.secret);
}

inline void write(std::vector<uint8_t>& buf, value_note const& note)
{
    using serialize::write;
    write(buf, note.value);
    write(buf, note.asset_id);
    write(buf, note.nonce);
    write(buf, note.owner);
    write(buf, note.secret);
}

} // namespace value
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup