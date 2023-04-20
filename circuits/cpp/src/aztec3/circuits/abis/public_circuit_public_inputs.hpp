#pragma once

#include "call_context.hpp"
#include "state_transition.hpp"
#include "state_read.hpp"
#include "../../constants.hpp"

#include <barretenberg/common/map.hpp>
#include <barretenberg/stdlib/primitives/witness/witness.hpp>
#include <aztec3/utils/array.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/circuit_types.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::zero_array;
using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;

template <typename NCT> struct PublicCircuitPublicInputs {
    typedef typename NCT::fr fr;
    typedef typename NCT::boolean boolean;
    typedef typename NCT::address address;

    CallContext<NCT> call_context{};

    std::array<fr, ARGS_LENGTH> args = zero_array<fr, ARGS_LENGTH>();
    std::array<fr, RETURN_VALUES_LENGTH> return_values = zero_array<fr, RETURN_VALUES_LENGTH>();

    std::array<fr, EMITTED_EVENTS_LENGTH> emitted_events = zero_array<fr, EMITTED_EVENTS_LENGTH>();

    std::array<StateTransition<NCT>, STATE_TRANSITIONS_LENGTH> state_transitions{};
    std::array<StateRead<NCT>, STATE_READS_LENGTH> state_reads{};

    std::array<fr, PUBLIC_CALL_STACK_LENGTH> public_call_stack = zero_array<fr, PUBLIC_CALL_STACK_LENGTH>();
    std::array<fr, L1_MSG_STACK_LENGTH> l1_msg_stack = zero_array<fr, L1_MSG_STACK_LENGTH>();

    fr historic_public_data_tree_root;

    address prover_address;

    boolean operator==(PublicCircuitPublicInputs<NCT> const& other) const
    {
        return call_context == other.call_context && args == other.args && return_values == other.return_values &&
               emitted_events == other.emitted_events && state_transitions == other.state_transitions &&
               state_reads == other.state_reads && public_call_stack == other.public_call_stack &&
               l1_msg_stack == other.l1_msg_stack &&
               historic_public_data_tree_root == other.historic_public_data_tree_root &&
               prover_address == other.prover_address;
    };

    template <typename Composer>
    PublicCircuitPublicInputs<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(composer, e); };
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

            .historic_public_data_tree_root = to_ct(historic_public_data_tree_root),

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

        inputs.push_back(historic_public_data_tree_root);

        return NCT::compress(inputs, GeneratorIndex::PUBLIC_CIRCUIT_PUBLIC_INPUTS);
    }

    template <size_t SIZE> void spread_arr_into_vec(std::array<fr, SIZE> const& arr, std::vector<fr>& vec) const
    {
        const auto arr_size = sizeof(arr) / sizeof(fr);
        vec.insert(vec.end(), &arr[0], &arr[0] + arr_size);
    }
}; // namespace aztec3::circuits::abis

template <typename NCT> void read(uint8_t const*& it, PublicCircuitPublicInputs<NCT>& public_circuit_public_inputs)
{
    using serialize::read;

    PublicCircuitPublicInputs<NCT>& pis = public_circuit_public_inputs;
    read(it, pis.call_context);
    read(it, pis.args);
    read(it, pis.return_values);
    read(it, pis.emitted_events);

    read(it, pis.state_transitions);
    read(it, pis.state_reads);

    read(it, pis.public_call_stack);
    read(it, pis.l1_msg_stack);

    read(it, pis.historic_public_data_tree_root);

    read(it, pis.prover_address);
};

template <typename NCT>
void write(std::vector<uint8_t>& buf, PublicCircuitPublicInputs<NCT> const& public_circuit_public_inputs)
{
    using serialize::write;

    PublicCircuitPublicInputs<NCT> const& pis = public_circuit_public_inputs;

    write(buf, pis.call_context);
    write(buf, pis.args);
    write(buf, pis.return_values);
    write(buf, pis.emitted_events);

    write(buf, pis.state_transitions);
    write(buf, pis.state_reads);

    write(buf, pis.public_call_stack);
    write(buf, pis.l1_msg_stack);

    write(buf, pis.historic_public_data_tree_root);

    write(buf, pis.prover_address);
};

template <typename NCT>
std::ostream& operator<<(std::ostream& os, PublicCircuitPublicInputs<NCT> const& public_circuit_public_inputs)

{
    PublicCircuitPublicInputs<NCT> const& pis = public_circuit_public_inputs;
    return os << "call_context: " << pis.call_context << "\n"
              << "args: " << pis.args << "\n"
              << "return_values: " << pis.return_values << "\n"
              << "emitted_events: " << pis.emitted_events << "\n"

              << "state_transitions: " << pis.state_transitions << "\n"
              << "state_reads: " << pis.state_reads << "\n"

              << "public_call_stack: " << pis.public_call_stack << "\n"
              << "l1_msg_stack: " << pis.l1_msg_stack << "\n"

              << "historic_public_data_tree_root: " << pis.historic_public_data_tree_root << "\n"

              << "prover_address: " << pis.prover_address << "\n";
}

} // namespace aztec3::circuits::abis