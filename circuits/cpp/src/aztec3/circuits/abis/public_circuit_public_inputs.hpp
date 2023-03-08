#pragma once

#include "call_context.hpp"
#include "state_transition.hpp"
#include "state_read.hpp"
#include "../../constants.hpp"

#include <common/map.hpp>
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>

namespace aztec3::circuits::abis {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename NCT> struct PublicCircuitPublicInputs {
    typedef typename NCT::fr fr;
    typedef typename NCT::boolean boolean;
    typedef typename NCT::address address;

    CallContext<NCT> call_context;

    std::array<fr, ARGS_LENGTH> args;
    std::array<fr, RETURN_VALUES_LENGTH> return_values;

    std::array<fr, EMITTED_EVENTS_LENGTH> emitted_events;

    std::array<StateTransition<NCT>, STATE_TRANSITIONS_LENGTH> state_transitions;
    std::array<StateRead<NCT>, STATE_READS_LENGTH> state_reads;

    std::array<fr, PUBLIC_CALL_STACK_LENGTH> public_call_stack;
    std::array<fr, L1_MSG_STACK_LENGTH> l1_msg_stack;

    fr old_private_data_tree_root;

    address prover_address;

    // bool operator==(PublicCircuitPublicInputs<NCT> const&) const = default;

    // static PublicCircuitPublicInputs<NCT> empty()
    // {
    //     PublicCircuitPublicInputs<NCT> pis = {
    //         std::array<fr, ARGS_LENGTH>::fill(0),
    //         std::array<fr, RETURN_VALUES_LENGTH>::fill(0),

    //         std::array<fr, EMITTED_EVENTS_LENGTH>::fill(0),

    //         std::array<StateTransition<NCT>, STATE_TRANSITIONS_LENGTH>::fill(StateTransition<NCT>::empty()),
    //         std::array<StateRead<NCT>, STATE_READS_LENGTH>::fill(StateRead<NCT>::empty()),

    //         std::array<fr, PUBLIC_CALL_STACK_LENGTH>::fill(0),

    //         std::array<fr, CONTRACT_DEPLOYMENT_CALL_STACK_LENGTH>::fill(0),
    //         std::array<fr, L1_MSG_STACK_LENGTH>::fill(0),

    //         .old_private_data_tree_root = 0,

    //         .prover_address = 0,
    //     };

    //     return pis;
    // };

    template <typename Composer>
    PublicCircuitPublicInputs<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };
        auto to_circuit_type = [&](auto& e) { return e.to_circuit_type(composer); };

        PublicCircuitPublicInputs<CircuitTypes<Composer>> pis = {
            .call_context = to_circuit_type(call_context),

            .args = to_ct(args),
            .return_values = to_ct(return_values),

            .emitted_events = to_ct(emitted_events),

            .state_transitions = map(state_transitions, to_circuit_type),
            .state_reads = map(state_reads, to_circuit_type),

            .public_call_stack = to_ct(public_call_stack),
            .l1_msg_stack = to_ct(l1_msg_stack),

            .old_private_data_tree_root = to_ct(old_private_data_tree_root),

            .prover_address = to_ct(prover_address),
        };

        return pis;
    };

    fr hash() const
    {
        auto to_hashes = []<typename T>(const T& e) { return e.hash(); };

        std::vector<fr> inputs;

        // NOTE: we omit the call_context from this hash function, and instead hash it within CallStackItem, for
        // efficiency, so that fewer hashes are needed to 'unwrap' the call_context in the kernel circuit.
        // inputs.push_back(call_context.hash());

        spread_arr_into_vec(args, inputs);
        spread_arr_into_vec(return_values, inputs);

        spread_arr_into_vec(emitted_events, inputs);

        spread_arr_into_vec(map(state_transitions, to_hashes), inputs);
        spread_arr_into_vec(map(state_reads, to_hashes), inputs);

        spread_arr_into_vec(public_call_stack, inputs);
        spread_arr_into_vec(l1_msg_stack, inputs);

        inputs.push_back(old_private_data_tree_root);

        return NCT::compress(inputs, GeneratorIndex::PRIVATE_CIRCUIT_PUBLIC_INPUTS);
    }

    template <size_t SIZE> void spread_arr_into_vec(std::array<fr, SIZE> const& arr, std::vector<fr>& vec) const
    {
        const auto arr_size = sizeof(arr) / sizeof(fr);
        vec.insert(vec.end(), &arr[0], &arr[0] + arr_size);
    }
}; // namespace aztec3::circuits::abis

template <typename NCT> void read(uint8_t const*& it, PublicCircuitPublicInputs<NCT>& private_circuit_public_inputs)
{
    using serialize::read;

    PublicCircuitPublicInputs<NCT>& pis = private_circuit_public_inputs;
    read(it, pis.args);
    read(it, pis.return_values);
    read(it, pis.emitted_events);
    read(it, pis.emitted_ouputs);

    read(it, pis.state_transitions);
    read(it, pis.state_reads);

    read(it, pis.public_call_stack);
    read(it, pis.l1_msg_stack);

    read(it, pis.old_private_data_tree_root);

    read(it, pis.prover_address);
};

template <typename NCT>
void write(std::vector<uint8_t>& buf, PublicCircuitPublicInputs<NCT> const& private_circuit_public_inputs)
{
    using serialize::write;

    PublicCircuitPublicInputs<NCT> const& pis = private_circuit_public_inputs;

    write(buf, pis.args);
    write(buf, pis.return_values);
    write(buf, pis.emitted_events);
    write(buf, pis.emitted_ouputs);

    write(buf, pis.state_transitions);
    write(buf, pis.state_reads);

    write(buf, pis.public_call_stack);
    write(buf, pis.l1_msg_stack);

    write(buf, pis.old_private_data_tree_root);

    write(buf, pis.prover_address);
};

template <typename NCT>
std::ostream& operator<<(std::ostream& os, PublicCircuitPublicInputs<NCT> const& private_circuit_public_inputs)

{
    PublicCircuitPublicInputs<NCT> const& pis = private_circuit_public_inputs;
    return os << "args: " << pis.args << "\n"
              << "return_values: " << pis.return_values << "\n"
              << "emitted_events: " << pis.emitted_events << "\n"

              << "state_transitions: " << pis.state_transitions << "\n"
              << "state_reads: " << pis.state_reads << "\n"

              << "public_call_stack: " << pis.public_call_stack << "\n"
              << "l1_msg_stack: " << pis.l1_msg_stack << "\n"

              << "old_private_data_tree_root: " << pis.old_private_data_tree_root << "\n"

              << "prover_address: " << pis.prover_address << "\n";
}

} // namespace aztec3::circuits::abis