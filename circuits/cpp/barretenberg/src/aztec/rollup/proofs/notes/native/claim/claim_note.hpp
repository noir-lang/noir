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

struct claim_note {
    // value of deposited tokens
    uint256_t deposit_value;
    // defi bridge identifier (address, assets involved, number of output notes)
    uint256_t bridge_id;
    // global rollup variable - total number of defi interactions made
    uint32_t defi_interaction_nonce;

    // binds the claim note to the user - this is a join-split note without the `value` and `asset_id` fields (used by
    // rollup provider to create output notes
    grumpkin::g1::affine_element partial_state;

    bool operator==(claim_note const&) const = default;
};

template <typename B> inline void read(B& buf, claim_note& note)
{
    using serialize::read;
    read(buf, note.deposit_value);
    read(buf, note.bridge_id);
    read(buf, note.defi_interaction_nonce);
    read(buf, note.partial_state);
}

template <typename B> inline void write(B& buf, claim_note const& note)
{
    using serialize::write;
    write(buf, note.deposit_value);
    write(buf, note.bridge_id);
    write(buf, note.defi_interaction_nonce);
    write(buf, note.partial_state);
}

inline std::ostream& operator<<(std::ostream& os, claim_note const& note)
{
    return os << format("{ deposit_value: ",
                        note.deposit_value,
                        ", bridge_id: ",
                        note.bridge_id,
                        ", interaction_nonce: ",
                        note.defi_interaction_nonce,
                        ", partial_state: ",
                        note.partial_state,
                        " }");
}

} // namespace claim
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup