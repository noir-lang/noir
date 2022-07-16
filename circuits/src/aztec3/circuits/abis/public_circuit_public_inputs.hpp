#pragma once
// #include <stdlib/hash/pedersen/pedersen.hpp>
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <common/map.hpp>
#include "../../constants.hpp"
#include "state_transition.hpp"
#include "state_read.hpp"
// #include "./executed_callback.hpp"
// #include "./callback_stack_item.hpp"

namespace aztec3::circuits::abis {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename NCT> struct PublicCircuitPublicInputs {
    typedef typename NCT::fr fr;
    typedef typename NCT::boolean boolean;
    typedef typename NCT::address address;

    // address msg_sender; // TODO!

    std::array<fr, CUSTOM_INPUTS_LENGTH> custom_inputs;
    std::array<fr, CUSTOM_OUTPUTS_LENGTH> custom_outputs;

    std::array<fr, EMITTED_PUBLIC_INPUTS_LENGTH> emitted_public_inputs;
    std::array<fr, EMITTED_OUTPUTS_LENGTH> emitted_outputs;

    ExecutedCallback<NCT> executed_callback;

    std::array<StateTransition<NCT>, STATE_TRANSITIONS_LENGTH> state_transitions;
    std::array<StateRead<NCT>, STATE_READS_LENGTH> state_reads;

    std::array<fr, PUBLIC_CALL_STACK_LENGTH> public_call_stack;
    std::array<fr, CONTRACT_DEPLOYMENT_CALL_STACK_LENGTH> contract_deployment_call_stack;
    std::array<fr, PARTIAL_L1_CALL_STACK_LENGTH> partial_l1_call_stack;
    std::array<CallbackStackItem<NCT>, CALLBACK_STACK_LENGTH> callback_stack;

    fr old_private_data_tree_root;

    address prover_address = 0;

    boolean is_fee_payment = false;
    boolean pay_fee_from_l1 = false;
    boolean called_from_l1 = false;

    bool operator==(PublicCircuitPublicInputs<NCT> const&) const = default;

    static PublicCircuitPublicInputs<NCT> empty()
    {
        PublicCircuitPublicInputs<NCT> pis = {
            std::array<fr, CUSTOM_INPUTS_LENGTH>::fill(0),
            std::array<fr, CUSTOM_OUTPUTS_LENGTH>::fill(0),

            std::array<fr, EMITTED_PUBLIC_INPUTS_LENGTH>::fill(0),
            std::array<fr, EMITTED_OUTPUTS_LENGTH>::fill(0),

            ExecutedCallback<NCT>::empty(),

            std::array<StateTransition<NCT>, STATE_TRANSITIONS_LENGTH>::fill(StateTransition<NCT>::empty()),
            std::array<StateRead<NCT>, STATE_READS_LENGTH>::fill(StateRead<NCT>::empty()),

            std::array<fr, PUBLIC_CALL_STACK_LENGTH>::fill(0),

            std::array<fr, CONTRACT_DEPLOYMENT_CALL_STACK_LENGTH>::fill(0),
            std::array<fr, PARTIAL_L1_CALL_STACK_LENGTH>::fill(0),
            std::array<CallbackStackItem<NCT>, CALLBACK_STACK_LENGTH>::fill(CallbackStackItem<NCT>::empty()),

            .old_private_data_tree_root = 0,
        };
        return pis;
    };

    template <typename Composer>
    PublicCircuitPublicInputs<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };
        auto to_circuit_type = [&](auto& e) { return e.to_circuit_type(composer); };

