#include "join_split_data.hpp"

namespace rollup {
namespace client_proofs {
namespace join_split {

join_split_data::join_split_data(std::vector<uint8_t> const& proof_data)
{
    public_input = from_buffer<uint32_t>(proof_data, 28);
    public_output = from_buffer<uint32_t>(proof_data, 60);
    std::copy(proof_data.data() + 2 * 32, proof_data.data() + 2 * 32 + 64, new_note1.begin());
    std::copy(proof_data.data() + 4 * 32, proof_data.data() + 4 * 32 + 64, new_note2.begin());
    merkle_root = from_buffer<barretenberg::fr>(proof_data, 6 * 32);
    nullifier1 = from_buffer<uint128_t>(proof_data, 7 * 32 + 16);
    nullifier2 = from_buffer<uint128_t>(proof_data, 8 * 32 + 16);
}

} // namespace join_split
} // namespace client_proofs
} // namespace rollup
