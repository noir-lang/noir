#include "compute_circuit_data.hpp"
#include "join_split_circuit.hpp"
#include "sign_join_split_tx.hpp"
#include <stdlib/merkle_tree/hash_path.hpp>

namespace rollup {
namespace proofs {
namespace join_split {

using namespace rollup::proofs::join_split;
using namespace plonk::stdlib::types::turbo;
using namespace rollup::proofs::notes;
using namespace plonk::stdlib::merkle_tree;

join_split_tx noop_tx()
{
    grumpkin::fr priv_key = grumpkin::fr::random_element();
    grumpkin::g1::affine_element pub_key = grumpkin::g1::one * priv_key;
    native::value::value_note gibberish_note = { 0, 0, 0, pub_key, fr::random_element() };
    gibberish_note.secret.data[3] = gibberish_note.secret.data[3] & 0x03FFFFFFFFFFFFFFULL;
    gibberish_note.secret = gibberish_note.secret.to_montgomery_form();
    auto gibberish_path = fr_hash_path(DATA_TREE_DEPTH, std::make_pair(fr::random_element(), fr::random_element()));

    join_split_tx tx;
    tx.public_input = 0;
    tx.public_output = 0;
    tx.asset_id = 0;
    tx.num_input_notes = 0;
    tx.input_index = { 0, 1 };
    tx.old_data_root = fr::random_element();
    tx.input_path = { gibberish_path, gibberish_path };
    tx.input_note = { gibberish_note, gibberish_note };
    tx.output_note = { gibberish_note, gibberish_note };
    tx.claim_note = { 0, 0, gibberish_note.secret };
    tx.account_index = 0;
    tx.account_path = gibberish_path;
    tx.signing_pub_key = pub_key;
    tx.account_private_key = priv_key;
    tx.alias_hash = 0;
    tx.nonce = 0;

    tx.input_owner = fr::random_element();
    tx.output_owner = fr::random_element();

    tx.signature = sign_join_split_tx(tx, { priv_key, pub_key });
    return tx;
}

circuit_data get_circuit_data(std::shared_ptr<waffle::ReferenceStringFactory> const& srs,
                              std::string const& key_path,
                              bool compute,
                              bool save,
                              bool load)
{
    std::cerr << "Getting join-split circuit data..." << std::endl;
    auto name = format("join_split");

    auto build_circuit = [&](Composer& composer) {
        join_split_tx tx(noop_tx());
        join_split_circuit(composer, tx);
    };

    return proofs::get_circuit_data(name, srs, key_path, compute, save, load, true, true, true, build_circuit);
}

} // namespace join_split
} // namespace proofs
} // namespace rollup