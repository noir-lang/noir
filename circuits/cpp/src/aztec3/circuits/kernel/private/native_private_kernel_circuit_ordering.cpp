#include "common.hpp"
#include "init.hpp"

#include "aztec3/circuits/abis/kernel_circuit_public_inputs.hpp"
#include "aztec3/circuits/abis/previous_kernel_data.hpp"
#include "aztec3/circuits/hash.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/array.hpp"
#include "aztec3/utils/circuit_errors.hpp"
#include "aztec3/utils/dummy_circuit_builder.hpp"

#include <cstddef>

namespace {
using NT = aztec3::utils::types::NativeTypes;

using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::PreviousKernelData;
using aztec3::utils::array_length;
using aztec3::utils::CircuitErrorCode;
using aztec3::utils::DummyCircuitBuilder;

void initialise_end_values(PreviousKernelData<NT> const& previous_kernel, KernelCircuitPublicInputs<NT>& public_inputs)
{
    public_inputs.constants = previous_kernel.public_inputs.constants;

    // Ensure the arrays are the same as previously, before we start pushing more data onto them in other
    // functions within this circuit:
    auto& end = public_inputs.end;
    const auto& start = previous_kernel.public_inputs.end;

    // NOTE: don't forward new_commitments as the nonce must be applied
    // end.new_commitments = start.new_commitments;
    end.new_nullifiers = start.new_nullifiers;

    end.private_call_stack = start.private_call_stack;
    end.public_call_stack = start.public_call_stack;
    end.new_l2_to_l1_msgs = start.new_l2_to_l1_msgs;

    end.encrypted_logs_hash = start.encrypted_logs_hash;
    end.unencrypted_logs_hash = start.unencrypted_logs_hash;

    end.encrypted_log_preimages_length = start.encrypted_log_preimages_length;
    end.unencrypted_log_preimages_length = start.unencrypted_log_preimages_length;

    end.optionally_revealed_data = start.optionally_revealed_data;
}
}  // namespace


