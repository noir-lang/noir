#include "inner_proof_data.hpp"

namespace rollup {
namespace proofs {

inner_proof_data::inner_proof_data(std::vector<uint8_t> const& proof_data)
{
    proof_id = from_buffer<uint256_t>(proof_data, InnerProofOffsets::PROOF_ID);
    note_commitment1 = from_buffer<grumpkin::fq>(proof_data, InnerProofOffsets::NOTE_COMMITMENT1);
    note_commitment2 = from_buffer<grumpkin::fq>(proof_data, InnerProofOffsets::NOTE_COMMITMENT2);
    nullifier1 = from_buffer<uint256_t>(proof_data, InnerProofOffsets::NULLIFIER1);
    nullifier2 = from_buffer<uint256_t>(proof_data, InnerProofOffsets::NULLIFIER2);
    public_value = from_buffer<uint256_t>(proof_data, InnerProofOffsets::PUBLIC_VALUE);
    public_owner = from_buffer<fr>(proof_data, InnerProofOffsets::PUBLIC_OWNER);
    asset_id = from_buffer<uint256_t>(proof_data, InnerProofOffsets::PUBLIC_ASSET_ID);
    merkle_root = from_buffer<fr>(proof_data, InnerProofOffsets::MERKLE_ROOT);
    tx_fee = from_buffer<uint256_t>(proof_data, InnerProofOffsets::TX_FEE);
    tx_fee_asset_id = from_buffer<uint256_t>(proof_data, InnerProofOffsets::TX_FEE_ASSET_ID);
    bridge_id = from_buffer<uint256_t>(proof_data, InnerProofOffsets::BRIDGE_ID);
    defi_deposit_value = from_buffer<uint256_t>(proof_data, InnerProofOffsets::DEFI_DEPOSIT_VALUE);
    defi_root = from_buffer<fr>(proof_data, InnerProofOffsets::DEFI_ROOT);
    backward_link = from_buffer<fr>(proof_data, InnerProofOffsets::BACKWARD_LINK);
    allow_chain = from_buffer<fr>(proof_data, InnerProofOffsets::ALLOW_CHAIN);
}

} // namespace proofs
} // namespace rollup
