#pragma once
#include "../optionally_revealed_data.hpp"
#include <barretenberg/common/map.hpp>
#include <barretenberg/stdlib/primitives/witness/witness.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>

namespace aztec3::circuits::abis::private_kernel {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using plonk::stdlib::witness_t;
using std::is_same;

template <typename NCT> struct AccumulatedData {
    typedef typename NCT::fr fr;
    typedef typename NCT::AggregationObject AggregationObject;

    AggregationObject aggregation_object;

    fr private_call_count;

    std::array<fr, KERNEL_NEW_COMMITMENTS_LENGTH> new_commitments;
    std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH> new_nullifiers;

    std::array<fr, KERNEL_PRIVATE_CALL_STACK_LENGTH> private_call_stack;
    std::array<fr, KERNEL_PUBLIC_CALL_STACK_LENGTH> public_call_stack;
    std::array<fr, KERNEL_L1_MSG_STACK_LENGTH> l1_msg_stack;

    std::array<OptionallyRevealedData<NCT>, KERNEL_OPTIONALLY_REVEALED_DATA_LENGTH> optionally_revealed_data;

    template <typename Composer> AccumulatedData<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        typedef CircuitTypes<Composer> CT;
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(composer, e); };
        auto to_circuit_type = [&](auto& e) { return e.to_circuit_type(composer); };

        AccumulatedData<CT> acc_data = {
            typename CT::AggregationObject{
                to_ct(aggregation_object.P0),
                to_ct(aggregation_object.P1),
                to_ct(aggregation_object.public_inputs),
                aggregation_object.proof_witness_indices,
                aggregation_object.has_data,
            },

            to_ct(private_call_count),

            to_ct(new_commitments),
            to_ct(new_nullifiers),

            to_ct(private_call_stack),
            to_ct(public_call_stack),
            to_ct(l1_msg_stack),

            map(optionally_revealed_data, to_circuit_type),
        };

        return acc_data;
    };

    template <typename Composer> AccumulatedData<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Composer>, NCT>::value);
        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Composer>(e); };
        auto to_native_type = []<typename T>(T& e) { return e.template to_native_type<Composer>(); };

        AccumulatedData<NativeTypes> acc_data = {
            typename NativeTypes::AggregationObject{
                to_nt(aggregation_object.P0),
                to_nt(aggregation_object.P1),
                to_nt(aggregation_object.public_inputs),
                aggregation_object.proof_witness_indices,
                aggregation_object.has_data,
            },

            to_nt(private_call_count),

            to_nt(new_commitments),
            to_nt(new_nullifiers),

            to_nt(private_call_stack),
            to_nt(public_call_stack),
            to_nt(l1_msg_stack),

            map(optionally_revealed_data, to_native_type),
        };

        return acc_data;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        aggregation_object.add_proof_outputs_as_public_inputs();

        private_call_count.set_public();

        set_array_public(new_commitments);
        set_array_public(new_nullifiers);

        set_array_public(private_call_stack);
        set_array_public(public_call_stack);
        set_array_public(l1_msg_stack);

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
};

} // namespace aztec3::circuits::abis::private_kernel