namespace aztec3::circuits::kernel::private_kernel {


// TODO(https://github.com/AztecProtocol/aztec-packages/issues/892): optimized based on hints
// regarding matching a read request to a commitment
// i.e., we get pairs i,j such that read_requests[i] == new_commitments[j]
void match_reads_to_commitments(DummyCircuitBuilder& builder,
                                std::array<NT::fr, MAX_READ_REQUESTS_PER_TX> const& read_requests,
                                std::array<ReadRequestMembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>,
                                           MAX_READ_REQUESTS_PER_TX> const& read_request_membership_witnesses,
                                std::array<NT::fr, MAX_NEW_COMMITMENTS_PER_TX> const& new_commitments)
{
    // Arrays read_request and read_request_membership_witnesses must be of the same length. Otherwise,
    // we might get into trouble when accumulating them in public_inputs.end
    builder.do_assert(array_length(read_requests) == array_length(read_request_membership_witnesses),
                      format("[private ordering circuit] mismatch array length between read_requests and witnesses - "
                             "read_requests length: ",
                             array_length(read_requests),
                             " witnesses length: ",
                             array_length(read_request_membership_witnesses)),
                      CircuitErrorCode::PRIVATE_KERNEL__READ_REQUEST_WITNESSES_ARRAY_LENGTH_MISMATCH);

    // match reads to commitments from the previous call(s)
    for (size_t rr_idx = 0; rr_idx < MAX_READ_REQUESTS_PER_TX; rr_idx++) {
        const auto& read_request = read_requests[rr_idx];
        const auto& witness = read_request_membership_witnesses[rr_idx];
        const auto is_transient_read = witness.is_transient;
        const auto& hint_to_commitment = witness.hint_to_commitment;


        if (is_transient_read) {
            size_t match_pos = MAX_NEW_COMMITMENTS_PER_TX;
            // TODO(https://github.com/AztecProtocol/aztec-packages/issues/892): inefficient
            // O(n^2) inner loop will be optimized via matching hints
            for (size_t c_idx = 0; c_idx < MAX_NEW_COMMITMENTS_PER_TX; c_idx++) {
                match_pos = (read_request == new_commitments[c_idx]) ? c_idx : match_pos;
            }

            // Transient reads MUST match a pending commitment
            builder.do_assert(
                match_pos != MAX_NEW_COMMITMENTS_PER_TX,
                format("read_request at position [",
                       rr_idx,
                       "]* is transient but does not match any new commitment.",
                       "\n\tread_request: ",
                       read_request,
                       "\n\tis_transient: ",
                       is_transient_read,
                       "\n\thint_to_commitment: ",
                       hint_to_commitment,
                       "\n\t* the read_request position/index is not expected to match position in app-circuit "
                       "outputs because kernel iterations gradually remove non-transient read_requests as "
                       "membership checks are resolved."),
                CircuitErrorCode::PRIVATE_KERNEL__TRANSIENT_READ_REQUEST_NO_MATCH);
        } else {
            // This if-condition means it is a non-empty read request and it is flagged as transient....
            // NON-transient reads MUST be membership-checked and removed during standard kernel iterations
            // NONE should be here in (let alone output from) the ordering circuit.
            builder.do_assert(
                read_request == NT::fr(0),  // basically: assert(is_transient_read || empty)
                format("read_request at position [",
                       rr_idx,
                       "]* is NOT transient but is still unresolved in the final kernel stage! This implies invalid "
                       "inputs "
                       "to the final (ordering) stage of the kernel.",
                       "\n\tread_request: ",
                       read_request,
                       "\n\tleaf_index: ",
                       witness.leaf_index,
                       "\n\tis_transient: ",
                       is_transient_read,
                       "\n\thint_to_commitment: ",
                       hint_to_commitment,
                       "\n\t* the read_request position/index is not expected to match position in app-circuit "
                       "outputs because kernel iterations gradually remove non-transient read_requests as "
                       "membership checks are resolved."),
                CircuitErrorCode::PRIVATE_KERNEL__UNRESOLVED_NON_TRANSIENT_READ_REQUEST);
        }
    }
}

// TODO(https://github.com/AztecProtocol/aztec-packages/issues/836): match_nullifiers_to_commitments_and_squash
// TODO(https://github.com/AztecProtocol/aztec-packages/issues/837): optimized based on hints
// regarding matching a nullifier to a commitment
// i.e., we get pairs i,j such that new_nullifiers[i] == new_commitments[j]
// void match_nullifiers_to_commitments_and_squash(
//    DummyCircuitBuilder& builder,
//    std::array<NT::fr, MAX_NEW_NULLIFIERS_PER_TX>& new_nullifiers,
//    // std::array<NT::fr, MAX_NEW_NULLIFIERS_PER_TX> const& nullifier_hints, // ???
//    std::array<NT::fr, MAX_NEW_COMMITMENTS_PER_TX>& new_commitments)
//{
//    // match reads to commitments from the previous call(s)
//    for (size_t n_idx = 0; n_idx < MAX_NEW_NULLIFIERS_PER_TX; n_idx++) {
//        const auto& nullifier = new_nullifiers[n_idx];  // new_nullifiers[n_idx].nullifier
//        const auto& nullified_commitment = NT::fr(0);   // new_nullifiers[n_idx].commitment
//        const auto is_transient_nullifier = false;      // nullifier_hints[n_idx].is_transient;
//        // const auto& hint_to_commitment = nullifier_hints[n_idx].hint_to_commitment;
//
//        if (is_transient_nullifier) {
//            size_t match_pos = MAX_NEW_COMMITMENTS_PER_TX;
//            // TODO(https://github.com/AztecProtocol/aztec-packages/issues/837): inefficient
//            // O(n^2) inner loop will be optimized via matching hints
//            for (size_t c_idx = 0; c_idx < MAX_NEW_COMMITMENTS_PER_TX; c_idx++) {
//                match_pos = (nullified_commitment == new_commitments[c_idx]) ? c_idx : match_pos;
//            }
//
//            if (match_pos != MAX_NEW_COMMITMENTS_PER_TX) {
//                new_commitments[match_pos] = fr(0);
//            } else {
//                // Transient nullifiers MUST match a pending commitment
//                builder.do_assert(false,  // match_pos != MAX_NEW_COMMITMENTS_PER_TX,
//                                  format("new_nullifier at position [",
//                                         n_idx,
//                                         "]* is transient but does not match any new commitment.",
//                                         "\n\tnullifier: ",
//                                         nullifier,
//                                         "\n\tnullified_commitment: ",
//                                         nullified_commitment),
//                                  //"\n\thint_to_commitment: ",
//                                  // hint_to_commitment,
//                                  CircuitErrorCode::PRIVATE_KERNEL__TRANSIENT_NEW_NULLIFIER_NO_MATCH);
//            }
//        }
//        // non-transient nullifiers are just kept in new_nullifiers array and forwarded to
//        // public inputs (used later by base rollup circuit)
//    }
//    // Move all zero-ed (removed) entries of these arrays to the end and preserve ordering of other entries
//    array_rearrange(new_commitments);
//    array_rearrange(new_nullifiers);
//}

void apply_commitment_nonces(NT::fr const& first_nullifier,
                             std::array<NT::fr, MAX_NEW_COMMITMENTS_PER_TX> const& siloed_commitments,
                             std::array<NT::fr, MAX_NEW_COMMITMENTS_PER_TX>& unique_siloed_commitments)
{
    for (size_t c_idx = 0; c_idx < MAX_NEW_COMMITMENTS_PER_TX; c_idx++) {
        // Apply nonce to all non-zero/non-empty commitments
        // Nonce is the hash of the first (0th) nullifier and the commitment's index into new_commitments array
        const auto nonce = compute_commitment_nonce<NT>(first_nullifier, c_idx);
        unique_siloed_commitments[c_idx] =
            siloed_commitments[c_idx] == 0 ? 0 : compute_unique_commitment<NT>(nonce, siloed_commitments[c_idx]);
    }
}

KernelCircuitPublicInputs<NT> native_private_kernel_circuit_ordering(DummyCircuitBuilder& builder,
                                                                     PreviousKernelData<NT> const& previous_kernel)
{
    // We'll be pushing data to this during execution of this circuit.
    KernelCircuitPublicInputs<NT> public_inputs{};

    // Do this before any functions can modify the inputs.
    initialise_end_values(previous_kernel, public_inputs);

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1329): validate that 0th nullifier is nonzero

