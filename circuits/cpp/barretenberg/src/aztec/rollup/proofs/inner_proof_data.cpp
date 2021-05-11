#include "inner_proof_data.hpp"

namespace rollup {
namespace proofs {

inner_proof_data::inner_proof_data(std::vector<uint8_t> const& proof_data)
{
    proof_id = from_buffer<uint256_t>(proof_data, InnerProofOffsets::PROOF_ID);
    public_input = from_buffer<uint256_t>(proof_data, InnerProofOffsets::PUBLIC_INPUT);
    public_output = from_buffer<uint256_t>(proof_data, InnerProofOffsets::PUBLIC_OUTPUT);
    asset_id = from_buffer<uint256_t>(proof_data, InnerProofOffsets::ASSET_ID);
    new_note1 = from_buffer<grumpkin::g1::affine_element>(proof_data, InnerProofOffsets::NEW_NOTE1_X);
    new_note2 = from_buffer<grumpkin::g1::affine_element>(proof_data, InnerProofOffsets::NEW_NOTE2_X);
    nullifier1 = from_buffer<uint256_t>(proof_data, InnerProofOffsets::NULLIFIER1);
    nullifier2 = from_buffer<uint256_t>(proof_data, InnerProofOffsets::NULLIFIER2);
    input_owner = from_buffer<fr>(proof_data, InnerProofOffsets::INPUT_OWNER);
    output_owner = from_buffer<fr>(proof_data, InnerProofOffsets::OUTPUT_OWNER);
    merkle_root = from_buffer<fr>(proof_data, InnerProofOffsets::MERKLE_ROOT);
    tx_fee = from_buffer<uint256_t>(proof_data, InnerProofOffsets::TX_FEE);
}

} // namespace proofs
} // namespace rollup
