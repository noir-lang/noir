#include "rollup_proof_data.hpp"
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace rollup {

rollup_proof_data::rollup_proof_data(std::vector<uint8_t> const& proof_data)
{
    using serialize::read;
    auto ptr = proof_data.data();
    ptr += 28;
    read(ptr, rollup_id);
    ptr += 28;
    read(ptr, rollup_size);
    ptr += 28;
    read(ptr, data_start_index);
    read(ptr, old_data_root);
    read(ptr, new_data_root);
    read(ptr, old_null_root);
    read(ptr, new_null_root);
    read(ptr, old_data_roots_root);
    read(ptr, new_data_roots_root);
    total_tx_fees.resize(NUM_ASSETS);
    for (size_t i = 0; i < NUM_ASSETS; ++i) {
        read(ptr, total_tx_fees[i]);
    }
    ptr += 28;
    read(ptr, num_txs);

    inner_proofs.resize(rollup_size);
    for (size_t i = 0; i < rollup_size; ++i) {
        read(ptr, inner_proofs[i].proof_id);
        read(ptr, inner_proofs[i].public_input);
        read(ptr, inner_proofs[i].public_output);
        read(ptr, inner_proofs[i].asset_id);
        read(ptr, inner_proofs[i].new_note1);
        read(ptr, inner_proofs[i].new_note2);
        read(ptr, inner_proofs[i].nullifier1);
        read(ptr, inner_proofs[i].nullifier2);
        read(ptr, inner_proofs[i].input_owner);
        read(ptr, inner_proofs[i].output_owner);
    }

    // Discard padding proofs.
    inner_proofs.resize(num_txs);

    for (auto& coord :
         { &recursion_output[0].x, &recursion_output[0].y, &recursion_output[1].x, &recursion_output[1].y }) {
        uint256_t limb[4];
        for (size_t li = 0; li < 4; ++li) {
            read(ptr, limb[li]);
        }
        *coord = limb[0] + (uint256_t(1) << 68) * limb[1] + (uint256_t(1) << 136) * limb[2] +
                 (uint256_t(1) << 204) * limb[3];
    }
}

} // namespace rollup
} // namespace proofs
} // namespace rollup