        PublicCircuitPublicInputs<CircuitTypes<Composer>> pis = {
            .custom_inputs = to_ct(custom_inputs),
            .custom_outputs = to_ct(custom_outputs),
            .emitted_public_inputs = to_ct(emitted_public_inputs),
            .emitted_outputs = to_ct(emitted_outputs),

            .executed_callback = to_circuit_type(executed_callback),

            .state_transitions = map(state_transitions, to_circuit_type),
            .state_reads = map(state_reads, to_circuit_type),

            .public_call_stack = to_ct(public_call_stack),
            .contract_deployment_call_stack = to_ct(contract_deployment_call_stack),
            .partial_l1_call_stack = to_ct(partial_l1_call_stack),
            .callback_stack = map(callback_stack, to_circuit_type),

            .prover_address = to_ct(prover_address),

            .is_fee_payment = to_ct(is_fee_payment),
            .pay_fee_from_l1 = to_ct(pay_fee_from_l1),
            .called_from_l1 = to_ct(called_from_l1),
        };

        return pis;
    };
};

template <typename NCT> void read(uint8_t const*& it, PublicCircuitPublicInputs<NCT>& private_circuit_public_inputs)
{
    using serialize::read;

    PublicCircuitPublicInputs<NCT>& pis = private_circuit_public_inputs;
    read(it, pis.custom_inputs);
    read(it, pis.custom_outputs);
    read(it, pis.emitted_public_inputs);
    read(it, pis.emitted_ouputs);
    read(it, pis.executed_callback);
    read(it, pis.state_transitions);
    read(it, pis.state_reads);
    read(it, pis.public_call_stack);
    read(it, pis.contract_deployment_call_stack);
    read(it, pis.partial_l1_call_stack);
    read(it, pis.callback_stack);
    read(it, pis.old_private_data_tree_root);
    read(it, pis.is_fee_payment);
    read(it, pis.pay_fee_from_l1);
    read(it, pis.called_from_l1);
};

template <typename NCT>
void write(std::vector<uint8_t>& buf, PublicCircuitPublicInputs<NCT> const& private_circuit_public_inputs)
{
    using serialize::write;

    PublicCircuitPublicInputs<NCT> const& pis = private_circuit_public_inputs;

    write(buf, pis.custom_inputs);
    write(buf, pis.custom_outputs);
    write(buf, pis.emitted_public_inputs);
    write(buf, pis.emitted_ouputs);
    write(buf, pis.executed_callback);
    write(buf, pis.state_transitions);
    write(buf, pis.state_reads);
    write(buf, pis.public_call_stack);
    write(buf, pis.contract_deployment_call_stack);
    write(buf, pis.partial_l1_call_stack);
    write(buf, pis.callback_stack);
    write(buf, pis.old_private_data_tree_root);
    write(buf, pis.is_fee_payment);
    write(buf, pis.pay_fee_from_l1);
    write(buf, pis.called_from_l1);
};

template <typename NCT>
std::ostream& operator<<(std::ostream& os, PublicCircuitPublicInputs<NCT> const& private_circuit_public_inputs)

{
    PublicCircuitPublicInputs<NCT> const& pis = private_circuit_public_inputs;
    return os << "custom_inputs: " << pis.custom_inputs << "\n"
              << "custom_outputs: " << pis.custom_outputs << "\n"
              << "emitted_public_inputs: " << pis.emitted_public_inputs << "\n"
              << "emitted_outputs: " << pis.emitted_outputs << "\n"
              << "executed_callback: " << pis.executed_callback << "\n"
              << "state_transitions: " << pis.state_transitions << "\n"
              << "state_reads: " << pis.state_reads << "\n"
              << "public_call_stack: " << pis.public_call_stack << "\n"
              << "contract_deployment_call_stack: " << pis.contract_deployment_call_stack << "\n"
              << "partial_l1_call_stack: " << pis.partial_l1_call_stack << "\n"
              << "callback_stack: " << pis.callback_stack << "\n"
              << "old_private_data_tree_root: " << pis.old_private_data_tree_root << "\n"
              << "is_fee_payment: " << pis.is_fee_payment << "\n"
              << "pay_fee_from_l1: " << pis.pay_fee_from_l1 << "\n"
              << "called_from_l1: " << pis.called_from_l1 << "\n";
}

} // namespace aztec3::circuits::abis