#include "inner_proof_data.hpp"

namespace bb::join_split_example::proofs {

using namespace bb;

inner_proof_data::inner_proof_data(std::vector<uint8_t> const& proof_data)
{
    proof_id = from_buffer<uint256_t>(proof_data, inner_proof_offsets::PROOF_ID);
    note_commitment1 = from_buffer<grumpkin::fq>(proof_data, inner_proof_offsets::NOTE_COMMITMENT1);
    note_commitment2 = from_buffer<grumpkin::fq>(proof_data, inner_proof_offsets::NOTE_COMMITMENT2);
    nullifier1 = from_buffer<uint256_t>(proof_data, inner_proof_offsets::NULLIFIER1);
    nullifier2 = from_buffer<uint256_t>(proof_data, inner_proof_offsets::NULLIFIER2);
    public_value = from_buffer<uint256_t>(proof_data, inner_proof_offsets::PUBLIC_VALUE);
    public_owner = from_buffer<fr>(proof_data, inner_proof_offsets::PUBLIC_OWNER);
    asset_id = from_buffer<uint256_t>(proof_data, inner_proof_offsets::PUBLIC_ASSET_ID);
    merkle_root = from_buffer<fr>(proof_data, inner_proof_offsets::MERKLE_ROOT);
    tx_fee = from_buffer<uint256_t>(proof_data, inner_proof_offsets::TX_FEE);
    tx_fee_asset_id = from_buffer<uint256_t>(proof_data, inner_proof_offsets::TX_FEE_ASSET_ID);
    bridge_call_data = from_buffer<uint256_t>(proof_data, inner_proof_offsets::BRIDGE_CALL_DATA);
    defi_deposit_value = from_buffer<uint256_t>(proof_data, inner_proof_offsets::DEFI_DEPOSIT_VALUE);
    defi_root = from_buffer<fr>(proof_data, inner_proof_offsets::DEFI_ROOT);
    backward_link = from_buffer<fr>(proof_data, inner_proof_offsets::BACKWARD_LINK);
    allow_chain = from_buffer<fr>(proof_data, inner_proof_offsets::ALLOW_CHAIN);
}

} // namespace bb::join_split_example::proofs
