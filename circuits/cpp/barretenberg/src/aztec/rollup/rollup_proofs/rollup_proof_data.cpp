#include "rollup_proof_data.hpp"

namespace rollup {
namespace client_proofs {
namespace join_split {

rollup_proof_data::rollup_proof_data(std::vector<uint8_t> const& proof_data)
{
    auto ptr = proof_data.data();
    ptr += 28;
    ::read(ptr, data_start_index);
    read(ptr, old_data_root);
    read(ptr, new_data_root);
    read(ptr, old_null_root);
    read(ptr, new_null_root);
    read(ptr, old_root_root);
    ptr += 28;
    ::read(ptr, num_txs);

    inner_proof_data.resize(num_txs);
    for (size_t i = 0; i < num_txs; ++i) {
        ptr += 28;
        ::read(ptr, inner_proof_data[i].public_input);
        ptr += 28;
        ::read(ptr, inner_proof_data[i].public_output);
        read(ptr, inner_proof_data[i].new_note1);
        read(ptr, inner_proof_data[i].new_note2);
        ptr += 16;
        ::read(ptr, inner_proof_data[i].nullifier1);
        ptr += 16;
        ::read(ptr, inner_proof_data[i].nullifier2);
    }
}

} // namespace join_split
} // namespace client_proofs
} // namespace rollup
