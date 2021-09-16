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
    auto is_defi = tx.claim_note.deposit_value > 0;
    auto asset_id = is_defi ? tx.claim_note.bridge_id : tx.asset_id;
    auto public_input = is_defi ? 0 : tx.public_input;
    auto public_output = is_defi ? tx.claim_note.deposit_value : tx.public_output;

    uint256_t total_input_value = tx.input_note[0].value + tx.input_note[1].value + tx.public_input;
    uint256_t total_output_value = tx.output_note[0].value * !is_defi + tx.output_note[1].value + public_output;
    grumpkin::fq tx_fee = total_input_value - total_output_value;

    auto partial_value_note_commitment =
        value::create_partial_commitment(tx.claim_note.note_secret, tx.input_note[0].owner, tx.input_note[0].nonce, 0);
    claim::claim_note claim_note = {
        tx.claim_note.deposit_value, tx.claim_note.bridge_id, 0, 0, partial_value_note_commitment
    };
    const grumpkin::fq input_note_1 = tx.input_note[0].commit();
    const grumpkin::fq input_note_2 = tx.input_note[1].commit();
    const grumpkin::fq output_note_1 = is_defi ? claim_note.partial_commit() : tx.output_note[0].commit();
    const grumpkin::fq output_note_2 = tx.output_note[1].commit();

    const auto nullifier1 =
        compute_nullifier(input_note_1, tx.input_index[0], tx.account_private_key, tx.num_input_notes >= 1);
    const auto nullifier2 =
        compute_nullifier(input_note_2, tx.input_index[1], tx.account_private_key, tx.num_input_notes >= 2);

    std::vector<grumpkin::fq> to_compress{
        public_input, public_output, grumpkin::fq(asset_id), output_note_1,   output_note_2,
        nullifier1,   nullifier2,    tx.input_owner,         tx.output_owner, tx_fee,
    };

    return compress_native(to_compress);
}

} // namespace join_split
} // namespace proofs
} // namespace rollup