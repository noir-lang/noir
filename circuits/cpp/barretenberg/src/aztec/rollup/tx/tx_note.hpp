#pragma once
#include <ecc/curves/grumpkin/grumpkin.hpp>

namespace rollup {
namespace tx {

struct tx_note {
    grumpkin::g1::affine_element owner;
    uint32_t value;
    barretenberg::fr secret;
};

grumpkin::g1::affine_element encrypt_note(const tx_note& plaintext);

inline std::ostream& operator<<(std::ostream& os, tx_note const& note)
{
    os << "owner_x:" << note.owner.x << " owner_y:" << note.owner.y << " view_key:" << note.secret
       << " value:" << note.value;
    return os;
}

} // namespace tx
} // namespace rollup
