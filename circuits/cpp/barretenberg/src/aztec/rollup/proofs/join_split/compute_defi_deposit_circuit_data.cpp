#include "compute_defi_deposit_circuit_data.hpp"
#include "sign_join_split_tx.hpp"
#include "../notes/native/value/value_note.hpp"
#include "../notes/native/value/encrypt.hpp"

#include <fstream>
#include <sys/stat.h>
#include <common/timer.hpp>

namespace rollup {
namespace proofs {
namespace join_split {

using namespace rollup::proofs::join_split;
using namespace plonk::stdlib::types::turbo;
using namespace rollup::proofs;
using namespace rollup::proofs::notes::native;
using namespace plonk::stdlib::merkle_tree;

std::vector<uint8_t> create_leaf_data(grumpkin::g1::affine_element const& enc_note)
{
    std::vector<uint8_t> buf;
    write(buf, enc_note.x);
    write(buf, enc_note.y);
    return buf;
}

void append_notes(MerkleTree<MemoryStore>& data_tree, std::vector<value::value_note> const& notes)
{
    for (auto note : notes) {
        auto enc_note = encrypt(note);
        data_tree.update_element(data_tree.size(), create_leaf_data(enc_note));
    }
}

join_split_tx create_defi_deposit_tx(MerkleTree<MemoryStore>& data_tree,
                                     fixtures::user_context& user,
                                     uint32_t defi_deposit_amount,
                                     uint32_t change_amount)
{
    uint32_t asset_id = 3;
    value::value_note dummy_note = { 0, asset_id, 0, user.owner.public_key, user.note_secret };

    // create join split tx inputs
    join_split_tx tx;
    tx.public_input = 0;
    tx.public_output = 0;
    tx.num_input_notes = 2;
    tx.input_index = { 0, 1 };
    tx.input_note = { dummy_note, dummy_note };
    tx.output_note = { dummy_note, dummy_note };
    tx.input_owner = fr::random_element();
    tx.output_owner = fr::random_element();
    tx.account_index = 0;
    tx.account_path = data_tree.get_hash_path(0);
    tx.signing_pub_key = user.signing_keys[0].public_key;
    tx.asset_id = asset_id;
    tx.account_private_key = user.owner.private_key;
    tx.alias_hash = user.alias_hash;
    tx.nonce = 0;
    tx.claim_note = { 0, 0, 0, 0 };

    // Updates for defi deposit proofs
    tx.input_note[0].value = 2 * change_amount;
    tx.input_note[1].value = defi_deposit_amount - change_amount;
    tx.claim_note.deposit_value = defi_deposit_amount;
    tx.output_note[1].value = change_amount;

    bridge_id bridge_id_native = { 0, 2, tx.asset_id, 0, 0 };
    tx.claim_note.bridge_id = bridge_id_native.to_uint256_t();

    // append notes to the data tree
    append_notes(data_tree, { tx.input_note[0], tx.input_note[1] });
    tx.old_data_root = data_tree.root();
    tx.input_path = { data_tree.get_hash_path(0), data_tree.get_hash_path(1) };

    // Sign the defi_deposit proof
    tx.signature = sign_join_split_tx(tx, { tx.account_private_key, tx.signing_pub_key });

    return tx;
}

} // namespace join_split
} // namespace proofs
} // namespace rollup