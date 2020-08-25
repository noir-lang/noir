#include "inner_proof_data.hpp"

namespace rollup {
namespace client_proofs {

inner_proof_data::inner_proof_data(std::vector<uint8_t> const& proof_data)
{
    using serialize::read;
    auto ptr = proof_data.data();
    read(ptr, public_input);
    read(ptr, public_output);
    std::copy(proof_data.data() + 2 * 32, proof_data.data() + 2 * 32 + 64, new_note1.begin());
    std::copy(proof_data.data() + 4 * 32, proof_data.data() + 4 * 32 + 64, new_note2.begin());
    nullifier1 = from_buffer<uint128_t>(proof_data, 6 * 32 + 16);
    nullifier2 = from_buffer<uint128_t>(proof_data, 7 * 32 + 16);
    input_owner = from_buffer<barretenberg::fr>(proof_data, 8 * 32);
    output_owner = from_buffer<barretenberg::fr>(proof_data, 9 * 32);
    merkle_root = from_buffer<barretenberg::fr>(proof_data, 10 * 32);
    account_nullifier = from_buffer<uint128_t>(proof_data, 11 * 32 + 16);
}

} // namespace client_proofs
} // namespace rollup
