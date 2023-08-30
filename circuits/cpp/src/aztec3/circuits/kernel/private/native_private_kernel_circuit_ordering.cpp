#include "common.hpp"
#include "init.hpp"

#include "aztec3/circuits/abis/kernel_circuit_public_inputs_final.hpp"
#include "aztec3/circuits/abis/previous_kernel_data.hpp"
#include "aztec3/circuits/hash.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/array.hpp"
#include "aztec3/utils/circuit_errors.hpp"
#include "aztec3/utils/dummy_circuit_builder.hpp"

#include <cstddef>

namespace {
using NT = aztec3::utils::types::NativeTypes;

using aztec3::circuits::abis::KernelCircuitPublicInputsFinal;
using aztec3::circuits::abis::PreviousKernelData;
using aztec3::circuits::kernel::private_kernel::common_initialise_end_values;
using aztec3::utils::array_length;
using aztec3::utils::array_rearrange;
using aztec3::utils::CircuitErrorCode;
using aztec3::utils::DummyCircuitBuilder;

void initialise_end_values(PreviousKernelData<NT> const& previous_kernel,
                           KernelCircuitPublicInputsFinal<NT>& public_inputs)
{
    common_initialise_end_values(previous_kernel, public_inputs);
    public_inputs.end.new_contracts = previous_kernel.public_inputs.end.new_contracts;
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

// TODO(https://github.com/AztecProtocol/aztec-packages/issues/837): optimized based on hints
// regarding matching a nullifier to a commitment
// i.e., we get pairs i,j such that new_nullifiers[i] == new_commitments[j]

/**
 * @brief This function matches transient nullifiers to commitments and squashes (deletes) them both.
 *
 * @details A non-zero entry in nullified_commitments at position i implies that
 * 1) new_commitments array contains at least an occurence of nullified_commitments[i]
 * 2) this commitment is nullified by new_nullifiers[i] (according to app circuit, the kernel cannot check this on its
 * own.)
 * Remark: We do not check that new_nullifiers[i] is non-empty. (app circuit responsibility)
 *
 * @param builder
 * @param new_nullifiers public_input's nullifiers that should be squashed when matching a transient commitment
 * @param nullified_commitments commitments that each new_nullifier nullifies. 0 here implies non-transient nullifier,
 * and a non-zero `nullified_commitment` implies a transient nullifier that MUST be matched to a new_commitment.
 * @param new_commitments public_input's commitments to be matched against and squashed when matched to a transient
 * nullifier.
 */
void match_nullifiers_to_commitments_and_squash(
    DummyCircuitBuilder& builder,
    std::array<NT::fr, MAX_NEW_NULLIFIERS_PER_TX>& new_nullifiers,
    std::array<NT::fr, MAX_NEW_NULLIFIERS_PER_TX> const& nullified_commitments,
    std::array<NT::fr, MAX_NEW_COMMITMENTS_PER_TX>& new_commitments)
{
    // match reads to commitments from the previous call(s)
    for (size_t n_idx = 0; n_idx < MAX_NEW_NULLIFIERS_PER_TX; n_idx++) {
        // Nullified_commitment of value `EMPTY_NULLIFIED_COMMITMENT` implies non-transient (persistable)
        // nullifier in which case no attempt will be made to match it to a commitment.
        // Non-empty nullified_commitment implies transient nullifier which MUST be matched to a commitment below!
        // 0-valued nullified_commitment is empty and will be ignored
        if (nullified_commitments[n_idx] != NT::fr(0) &&
            nullified_commitments[n_idx] != NT::fr(EMPTY_NULLIFIED_COMMITMENT)) {
            size_t match_pos = MAX_NEW_COMMITMENTS_PER_TX;
            // TODO(https://github.com/AztecProtocol/aztec-packages/issues/837): inefficient
            // O(n^2) inner loop will be optimized via matching hints
            for (size_t c_idx = 0; c_idx < MAX_NEW_COMMITMENTS_PER_TX; c_idx++) {
                // If there are multiple matches, this picks the last one
                match_pos = (nullified_commitments[n_idx] == new_commitments[c_idx]) ? c_idx : match_pos;
            }

            if (match_pos != MAX_NEW_COMMITMENTS_PER_TX) {
                // match found!
                // squash both the nullifier and the commitment
                // (set to 0 here and then rearrange array after loop)
                important("chopped commitment for siloed inner hash note \n", new_commitments[match_pos]);
                new_commitments[match_pos] = NT::fr(0);
                new_nullifiers[n_idx] = NT::fr(0);
            } else {
                // Transient nullifiers MUST match a pending commitment
                builder.do_assert(false,
                                  format("new_nullifier at position [",
                                         n_idx,
                                         "]* is transient but does not match any new commitment.",
                                         "\n\tnullifier: ",
                                         new_nullifiers[n_idx],
                                         "\n\tnullified_commitment: ",
                                         nullified_commitments[n_idx]),
                                  CircuitErrorCode::PRIVATE_KERNEL__TRANSIENT_NEW_NULLIFIER_NO_MATCH);
            }
        }
        // non-transient (persistable) nullifiers are just kept in new_nullifiers array and forwarded
        // to public inputs (used later by base rollup circuit)
    }
    // Move all zero-ed (removed) entries of these arrays to the end and preserve ordering of other entries
    array_rearrange(new_commitments);
    array_rearrange(new_nullifiers);
}

void apply_commitment_nonces(NT::fr const& first_nullifier,
                             std::array<NT::fr, MAX_NEW_COMMITMENTS_PER_TX>& new_commitments)
{
    for (size_t c_idx = 0; c_idx < MAX_NEW_COMMITMENTS_PER_TX; c_idx++) {
        // Apply nonce to all non-zero/non-empty commitments
        // Nonce is the hash of the first (0th) nullifier and the commitment's index into new_commitments array
        const auto nonce = compute_commitment_nonce<NT>(first_nullifier, c_idx);
        new_commitments[c_idx] =
            new_commitments[c_idx] == 0 ? 0 : compute_unique_commitment<NT>(nonce, new_commitments[c_idx]);
    }
}

KernelCircuitPublicInputsFinal<NT> native_private_kernel_circuit_ordering(DummyCircuitBuilder& builder,
                                                                          PreviousKernelData<NT> const& previous_kernel)
{
    // We'll be pushing data to this during execution of this circuit.
    KernelCircuitPublicInputsFinal<NT> public_inputs{};

    // Do this before any functions can modify the inputs.
    initialise_end_values(previous_kernel, public_inputs);

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1329): validate that 0th nullifier is nonzero
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1486): validate that `len(new_nullifiers) ==
    // len(nullified_commitments)`

