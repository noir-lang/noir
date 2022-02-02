#include "compute_circuit_data.hpp"
#include "join_split_circuit.hpp"
#include "sign_join_split_tx.hpp"
#include "../notes/native/index.hpp"
#include <stdlib/merkle_tree/hash_path.hpp>

namespace rollup {
namespace proofs {
namespace join_split {

using namespace rollup::proofs::join_split;
using namespace plonk::stdlib::types::turbo;
using namespace rollup::proofs::notes::native;
using namespace plonk::stdlib::merkle_tree;

join_split_tx noop_tx()
{
    grumpkin::fr priv_key = grumpkin::fr::random_element();
    grumpkin::g1::affine_element pub_key = grumpkin::g1::one * priv_key;

    value::value_note input_note1 = { 0, 0, 0, pub_key, fr::random_element(), 0, fr::random_element() };
    value::value_note input_note2 = { 0, 0, 0, pub_key, fr::random_element(), 0, fr::random_element() };
    auto input_nullifier1 = compute_nullifier(input_note1.commit(), priv_key, false);
    auto input_nullifier2 = compute_nullifier(input_note2.commit(), priv_key, false);
    value::value_note output_note1 = { 0, 0, 0, pub_key, fr::random_element(), 0, input_nullifier1 };
    value::value_note output_note2 = { 0, 0, 0, pub_key, fr::random_element(), 0, input_nullifier2 };

    auto gibberish_path = fr_hash_path(DATA_TREE_DEPTH, std::make_pair(fr::random_element(), fr::random_element()));

    join_split_tx tx;
    tx.proof_id = ProofIds::DEPOSIT;
    tx.public_value = 1;
    tx.public_owner = fr::one();
    tx.asset_id = 0;
    tx.num_input_notes = 0;
    tx.input_index = { 0, 1 };
    tx.old_data_root = fr::random_element();
    tx.input_path = { gibberish_path, gibberish_path };
    tx.input_note = { input_note1, input_note2 };
    tx.output_note = { output_note1, output_note2 };
    tx.partial_claim_note = { 0, 0, fr::random_element(), 0 };
    tx.account_index = 0;
    tx.account_path = gibberish_path;
    tx.signing_pub_key = pub_key;
    tx.account_private_key = priv_key;
    tx.alias_hash = 0;
    tx.nonce = 0;
    tx.backward_link = fr::zero();
    tx.allow_chain = 0;

    tx.signature = sign_join_split_tx(tx, { priv_key, pub_key });

    return tx;
}

circuit_data get_circuit_data(std::shared_ptr<waffle::ReferenceStringFactory> const& srs)
{
    std::cerr << "Getting join-split circuit data..." << std::endl;

    auto build_circuit = [&](Composer& composer) {
        join_split_tx tx(noop_tx());
        join_split_circuit(composer, tx);
    };

    return proofs::get_circuit_data<Composer>("", srs, "", true, false, false, true, true, true, build_circuit);
}

} // namespace join_split
} // namespace proofs
} // namespace rollup