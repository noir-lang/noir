#pragma once
#include <common/serialize.hpp>
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include "create_partial_commitment.hpp"
#include "complete_partial_commitment.hpp"
#include "../bridge_call_data.hpp"

namespace join_split_example {
namespace proofs {
namespace notes {
namespace native {
namespace claim {

struct claim_note {
    uint256_t deposit_value;
    uint256_t bridge_call_data;
    uint32_t defi_interaction_nonce;
    uint256_t fee;
    grumpkin::fq value_note_partial_commitment;
    grumpkin::fq input_nullifier;

    bool operator==(claim_note const&) const = default;

    grumpkin::fq commit() const { return complete_partial_commitment(partial_commit(), defi_interaction_nonce, fee); }

    grumpkin::fq partial_commit() const
    {
        return create_partial_commitment(
            deposit_value, bridge_call_data, value_note_partial_commitment, input_nullifier);
    }
};

template <typename B> inline void read(B& buf, claim_note& note)
{
    using serialize::read;
    read(buf, note.deposit_value);
    read(buf, note.bridge_call_data);
    read(buf, note.defi_interaction_nonce);
    read(buf, note.fee);
    read(buf, note.value_note_partial_commitment);
    read(buf, note.input_nullifier);
}
template <typename B> inline void write(B& buf, claim_note const& note)
{
    write(buf, note.deposit_value);
    write(buf, note.bridge_call_data);
    write(buf, note.defi_interaction_nonce);
    write(buf, note.fee);
    write(buf, note.value_note_partial_commitment);
    write(buf, note.input_nullifier);
}

inline std::ostream& operator<<(std::ostream& os, claim_note const& note)
{
    return os << format("{ deposit_value: ",
                        note.deposit_value,
                        ", bridge_call_data: ",
                        note.bridge_call_data,
                        ", interaction_nonce: ",
                        note.defi_interaction_nonce,
                        ", fee: ",
                        note.fee,
                        ", value_note_partial_commitment: ",
                        note.value_note_partial_commitment,
                        ", input_nullifier: ",
                        note.input_nullifier,
                        " }");
}

} // namespace claim
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace join_split_example