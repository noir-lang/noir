#pragma once
#include <common/serialize.hpp>
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>

namespace rollup {
namespace proofs {
namespace notes {
namespace native {

using namespace barretenberg;

struct value_note {
    grumpkin::g1::affine_element owner;
    uint256_t value;
    barretenberg::fr secret;
    uint32_t asset_id;
};

inline bool operator==(value_note const& lhs, value_note const& rhs)
{
    return lhs.owner == rhs.owner && lhs.value == rhs.value && lhs.secret == rhs.secret;
}

inline std::ostream& operator<<(std::ostream& os, value_note const& note)
{
    os << "{ owner_x: " << note.owner.x << ", owner_y: " << note.owner.y << ", view_key: " << note.secret
       << ", value: " << note.value << ", asset_id: " << note.asset_id << " }";
    return os;
}

inline void read(uint8_t const*& it, value_note& note)
{
    using serialize::read;
    read(it, note.owner);
    read(it, note.value);
    read(it, note.secret);
    read(it, note.asset_id);
}

inline void write(std::vector<uint8_t>& buf, value_note const& note)
{
    using serialize::write;
    write(buf, note.owner);
    write(buf, note.value);
    write(buf, note.secret);
    write(buf, note.asset_id);
}

} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup