#include "join_split_tx.hpp"
#include <crypto/pedersen/pedersen.hpp>

namespace rollup {
namespace proofs {
namespace join_split {

using namespace barretenberg;

void write(std::vector<uint8_t>& buf, join_split_tx const& tx)
{
    using serialize::write;
    write(buf, tx.public_input);
    write(buf, tx.public_output);
    write(buf, tx.asset_id);
    write(buf, tx.num_input_notes);
    write(buf, tx.input_index);
    write(buf, tx.old_data_root);
    write(buf, tx.input_path);
    write(buf, tx.input_note);
    write(buf, tx.output_note);
    write(buf, tx.claim_note);

    write(buf, tx.account_private_key);
    write(buf, tx.alias_hash);
    write(buf, tx.nonce);
    write(buf, tx.account_index);
    write(buf, tx.account_path);
    write(buf, tx.signing_pub_key);
    write(buf, tx.signature);

    write(buf, tx.input_owner);
    write(buf, tx.output_owner);
}

void read(uint8_t const*& it, join_split_tx& tx)
{
    using serialize::read;
    read(it, tx.public_input);
    read(it, tx.public_output);
    read(it, tx.asset_id);
    read(it, tx.num_input_notes);
    read(it, tx.input_index);
    read(it, tx.old_data_root);
    read(it, tx.input_path);
    read(it, tx.input_note);
    read(it, tx.output_note);
    read(it, tx.claim_note);

    read(it, tx.account_private_key);
    read(it, tx.alias_hash);
    read(it, tx.nonce);
    read(it, tx.account_index);
    read(it, tx.account_path);
    read(it, tx.signing_pub_key);
    read(it, tx.signature);

    read(it, tx.input_owner);
    read(it, tx.output_owner);
}

std::ostream& operator<<(std::ostream& os, join_split_tx const& tx)
{
    return os << "public_input: " << tx.public_input << "\n"
              << "public_output: " << tx.public_output << "\n"
              << "asset_id: " << tx.asset_id << "\n"
              << "num_input_notes: " << tx.num_input_notes << "\n"
              << "in_index1: " << tx.input_index[0] << "\n"
              << "in_index2: " << tx.input_index[1] << "\n"
              << "merkle_root: " << tx.old_data_root << "\n"
              << "in_path1: " << tx.input_path[0] << "\n"
              << "in_path2: " << tx.input_path[1] << "\n"
              << "in_note1: " << tx.input_note[0] << "\n"
              << "in_note2: " << tx.input_note[1] << "\n"
              << "out_note1: " << tx.output_note[0] << "\n"
              << "out_note2: " << tx.output_note[1] << "\n"
              << "claim_note: " << tx.claim_note << "\n"
              << "account_private_key: " << tx.account_private_key << "\n"
              << "alias_hash: " << tx.alias_hash << "\n"
              << "nonce: " << tx.nonce << "\n"
              << "account_index: " << tx.account_index << "\n"
              << "account_path: " << tx.account_path << "\n"
              << "signing_pub_key: " << tx.signing_pub_key << "\n"
              << "signature: " << tx.signature << "\n"
              << "input_owner: " << tx.input_owner << "\n"
              << "output_owner: " << tx.output_owner << "\n";
}

} // namespace join_split
} // namespace proofs
} // namespace rollup
