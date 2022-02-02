#pragma once
#include <common/serialize.hpp>
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include "../bridge_id.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace native {
namespace defi_interaction {

struct note {
    uint256_t bridge_id;
    uint32_t interaction_nonce;
    uint256_t total_input_value;
    uint256_t total_output_value_a;
    // output_b_value defaults to 0 if there is only one output note for a given defi bridge
    uint256_t total_output_value_b;
    // did the rollup smart contract call to the defi bridge succeed or fail?
    bool interaction_result;

    bool operator==(note const&) const = default;

    auto commit() const
    {
        return crypto::pedersen::compress_native({ bridge_id,
                                                   total_input_value,
                                                   total_output_value_a,
                                                   total_output_value_b,
                                                   interaction_nonce,
                                                   interaction_result },
                                                 GeneratorIndex::DEFI_INTERACTION_NOTE_COMMITMENT);
    }

    // Returns a byte array where all input fields are treated as 32 bytes.
    // Used for generating the previous_defi_interaction_hash.
    std::vector<uint8_t> to_byte_array() const
    {
        std::vector<uint8_t> buf;

        write(buf, bridge_id);
        write(buf, uint256_t(interaction_nonce));
        write(buf, total_input_value);
        write(buf, total_output_value_a);
        write(buf, total_output_value_b);
        write(buf, uint256_t(interaction_result));

        return buf;
    }
};

inline std::ostream& operator<<(std::ostream& os, note const& note)
{
    os << "{ bridge_id: " << note.bridge_id << ", total_input_value: " << note.total_input_value
       << ", total_output_value_a: " << note.total_output_value_a
       << ", total_output_value_b: " << note.total_output_value_b << ", interaction_nonce: " << note.interaction_nonce
       << ", interaction_result: " << note.interaction_result << " }";
    return os;
}

template <typename B> inline void read(B& buf, note& note)
{
    using serialize::read;
    read(buf, note.bridge_id);
    read(buf, note.total_input_value);
    read(buf, note.total_output_value_a);
    read(buf, note.total_output_value_b);
    read(buf, note.interaction_nonce);
    read(buf, note.interaction_result);
}

template <typename B> inline void write(B& buf, note const& note)
{
    using serialize::write;
    write(buf, note.bridge_id);
    write(buf, note.total_input_value);
    write(buf, note.total_output_value_a);
    write(buf, note.total_output_value_b);
    write(buf, note.interaction_nonce);
    write(buf, note.interaction_result);
}

} // namespace defi_interaction
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup