#pragma once
#include "new_contract_data.hpp"
#include "optionally_revealed_data.hpp"
#include "public_data_read.hpp"
#include "public_data_update_request.hpp"

#include "aztec3/circuits/abis/membership_witness.hpp"
#include "aztec3/circuits/abis/read_request_membership_witness.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

#include <array>
#include <cstddef>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT> struct FinalAccumulatedData {
    using fr = typename NCT::fr;
    using uint32 = typename NCT::uint32;
    using boolean = typename NCT::boolean;
    using AggregationObject = typename NCT::AggregationObject;

    AggregationObject aggregation_object{};

    std::array<fr, MAX_NEW_COMMITMENTS_PER_TX> new_commitments{};
    std::array<fr, MAX_NEW_NULLIFIERS_PER_TX> new_nullifiers{};
    std::array<fr, MAX_NEW_NULLIFIERS_PER_TX> nullified_commitments{};
    // For pending nullifiers, we have:
    // nullifiedCommitments[j] != 0 <==> newNullifiers[j] nullifies nullifiedCommitments[j]

    std::array<fr, MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX> private_call_stack{};
    std::array<fr, MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX> public_call_stack{};
    std::array<fr, MAX_NEW_L2_TO_L1_MSGS_PER_TX> new_l2_to_l1_msgs{};

    std::array<fr, NUM_FIELDS_PER_SHA256> encrypted_logs_hash{};
    std::array<fr, NUM_FIELDS_PER_SHA256> unencrypted_logs_hash{};

    // Here so that the gas cost of this request can be measured by circuits, without actually needing to feed in the
    // variable-length data.
    fr encrypted_log_preimages_length = 0;
    fr unencrypted_log_preimages_length = 0;

    std::array<NewContractData<NCT>, MAX_NEW_CONTRACTS_PER_TX> new_contracts{};

    std::array<OptionallyRevealedData<NCT>, MAX_OPTIONALLY_REVEALED_DATA_LENGTH_PER_TX> optionally_revealed_data{};

    // for serialization, update with new fields
    MSGPACK_FIELDS(aggregation_object,
                   new_commitments,
                   new_nullifiers,
                   nullified_commitments,
                   private_call_stack,
                   public_call_stack,
                   new_l2_to_l1_msgs,
                   encrypted_logs_hash,
                   unencrypted_logs_hash,
                   encrypted_log_preimages_length,
                   unencrypted_log_preimages_length,
                   new_contracts,
                   optionally_revealed_data);
    boolean operator==(FinalAccumulatedData<NCT> const& other) const
    {
        return msgpack_derived_equals<boolean>(*this, other);
    };

    template <typename Builder> FinalAccumulatedData<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        typedef CircuitTypes<Builder> CT;
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the circuit builder:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };
        auto to_circuit_type = [&](auto& e) { return e.to_circuit_type(builder); };

        FinalAccumulatedData<CT> acc_data = {
            typename CT::AggregationObject{
                to_ct(aggregation_object.P0),
                to_ct(aggregation_object.P1),
                to_ct(aggregation_object.public_inputs),
                aggregation_object.proof_witness_indices,
                aggregation_object.has_data,
            },

            to_ct(new_commitments),
            to_ct(new_nullifiers),
            to_ct(nullified_commitments),

            to_ct(private_call_stack),
            to_ct(public_call_stack),
            to_ct(new_l2_to_l1_msgs),

            to_ct(encrypted_logs_hash),
            to_ct(unencrypted_logs_hash),

            to_ct(encrypted_log_preimages_length),
            to_ct(unencrypted_log_preimages_length),

            map(new_contracts, to_circuit_type),
            map(optionally_revealed_data, to_circuit_type),
        };

        return acc_data;
    };

    template <typename Builder> FinalAccumulatedData<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Builder>, NCT>::value);
        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Builder>(e); };
        auto to_native_type = []<typename T>(T& e) { return e.template to_native_type<Builder>(); };

        FinalAccumulatedData<NativeTypes> acc_data = {
            typename NativeTypes::AggregationObject{
                to_nt(aggregation_object.P0),
                to_nt(aggregation_object.P1),
                to_nt(aggregation_object.public_inputs),
                aggregation_object.proof_witness_indices,
                aggregation_object.has_data,
            },

            to_nt(new_commitments),
            to_nt(new_nullifiers),
            to_nt(nullified_commitments),

            to_nt(private_call_stack),
            to_nt(public_call_stack),
            to_nt(new_l2_to_l1_msgs),

            to_nt(encrypted_logs_hash),
            to_nt(unencrypted_logs_hash),

            to_nt(encrypted_log_preimages_length),
            to_nt(unencrypted_log_preimages_length),

            map(new_contracts, to_native_type),
            map(optionally_revealed_data, to_native_type),
        };
        return acc_data;
    }

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        aggregation_object.add_proof_outputs_as_public_inputs();

        set_array_public(new_commitments);
        set_array_public(new_nullifiers);
        set_array_public(nullified_commitments);

        set_array_public(private_call_stack);
        set_array_public(public_call_stack);
        set_array_public(new_l2_to_l1_msgs);

        set_array_public(encrypted_logs_hash);
        set_array_public(unencrypted_logs_hash);

        set_array_public(new_contracts);
        set_array_public(optionally_revealed_data);
    }

    template <typename T, size_t SIZE> void set_array_public(std::array<T, SIZE>& arr)
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));
        for (T& e : arr) {
            fr(e).set_public();
        }
    }

    template <size_t SIZE> void set_array_public(std::array<OptionallyRevealedData<NCT>, SIZE>& arr)
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));
        for (auto& e : arr) {
            e.set_public();
        }
    }

    template <size_t SIZE> void set_array_public(std::array<NewContractData<NCT>, SIZE>& arr)
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));
        for (auto& e : arr) {
            e.set_public();
        }
    }
};

}  // namespace aztec3::circuits::abis
