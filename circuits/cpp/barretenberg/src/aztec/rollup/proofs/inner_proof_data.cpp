#include "inner_proof_data.hpp"

namespace rollup {
namespace proofs {

inner_proof_data::inner_proof_data(std::vector<uint8_t> const& proof_data)
{
    proof_id = from_buffer<uint256_t>(proof_data, 0 * 32);
    public_input = from_buffer<uint256_t>(proof_data, 1 * 32);
    public_output = from_buffer<uint256_t>(proof_data, 2 * 32);

    std::copy(proof_data.data() + 3 * 32, proof_data.data() + 3 * 32 + 64, new_note1.begin());
    std::copy(proof_data.data() + 5 * 32, proof_data.data() + 5 * 32 + 64, new_note2.begin());
    nullifier1 = from_buffer<uint128_t>(proof_data, 7 * 32 + 16);
    nullifier2 = from_buffer<uint128_t>(proof_data, 8 * 32 + 16);
    input_owner = from_buffer<barretenberg::fr>(proof_data, 9 * 32);
    output_owner = from_buffer<barretenberg::fr>(proof_data, 10 * 32);
    merkle_root = from_buffer<barretenberg::fr>(proof_data, 11 * 32);
    account_nullifier = from_buffer<uint128_t>(proof_data, 12 * 32 + 16);
}

} // namespace proofs
} // namespace rollup
