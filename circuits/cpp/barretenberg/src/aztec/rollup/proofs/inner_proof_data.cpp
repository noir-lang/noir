#include "inner_proof_data.hpp"

namespace rollup {
namespace proofs {

inner_proof_data::inner_proof_data(std::vector<uint8_t> const& proof_data)
{
    proof_id = from_buffer<uint256_t>(proof_data, InnerProofOffsets::PROOF_ID);
    public_input = from_buffer<uint256_t>(proof_data, InnerProofOffsets::PUBLIC_INPUT);
    public_output = from_buffer<uint256_t>(proof_data, InnerProofOffsets::PUBLIC_OUTPUT);
    asset_id = from_buffer<uint256_t>(proof_data, InnerProofOffsets::ASSET_ID);

    std::copy(proof_data.data() + InnerProofOffsets::NEW_NOTE1_X,
              proof_data.data() + InnerProofOffsets::NEW_NOTE1_X + 64,
              new_note1.begin());
    std::copy(proof_data.data() + InnerProofOffsets::NEW_NOTE2_X,
              proof_data.data() + InnerProofOffsets::NEW_NOTE2_X + 64,
              new_note2.begin());
    nullifier1 = from_buffer<uint256_t>(proof_data, InnerProofOffsets::NULLIFIER1);
    nullifier2 = from_buffer<uint256_t>(proof_data, InnerProofOffsets::NULLIFIER2);
    input_owner = from_buffer<fr>(proof_data, InnerProofOffsets::INPUT_OWNER);
    output_owner = from_buffer<fr>(proof_data, InnerProofOffsets::OUTPUT_OWNER);
    merkle_root = from_buffer<fr>(proof_data, InnerProofOffsets::MERKLE_ROOT);
    account_nullifier = from_buffer<uint256_t>(proof_data, InnerProofOffsets::ACCOUNT_NULLIFIER);
}

} // namespace proofs
} // namespace rollup
