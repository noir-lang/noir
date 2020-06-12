#include "rollup_proof_data.hpp"

namespace rollup {
namespace client_proofs {
namespace join_split {

rollup_proof_data::rollup_proof_data(std::vector<uint8_t> const& proof_data)
{
    using serialize::read;
    auto ptr = proof_data.data();
    ptr += 28;
    read(ptr, rollup_id);
    ptr += 28;
    read(ptr, data_start_index);
    read(ptr, old_data_root);
    read(ptr, new_data_root);
    read(ptr, old_null_root);
    read(ptr, new_null_root);
    read(ptr, old_data_roots_root);
    read(ptr, new_data_roots_root);
    ptr += 28;
    read(ptr, num_txs);

    inner_proofs.resize(num_txs);
    for (size_t i = 0; i < num_txs; ++i) {
        ptr += 28;
        read(ptr, inner_proofs[i].public_input);
        ptr += 28;
        read(ptr, inner_proofs[i].public_output);
        read(ptr, inner_proofs[i].new_note1);
        read(ptr, inner_proofs[i].new_note2);
        ptr += 16;
        read(ptr, inner_proofs[i].nullifier1);
        ptr += 16;
        read(ptr, inner_proofs[i].nullifier2);
        read(ptr, inner_proofs[i].public_owner);
    }
}

} // namespace join_split
} // namespace client_proofs
} // namespace rollup