    common_validate_previous_kernel_read_requests(builder,
                                                  previous_kernel.public_inputs.end.read_requests,
                                                  previous_kernel.public_inputs.end.read_request_membership_witnesses);

    // Matching read requests to pending commitments requires the full list of new commitments accumulated over
    // all iterations of the private kernel. Therefore, we match reads against new_commitments in
    // previous_kernel.public_inputs.end, where "previous kernel" is the last "inner" kernel iteration.
    // Remark: The commitments in public_inputs.end have already been siloed by contract address!
    match_reads_to_commitments(builder,
                               previous_kernel.public_inputs.end.read_requests,
                               previous_kernel.public_inputs.end.read_request_membership_witnesses,
                               previous_kernel.public_inputs.end.new_commitments);

    // Matching nullifiers to pending commitments requires the full list of new commitments accumulated over
    // all iterations of the private kernel. Therefore, we match nullifiers (their nullified_commitments)
    // against new_commitments in public_inputs.end which has been initialized to
    // previous_kernel.public_inputs.end in common_initialise_*() above.
    // Remark: The commitments in public_inputs.end have already been siloed by contract address!
    match_nullifiers_to_commitments_and_squash(builder,
                                               public_inputs.end.new_nullifiers,
                                               public_inputs.end.nullified_commitments,
                                               public_inputs.end.new_commitments);

    // tx hash
    const auto& first_nullifier = previous_kernel.public_inputs.end.new_nullifiers[0];
    apply_commitment_nonces(first_nullifier, public_inputs.end.new_commitments);

    return public_inputs;
};

}  // namespace aztec3::circuits::kernel::private_kernel