#pragma once
#include "account.hpp"
#include <plonk/proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>
#include "../compute_circuit_data.hpp"

namespace rollup {
namespace proofs {
namespace account {

using namespace plonk::stdlib::merkle_tree;

/**
 * @brief Create an account noop transaction that sets the members in account_tx to be random/zero values.
 * Note that the noop account tx satisfies the circuit logic, and hence can be used to create "dummy" account proofs
 * that pass verification.
 *
 * @warning This must not be used in any production code!
 */
inline account_tx noop_tx()
{
    grumpkin::fr priv_key = grumpkin::fr::random_element();
    grumpkin::g1::affine_element pub_key = grumpkin::g1::one * priv_key;

    grumpkin::fr new_priv_key = grumpkin::fr::random_element();
    grumpkin::g1::affine_element new_pub_key = grumpkin::g1::one * new_priv_key;

    auto gibberish_path = fr_hash_path(DATA_TREE_DEPTH, std::make_pair(fr::random_element(), fr::random_element()));

    account_tx tx = {};
    tx.merkle_root = fr::random_element();
    tx.account_public_key = pub_key;
    tx.new_account_public_key = pub_key;
    tx.new_signing_pub_key_1 = new_pub_key;
    tx.new_signing_pub_key_2 = new_pub_key;
    tx.alias_hash = (uint256_t(fr::random_element()) & 0xffffffff);
    tx.create = true;
    tx.migrate = false;
    tx.account_note_index = 0;
    tx.signing_pub_key = pub_key;
    tx.account_note_path = gibberish_path;
    tx.sign({ priv_key, pub_key });
    return tx;
}

using circuit_data = proofs::circuit_data;

inline circuit_data get_circuit_data(std::shared_ptr<waffle::ReferenceStringFactory> const& srs, bool mock = false)
{
    std::cerr << "Getting account circuit data..." << std::endl;

    auto build_circuit = [&](Composer& composer) {
        account_tx tx(noop_tx());
        tx.account_note_path.resize(DATA_TREE_DEPTH);
        account_circuit(composer, tx);
    };

    return proofs::get_circuit_data<Composer>(
        "account", "", srs, "", true, false, false, true, true, false, mock, build_circuit);
}

} // namespace account
} // namespace proofs
} // namespace rollup
