#include "compute_signing_data.hpp"
#include "../notes/native/index.hpp"
#include <crypto/pedersen/pedersen.hpp>

namespace rollup {
namespace proofs {
namespace join_split {

using namespace crypto::pedersen;
using namespace notes::native;

barretenberg::fr compute_signing_data(join_split_tx const& tx)
{
    auto proof_id = tx.proof_id;
    auto is_deposit = proof_id == ProofIds::DEPOSIT;
    auto is_withdraw = proof_id == ProofIds::WITHDRAW;
    auto is_defi = proof_id == ProofIds::DEFI_DEPOSIT;
    auto public_value = tx.public_value;
    auto public_asset_id = tx.asset_id * (is_deposit || is_withdraw);

    auto partial_value_note_commitment = value::create_partial_commitment(
        tx.partial_claim_note.note_secret, tx.input_note[0].owner, tx.input_note[0].account_required, 0);
    claim::claim_note claim_note = { tx.partial_claim_note.deposit_value, tx.partial_claim_note.bridge_id,      0, 0,
                                     partial_value_note_commitment,       tx.partial_claim_note.input_nullifier };
    const grumpkin::fq input_note_1 = tx.input_note[0].commit();
    const grumpkin::fq input_note_2 = tx.input_note[1].commit();
    const grumpkin::fq output_note_1 = is_defi ? claim_note.partial_commit() : tx.output_note[0].commit();
    const grumpkin::fq output_note_2 = tx.output_note[1].commit();

    const auto nullifier1 = compute_nullifier(input_note_1, tx.account_private_key, tx.num_input_notes >= 1);
    const auto nullifier2 = compute_nullifier(input_note_2, tx.account_private_key, tx.num_input_notes >= 2);

    std::vector<grumpkin::fq> to_compress{ public_value,  tx.public_owner,  grumpkin::fq(public_asset_id),
                                           output_note_1, output_note_2,    nullifier1,
                                           nullifier2,    tx.backward_link, tx.allow_chain };

    return compress_native(to_compress);
}

} // namespace join_split
} // namespace proofs
} // namespace rollup