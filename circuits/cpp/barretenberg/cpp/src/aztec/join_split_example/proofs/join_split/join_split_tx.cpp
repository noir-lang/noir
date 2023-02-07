#include "join_split_tx.hpp"
#include <crypto/pedersen/pedersen.hpp>

namespace join_split_example {
namespace proofs {
namespace join_split {

using namespace barretenberg;

void write(std::vector<uint8_t>& buf, join_split_tx const& tx)
{
    using serialize::write;
    write(buf, tx.proof_id);
    write(buf, tx.public_value);
    write(buf, tx.public_owner);
    write(buf, tx.asset_id);
    write(buf, tx.num_input_notes);
    write(buf, tx.input_index);
    write(buf, tx.old_data_root);
    write(buf, tx.input_path);
    write(buf, tx.input_note);
    write(buf, tx.output_note);
    write(buf, tx.partial_claim_note);

    write(buf, tx.account_private_key);
    write(buf, tx.alias_hash);
    write(buf, tx.account_required);
    write(buf, tx.account_note_index);
    write(buf, tx.account_note_path);
    write(buf, tx.signing_pub_key);

    write(buf, tx.backward_link);
    write(buf, tx.allow_chain);

    write(buf, tx.signature);
}

void read(uint8_t const*& it, join_split_tx& tx)
{
    using serialize::read;
    read(it, tx.proof_id);
    read(it, tx.public_value);
    read(it, tx.public_owner);
    read(it, tx.asset_id);
    read(it, tx.num_input_notes);
    read(it, tx.input_index);
    read(it, tx.old_data_root);
    read(it, tx.input_path);
    read(it, tx.input_note);
    read(it, tx.output_note);
    read(it, tx.partial_claim_note);

    read(it, tx.account_private_key);
    read(it, tx.alias_hash);
    read(it, tx.account_required);
    read(it, tx.account_note_index);
    read(it, tx.account_note_path);
    read(it, tx.signing_pub_key);

    read(it, tx.backward_link);
    read(it, tx.allow_chain);

    read(it, tx.signature);
}

std::ostream& operator<<(std::ostream& os, join_split_tx const& tx)
{
    return os << "proof_id: " << tx.proof_id << "\n"
              << "public_value: " << tx.public_value << "\n"
              << "public_owner: " << tx.public_owner << "\n"
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
              << "partial_claim_note: " << tx.partial_claim_note << "\n"
              << "account_private_key: " << tx.account_private_key << "\n"
              << "alias_hash: " << tx.alias_hash << "\n"
              << "account_required: " << tx.account_required << "\n"
              << "account_note_index: " << tx.account_note_index << "\n"
              << "account_note_path: " << tx.account_note_path << "\n"
              << "signing_pub_key: " << tx.signing_pub_key << "\n"
              << "backward_link: " << tx.backward_link << "\n"
              << "allow_chain: " << tx.allow_chain << "\n"
              << "signature: " << tx.signature << "\n";
}

} // namespace join_split
} // namespace proofs
} // namespace join_split_example
