#pragma once

#include "call_context.hpp"
#include "contract_storage_read.hpp"
#include "contract_storage_update_request.hpp"
#include "../../constants.hpp"

#include "aztec3/utils/msgpack_derived_output.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include "barretenberg/common/throw_or_abort.hpp"
#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;

template <typename NCT> struct PublicCircuitPublicInputs {
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;
    using address = typename NCT::address;

    CallContext<NCT> call_context{};

    fr args_hash = 0;
    std::array<fr, RETURN_VALUES_LENGTH> return_values{};

    std::array<ContractStorageUpdateRequest<NCT>, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL>
        contract_storage_update_requests{};
    std::array<ContractStorageRead<NCT>, MAX_PUBLIC_DATA_READS_PER_CALL> contract_storage_reads{};

    std::array<fr, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL> public_call_stack{};
    std::array<fr, MAX_NEW_COMMITMENTS_PER_CALL> new_commitments{};
    std::array<fr, MAX_NEW_NULLIFIERS_PER_CALL> new_nullifiers{};
    std::array<fr, MAX_NEW_L2_TO_L1_MSGS_PER_CALL> new_l2_to_l1_msgs{};

    std::array<fr, NUM_FIELDS_PER_SHA256> unencrypted_logs_hash{};

    // Here so that the gas cost of this request can be measured by circuits, without actually needing to feed in the
    // variable-length data.
    fr unencrypted_log_preimages_length = 0;

    fr historic_public_data_tree_root = 0;

    address prover_address;

    // for serialization, update with new fields
    MSGPACK_FIELDS(call_context,
                   args_hash,
                   return_values,
                   contract_storage_update_requests,
                   contract_storage_reads,
                   public_call_stack,
                   new_commitments,
                   new_nullifiers,
                   new_l2_to_l1_msgs,
                   unencrypted_logs_hash,
                   unencrypted_log_preimages_length,
                   historic_public_data_tree_root,
                   prover_address);

    boolean operator==(PublicCircuitPublicInputs<NCT> const& other) const
    {
        return msgpack_derived_equals<boolean>(*this, other);
    }

    template <typename Builder> PublicCircuitPublicInputs<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the circuit builder:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };
        auto to_circuit_type = [&](auto& e) { return e.to_circuit_type(builder); };

        PublicCircuitPublicInputs<CircuitTypes<Builder>> pis = {
            .call_context = to_circuit_type(call_context),

            .args_hash = to_ct(args_hash),
            .return_values = to_ct(return_values),

            .contract_storage_update_requests = map(contract_storage_update_requests, to_circuit_type),
            .contract_storage_reads = map(contract_storage_reads, to_circuit_type),

            .public_call_stack = to_ct(public_call_stack),
            .new_commitments = to_ct(new_commitments),
            .new_nullifiers = to_ct(new_nullifiers),
            .new_l2_to_l1_msgs = to_ct(new_l2_to_l1_msgs),

            .unencrypted_logs_hash = to_ct(unencrypted_logs_hash),
            .unencrypted_log_preimages_length = to_ct(unencrypted_log_preimages_length),

            .historic_public_data_tree_root = to_ct(historic_public_data_tree_root),

            .prover_address = to_ct(prover_address),
        };

        return pis;
    };

    fr hash() const
    {
        auto to_hashes = []<typename T>(const T& e) { return e.hash(); };

        std::vector<fr> inputs;

        inputs.push_back(call_context.hash());

        inputs.push_back(args_hash);
        spread_arr_into_vec(return_values, inputs);

        spread_arr_into_vec(map(contract_storage_update_requests, to_hashes), inputs);
        spread_arr_into_vec(map(contract_storage_reads, to_hashes), inputs);

        spread_arr_into_vec(public_call_stack, inputs);
        spread_arr_into_vec(new_commitments, inputs);
        spread_arr_into_vec(new_nullifiers, inputs);
        spread_arr_into_vec(new_l2_to_l1_msgs, inputs);

        spread_arr_into_vec(unencrypted_logs_hash, inputs);
        inputs.push_back(unencrypted_log_preimages_length);

        inputs.push_back(historic_public_data_tree_root);
        inputs.push_back(prover_address);

        if (inputs.size() != PUBLIC_CIRCUIT_PUBLIC_INPUTS_HASH_INPUT_LENGTH) {
            throw_or_abort("Incorrect number of input fields when hashing PublicCircuitPublicInputs");
        }
        return NCT::hash(inputs, GeneratorIndex::PUBLIC_CIRCUIT_PUBLIC_INPUTS);
    }

    template <size_t SIZE> void spread_arr_into_vec(std::array<fr, SIZE> const& arr, std::vector<fr>& vec) const
    {
        const auto arr_size = sizeof(arr) / sizeof(fr);
        vec.insert(vec.end(), arr.data(), arr.data() + arr_size);
    }
};  // namespace aztec3::circuits::abis

template <typename NCT> void read(uint8_t const*& it, PublicCircuitPublicInputs<NCT>& public_circuit_public_inputs)
{
    using serialize::read;

    PublicCircuitPublicInputs<NCT>& pis = public_circuit_public_inputs;
    read(it, pis.call_context);
    read(it, pis.args_hash);
    read(it, pis.return_values);

    read(it, pis.contract_storage_update_requests);
    read(it, pis.contract_storage_reads);

    read(it, pis.public_call_stack);
    read(it, pis.new_commitments);
    read(it, pis.new_nullifiers);
    read(it, pis.new_l2_to_l1_msgs);

    read(it, pis.unencrypted_logs_hash);
    read(it, pis.unencrypted_log_preimages_length);

    read(it, pis.historic_public_data_tree_root);

    read(it, pis.prover_address);
};

template <typename NCT>
void write(std::vector<uint8_t>& buf, PublicCircuitPublicInputs<NCT> const& public_circuit_public_inputs)
{
    using serialize::write;

    PublicCircuitPublicInputs<NCT> const& pis = public_circuit_public_inputs;

    write(buf, pis.call_context);
    write(buf, pis.args_hash);
    write(buf, pis.return_values);

    write(buf, pis.contract_storage_update_requests);
    write(buf, pis.contract_storage_reads);

    write(buf, pis.public_call_stack);
    write(buf, pis.new_commitments);
    write(buf, pis.new_nullifiers);
    write(buf, pis.new_l2_to_l1_msgs);

    write(buf, pis.unencrypted_logs_hash);
    write(buf, pis.unencrypted_log_preimages_length);

    write(buf, pis.historic_public_data_tree_root);

    write(buf, pis.prover_address);
};

template <typename NCT>
std::ostream& operator<<(std::ostream& os, PublicCircuitPublicInputs<NCT> const& public_circuit_public_inputs)

{
    utils::msgpack_derived_output(os, public_circuit_public_inputs);
    return os;
}
}  // namespace aztec3::circuits::abis