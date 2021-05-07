#include "sign_notes.hpp"
#include "encrypt_note.hpp"
#include "compute_nullifier.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <crypto/schnorr/schnorr.hpp>

namespace rollup {
namespace proofs {
namespace notes {
namespace native {

using namespace crypto::schnorr;
using namespace crypto::pedersen;

signature sign_notes(proofs::join_split::join_split_tx const& tx,
                     key_pair<grumpkin::fr, grumpkin::g1> const& keys,
                     numeric::random::Engine* engine)
{

    uint256_t total_input_value = tx.input_note[0].value + tx.input_note[1].value + tx.public_input;
    uint256_t total_output_value =
        tx.output_note[0].value + tx.output_note[1].value + tx.public_output + tx.claim_note.deposit_value;
    grumpkin::fq tx_fee = total_input_value - total_output_value;

    auto is_defi = tx.claim_note.deposit_value > 0;
    auto asset_id = is_defi ? tx.claim_note.bridge_id : tx.asset_id;
    auto public_input = is_defi ? 0 : tx.public_input;
    auto public_output = is_defi ? tx.claim_note.deposit_value : tx.public_output;

    auto partial_state =
        create_partial_value_note(tx.claim_note.note_secret, tx.input_note[0].owner, tx.input_note[0].nonce);
    claim_note claim_note = {
        tx.claim_note.deposit_value, tx.claim_note.bridge_id, tx.claim_note.defi_interaction_nonce, partial_state
    };
    const grumpkin::g1::affine_element input_note_1 = encrypt_note(tx.input_note[0]);
    const grumpkin::g1::affine_element input_note_2 = encrypt_note(tx.input_note[1]);
    const grumpkin::g1::affine_element output_note_1 =
        is_defi ? encrypt_note(claim_note) : encrypt_note(tx.output_note[0]);
    const grumpkin::g1::affine_element output_note_2 = encrypt_note(tx.output_note[1]);

    const auto nullifier1 =
        compute_nullifier(input_note_1, tx.input_index[0], tx.account_private_key, tx.num_input_notes >= 1);
    const auto nullifier2 =
        compute_nullifier(input_note_2, tx.input_index[1], tx.account_private_key, tx.num_input_notes >= 2);

    std::vector<grumpkin::fq> to_compress;

    to_compress.push_back(public_input);
    to_compress.push_back(public_output);
    to_compress.push_back(grumpkin::fq(asset_id));
    to_compress.push_back(output_note_1.x);
    to_compress.push_back(output_note_1.y);
    to_compress.push_back(output_note_2.x);
    to_compress.push_back(output_note_2.y);
    to_compress.push_back(nullifier1);
    to_compress.push_back(nullifier2);
    to_compress.push_back(tx.input_owner);
    to_compress.push_back(tx.output_owner);
    to_compress.push_back(tx_fee);

    fr compressed = compress_native(to_compress);

    std::vector<uint8_t> message(sizeof(fr));
    fr::serialize_to_buffer(compressed, &message[0]);

    crypto::schnorr::signature signature =
        crypto::schnorr::construct_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
            std::string(message.begin(), message.end()), keys, engine);
    return signature;
}

} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup