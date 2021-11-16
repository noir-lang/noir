#pragma once
#include <common/container.hpp>
#include "../root_rollup/root_rollup_broadcast_data.hpp"
#include "../root_rollup/verify.hpp"
#include "verify.hpp"

namespace rollup {
namespace proofs {
namespace root_verifier {

inline root_verifier_tx create_root_verifier_tx(root_rollup::verify_result const& result)
{
    root_verifier_tx tx;

    root_rollup::root_rollup_broadcast_data broadcast_data(result.broadcast_data);

    tx.broadcast_data = to_buffer(broadcast_data);
    tx.proof_data = result.proof_data;
    return tx;
}

inline root_verifier_tx create_root_verifier_tx(std::vector<uint8_t> proof_buf, size_t rollup_size)
{
    root_verifier_tx tx;

    size_t broadcast_data_byte_len = 32 * (root_rollup::RootRollupBroadcastFields::INNER_PROOFS_DATA +
                                           rollup_size * rollup::PropagatedInnerProofFields::NUM_FIELDS);
    std::vector<uint8_t> broadcast_data(slice(proof_buf, 0, broadcast_data_byte_len));
    std::vector<uint8_t> root_rollup_proof(slice(proof_buf, broadcast_data_byte_len, proof_buf.size()));

    tx.broadcast_data = broadcast_data;
    tx.proof_data = root_rollup_proof;
    return tx;
}

} // namespace root_verifier
} // namespace proofs
} // namespace rollup
