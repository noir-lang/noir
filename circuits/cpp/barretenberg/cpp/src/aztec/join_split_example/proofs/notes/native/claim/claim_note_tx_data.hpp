#pragma once
#include <common/serialize.hpp>
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include "../bridge_call_data.hpp"

namespace join_split_example {
namespace proofs {
namespace notes {
namespace native {
namespace claim {

struct partial_claim_note_data {
    uint256_t deposit_value;
    uint256_t bridge_call_data;
    uint256_t note_secret;
    barretenberg::fr input_nullifier;

    bool operator==(partial_claim_note_data const&) const = default;
};

inline std::ostream& operator<<(std::ostream& os, partial_claim_note_data const& note)
{
    return os << "{ value: " << note.deposit_value << ", bridge_call_data: " << note.bridge_call_data
              << ", secret: " << note.note_secret << ", input_nullifier: " << note.input_nullifier << " }";
}

inline void read(uint8_t const*& it, partial_claim_note_data& note)
{
    using serialize::read;
    read(it, note.deposit_value);
    read(it, note.bridge_call_data);
    read(it, note.note_secret);
    read(it, note.input_nullifier);
}

inline void write(std::vector<uint8_t>& buf, partial_claim_note_data const& note)
{
    using serialize::write;
    write(buf, note.deposit_value);
    write(buf, note.bridge_call_data);
    write(buf, note.note_secret);
    write(buf, note.input_nullifier);
}

} // namespace claim
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace join_split_example