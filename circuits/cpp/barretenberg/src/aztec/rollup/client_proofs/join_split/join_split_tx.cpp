#include "join_split_tx.hpp"
#include "tx_note.hpp"
#include <crypto/pedersen/pedersen.hpp>

namespace rollup {
namespace client_proofs {
namespace join_split {

using namespace barretenberg;

void write(std::vector<uint8_t>& buf, join_split_tx const& tx)
{
    using serialize::write;
    write(buf, tx.owner_pub_key);
    write(buf, tx.public_input);
    write(buf, tx.public_output);
    write(buf, tx.num_input_notes);
    write(buf, tx.input_index);
    write(buf, tx.merkle_root);
    write(buf, tx.input_path);
    write(buf, tx.input_note);
    write(buf, tx.output_note);
    write(buf, tx.signature);
    write(buf, tx.input_owner);
    write(buf, tx.output_owner);
}

void read(uint8_t const*& it, join_split_tx& tx)
{
    using serialize::read;
    read(it, tx.owner_pub_key);
    read(it, tx.public_input);
    read(it, tx.public_output);
    read(it, tx.num_input_notes);
    read(it, tx.input_index);
    read(it, tx.merkle_root);
    read(it, tx.input_path);
    read(it, tx.input_note);
    read(it, tx.output_note);
    read(it, tx.signature);
    read(it, tx.input_owner);
    read(it, tx.output_owner);
}

bool operator==(join_split_tx const& lhs, join_split_tx const& rhs)
{
    // clang-format off
    return lhs.owner_pub_key == rhs.owner_pub_key
        && lhs.public_input == rhs.public_input
        && lhs.public_output == rhs.public_output
        && lhs.num_input_notes == rhs.num_input_notes
        && lhs.input_index == rhs.input_index
        && lhs.merkle_root == rhs.merkle_root
        && lhs.input_path == rhs.input_path
        && lhs.input_note == rhs.input_note
        && lhs.output_note == rhs.output_note
        && lhs.signature == rhs.signature
        && lhs.input_owner == rhs.input_owner
        && lhs.output_owner == rhs.output_owner;
    // clang-format on
}

std::ostream& operator<<(std::ostream& os, join_split_tx const& tx)
{
    return os << "owner: " << tx.owner_pub_key << "\n"
              << "public_input: " << tx.public_input << "\n"
              << "public_output: " << tx.public_output << "\n"
              << "num_input_notes: " << tx.num_input_notes << "\n"
              << "in_index1: " << tx.input_index[0] << "\n"
              << "in_index2: " << tx.input_index[1] << "\n"
              << "merkle_root: " << tx.merkle_root << "\n"
              << "in_path1: " << tx.input_path[0] << "\n"
              << "in_path2: " << tx.input_path[1] << "\n"
              << "in_note1: " << tx.input_note[0] << "\n"
              << "in_note2: " << tx.input_note[1] << "\n"
              << "out_note1: " << tx.output_note[0] << "\n"
              << "out_note2: " << tx.output_note[1] << "\n"
              << "signature: " << tx.signature << "\n"
              << "input_owner: " << tx.input_owner << "\n"
              << "output_owner: " << tx.output_owner << "\n";
}

} // namespace join_split
} // namespace client_proofs
} // namespace rollup
