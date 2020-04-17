#pragma once
#include "tx_note.hpp"
#include <crypto/schnorr/schnorr.hpp>
#include <stdlib/merkle_tree/hash_path.hpp>

namespace rollup {
namespace client_proofs {
namespace join_split {

using namespace plonk::stdlib;

class join_split_tx {
public:
    std::vector<uint8_t> to_buffer();
    static join_split_tx from_buffer(std::vector<uint8_t> buffer);

    grumpkin::g1::affine_element owner_pub_key;
    uint32_t public_input;
    uint32_t public_output;
    uint32_t num_input_notes;
    merkle_tree::fr_hash_path input_note_hash_paths[2];
    tx_note input_note[2];
    tx_note output_note[2];
    crypto::schnorr::signature signature;
};

inline std::ostream& operator<<(std::ostream& os, join_split_tx const& tx)
{
    return os << "owner: " << tx.owner_pub_key << "\n"
              << "public_input: " << tx.public_input << "\n"
              << "public_output: " << tx.public_output << "\n"
              << "num_input_notes: " << tx.num_input_notes << "\n"
              << "in_path1: " << tx.input_note_hash_paths[0] << "\n"
              << "in_path2: " << tx.input_note_hash_paths[1] << "\n"
              << "in_note1: " << tx.input_note[0] << "\n"
              << "in_note2: " << tx.input_note[1] << "\n"
              << "out_note1: " << tx.output_note[0] << "\n"
              << "out_note2: " << tx.output_note[1] << "\n"
              << "signature: " << tx.signature << "\n";
}

} // namespace join_split
} // namespace client_proofs
} // namespace rollup