    common_validate_previous_kernel_read_requests(builder,
                                                  previous_kernel.public_inputs.end.read_requests,
                                                  previous_kernel.public_inputs.end.read_request_membership_witnesses);

    // Matching read requests to pending commitments requires the full list of new commitments accumulated over
    // all iterations of the private kernel. Therefore, we match reads against new_commitments in
    // previous_kernel.public_inputs.end, where "previous kernel" is the last "inner" kernel iteration.
    // Remark: The commitments in public_inputs.end have already been SILOED!
    match_reads_to_commitments(builder,
                               previous_kernel.public_inputs.end.read_requests,
                               previous_kernel.public_inputs.end.read_request_membership_witnesses,
                               previous_kernel.public_inputs.end.new_commitments);
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1074): ideally final public_inputs
    // shouldn't even include read_requests and read_request_membership_witnesses as they should be empty.

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/836): match_nullifiers_to_commitments_and_squash
    // Matching nullifiers to pending commitments requires the full list of new commitments accumulated over
    // all iterations of the private kernel. Therefore, we match reads against new_commitments in public_inputs.end
    // which has been initialized to previous_kernel.public_inputs.end in common_initialise_*() above.
    // Remark: The commitments in public_inputs.end have already been SILOED!
    // match_nullifiers_to_commitments_and_squash(builder,
    //                                           public_inputs.end.new_nullifiers,
    //                                           // public_inputs.end.nullifier_hints,
    //                                           public_inputs.end.new_commitments);

    apply_commitment_nonces(previous_kernel.public_inputs.end.new_nullifiers[0],
                            previous_kernel.public_inputs.end.new_commitments,
                            public_inputs.end.new_commitments);

    return public_inputs;
};

}  // namespace aztec3::circuits::kernel::private_kernel