#include <common/streams.hpp>
#include "escape_hatch_tx.hpp"
#include "tx_note.hpp"
#include <crypto/pedersen/pedersen.hpp>

namespace rollup {
namespace client_proofs {
namespace escape_hatch {

using namespace barretenberg;

void write(std::vector<uint8_t>& buf, escape_hatch_tx const& tx)
{
    using serialize::write;
    write(buf, tx.public_output);
    write(buf, tx.num_input_notes);
    write(buf, tx.input_index);
    write(buf, tx.old_data_root);
    write(buf, tx.input_path);
    write(buf, tx.input_note);
    write(buf, tx.signature);
    write(buf, tx.public_owner);
    write(buf, tx.old_nullifier_merkle_root);
    write(buf, tx.new_null_roots);
    write(buf, tx.current_nullifier_paths);
    write(buf, tx.new_nullifier_paths);
    write(buf, tx.account_index);
    write(buf, tx.account_path);
    write(buf, tx.account_nullifier_path);
    write(buf, tx.signing_pub_key);
    write(buf, tx.new_data_root);
    write(buf, tx.old_data_roots_root);
    write(buf, tx.new_data_roots_root);
}

void read(uint8_t const*& it, escape_hatch_tx& tx)
{
    using serialize::read;
    read(it, tx.public_output);
    read(it, tx.num_input_notes);
    read(it, tx.input_index);
    read(it, tx.old_data_root);
    read(it, tx.input_path);
    read(it, tx.input_note);
    read(it, tx.signature);
    read(it, tx.public_owner);
    read(it, tx.old_nullifier_merkle_root);
    read(it, tx.new_null_roots);
    read(it, tx.current_nullifier_paths);
    read(it, tx.new_nullifier_paths);
    read(it, tx.account_index);
    read(it, tx.account_path);
    read(it, tx.account_nullifier_path);
    read(it, tx.signing_pub_key);
    read(it, tx.new_data_root);
    read(it, tx.old_data_roots_root);
    read(it, tx.new_data_roots_root);
}

bool operator==(escape_hatch_tx const& lhs, escape_hatch_tx const& rhs)
{
    // clang-format off
    return lhs.public_output == rhs.public_output
        && lhs.num_input_notes == rhs.num_input_notes
        && lhs.input_index == rhs.input_index
        && lhs.old_data_root == rhs.old_data_root
        && lhs.input_path == rhs.input_path
        && lhs.input_note == rhs.input_note
        && lhs.signature == rhs.signature
        && lhs.public_owner == rhs.public_owner
        && lhs.old_nullifier_merkle_root == rhs.old_nullifier_merkle_root
        && lhs.new_null_roots == rhs.new_null_roots
        && lhs.current_nullifier_paths == rhs.current_nullifier_paths
        && lhs.new_nullifier_paths == rhs.new_nullifier_paths
        && lhs.account_index == rhs.account_index
        && lhs.account_path == rhs.account_path
        && lhs.account_nullifier_path == rhs.account_nullifier_path
        && lhs.signing_pub_key == rhs.signing_pub_key
        && lhs.new_data_root == rhs.new_data_root
        && lhs.old_data_roots_root == rhs.old_data_roots_root
        && lhs.new_data_roots_root == rhs.new_data_roots_root;
    // clang-format on
}

std::ostream& operator<<(std::ostream& os, escape_hatch_tx const& tx)
{
    return os << "public_output: " << tx.public_output << "\n"
              << "num_input_notes: " << tx.num_input_notes << "\n"
              << "in_index1: " << tx.input_index[0] << "\n"
              << "in_index2: " << tx.input_index[1] << "\n"
              << "merkle_root: " << tx.old_data_root << "\n"
              << "in_path1: " << tx.input_path[0] << "\n"
              << "in_path2: " << tx.input_path[1] << "\n"
              << "in_note1: " << tx.input_note[0] << "\n"
              << "in_note2: " << tx.input_note[1] << "\n"
              << "signature: " << tx.signature << "\n"
              << "public_owner: " << tx.public_owner << "\n"
              << "nullifier_merkle_root: " << tx.old_nullifier_merkle_root << "\n"
              << "new_null_roots1: " << tx.new_null_roots[0] << "\n"
              << "new_null_roots2: " << tx.new_null_roots[1] << "\n"
              << "current_nullifier_paths1: " << tx.current_nullifier_paths[0] << "\n"
              << "current_nullifier_paths2: " << tx.current_nullifier_paths[1] << "\n"
              << "new_nullifier_paths1: " << tx.new_nullifier_paths[0] << "\n"
              << "new_nullifier_paths2: " << tx.new_nullifier_paths[1] << "\n"
              << "account_index: " << tx.account_index << "\n"
              << "account_path: " << tx.account_path << "\n"
              << "account_nullifier_path: " << tx.account_nullifier_path << "\n"
              << "signing_pub_key: " << tx.signing_pub_key << "\n"
              << "new_data_root: " << tx.new_data_root << "\n"
              << "old_data_roots_root: " << tx.old_data_roots_root << "\n"
              << "new_data_roots_root: " << tx.new_data_roots_root << "\n";
}

} // namespace escape_hatch
} // namespace client_proofs
} // namespace rollup
