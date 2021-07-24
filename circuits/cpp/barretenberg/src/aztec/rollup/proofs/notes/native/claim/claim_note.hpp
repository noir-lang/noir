#pragma once
#include <common/serialize.hpp>
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include "create_partial_commitment.hpp"
#include "complete_partial_commitment.hpp"
#include "../bridge_id.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace native {
namespace claim {

struct claim_note {
    uint256_t deposit_value;
    uint256_t bridge_id;
    uint32_t defi_interaction_nonce;
    grumpkin::fq value_note_partial_commitment;

    bool operator==(claim_note const&) const = default;

    auto commit() const
    {
        return complete_partial_commitment(
            create_partial_commitment(deposit_value, bridge_id, value_note_partial_commitment), defi_interaction_nonce);
    }

    auto partial_commit() const
    {
        return create_partial_commitment(deposit_value, bridge_id, value_note_partial_commitment);
    }
};

template <typename B> inline void read(B& buf, claim_note& note)
{
    using serialize::read;
    read(buf, note.deposit_value);
    read(buf, note.bridge_id);
    read(buf, note.defi_interaction_nonce);
    read(buf, note.value_note_partial_commitment);
}

template <typename B> inline void write(B& buf, claim_note const& note)
{
    using serialize::write;
    write(buf, note.deposit_value);
    write(buf, note.bridge_id);
    write(buf, note.defi_interaction_nonce);
    write(buf, note.value_note_partial_commitment);
}

inline std::ostream& operator<<(std::ostream& os, claim_note const& note)
{
    return os << format("{ deposit_value: ",
                        note.deposit_value,
                        ", bridge_id: ",
                        note.bridge_id,
                        ", interaction_nonce: ",
                        note.defi_interaction_nonce,
                        ", value_note_partial_commitment: ",
                        note.value_note_partial_commitment,
                        " }");
}

} // namespace claim
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup