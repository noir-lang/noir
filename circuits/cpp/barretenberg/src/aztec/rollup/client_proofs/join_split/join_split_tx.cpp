#include "join_split_tx.hpp"
#include "tx_note.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <common/net.hpp>

namespace rollup {
namespace client_proofs {
namespace join_split {

using namespace barretenberg;

std::vector<uint8_t> join_split_tx::to_buffer() {
    std::vector<uint8_t> buf(4636);
    auto buffer = buf.data();
    grumpkin::g1::affine_element::serialize_to_buffer(owner_pub_key, buffer);
    buffer += 64;
    *reinterpret_cast<uint32_t*>(buffer) = htonl(public_input);
    buffer += 4;
    *reinterpret_cast<uint32_t*>(buffer) = htonl(public_output);
    buffer += 4;
    *reinterpret_cast<uint32_t*>(buffer) = htonl(num_input_notes);
    buffer += 4;

    merkle_tree::serialize_hash_path(input_note_hash_paths[0], buffer);
    buffer += 32 * 64;
    merkle_tree::serialize_hash_path(input_note_hash_paths[1], buffer);
    buffer += 32 * 64;

    serialize_tx_note(input_note[0], buffer);
    buffer += 100;
    serialize_tx_note(input_note[1], buffer);
    buffer += 100;
    serialize_tx_note(output_note[0], buffer);
    buffer += 100;
    serialize_tx_note(output_note[1], buffer);
    buffer += 100;

    std::copy(signature.s.begin(), signature.s.end(), buffer);
    buffer += 32;
    std::copy(signature.e.begin(), signature.e.end(), buffer);

    return buf;
}

join_split_tx join_split_tx::from_buffer(std::vector<uint8_t> buf)
{
    auto buffer = buf.data();
    join_split_tx tx;

    tx.owner_pub_key = grumpkin::g1::affine_element::serialize_from_buffer(buffer);
    buffer += 64;
    tx.public_input = ntohl(*reinterpret_cast<uint32_t*>(buffer));
    buffer += 4;
    tx.public_output = ntohl(*reinterpret_cast<uint32_t*>(buffer));
    buffer += 4;
    tx.num_input_notes = ntohl(*reinterpret_cast<uint32_t*>(buffer));
    buffer += 4;

    tx.input_note_hash_paths[0] = merkle_tree::deserialize_hash_path(buffer, 32);
    buffer += 32 * 64;
    tx.input_note_hash_paths[1] = merkle_tree::deserialize_hash_path(buffer, 32);
    buffer += 32 * 64;

    tx.input_note[0] = deserialize_tx_note(buffer);
    buffer += 100;
    tx.input_note[1] = deserialize_tx_note(buffer);
    buffer += 100;
    tx.output_note[0] = deserialize_tx_note(buffer);
    buffer += 100;
    tx.output_note[1] = deserialize_tx_note(buffer);
    buffer += 100;

    std::copy(buffer, buffer + 32, tx.signature.s.data());
    buffer += 32;
    std::copy(buffer, buffer + 32, tx.signature.e.data());

    return tx;
}

} // namespace join_split
} // namespace client_proofs
} // namespace rollup