#pragma once
#include <common/container.hpp>
#include <aztec3/constants.hpp>
#include <stdlib/types/convert.hpp>
#include <aztec3/circuits/abis/function_signature.hpp>
#include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>
#include "contract_factory.hpp"
#include "nullifier_preimage.hpp"
#include "oracle_wrapper.hpp"
#include "private_state_note.hpp"
// #include "private_state_var.hpp"
// #include "function.hpp"
// #include "l1_function_interface.hpp"

namespace aztec3::circuits::apps {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using NT = plonk::stdlib::types::NativeTypes;
using aztec3::circuits::abis::FunctionSignature;
using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;

template <typename Composer> class FunctionExecutionContext {
    typedef CircuitTypes<Composer> CT;
    typedef typename CT::fr fr;

  public:
    Composer& composer;
    OracleWrapperInterface<Composer>& oracle;

    Contract<Composer>* contract = nullptr;

    OptionalPrivateCircuitPublicInputs<CT> private_circuit_public_inputs;
    // UnpackedData<CT> unpacked_data;

    std::vector<PrivateStateNote<Composer>> new_private_state_notes;
    std::vector<fr> new_commitments;
    std::vector<NullifierPreimage<CT>> new_nullifier_preimages;
    std::vector<fr> new_nullifiers;

    FunctionExecutionContext<Composer>(Composer& composer, OracleWrapperInterface<Composer>& oracle)
        : composer(composer)
        , oracle(oracle)
        , private_circuit_public_inputs(OptionalPrivateCircuitPublicInputs<CT>::create())
    {
        private_circuit_public_inputs.call_context = oracle.get_call_context();
    }

    void register_contract(Contract<Composer>* contract)
    {
        if (this->contract != nullptr) {
            throw_or_abort("contract already assigned to this FunctionExecutionContext");
        }
        this->contract = contract;
    }

    void push_new_note(PrivateStateNote<Composer> const private_state_note)
    {
        new_private_state_notes.push_back(private_state_note);
    }

    void push_new_nullifier_data(fr nullifier, NullifierPreimage<CT> nullifier_preimage)
    {
        new_nullifiers.push_back(nullifier);
        new_nullifier_preimages.push_back(nullifier_preimage);
    }

    void finalise_private_state_changes()
    {
        if (new_private_state_notes.size() > new_nullifier_preimages.size()) {
            // We've created more commitments than nullifiers so far. But we want to inject an input_nullifier into each
            // new commitment. So, let's create more dummy nullifiers.
            const auto& msg_sender_private_key = oracle.get_msg_sender_private_key();
            for (size_t i = new_nullifier_preimages.size(); i < new_private_state_notes.size(); ++i) {
                auto dummy_commitment = oracle.generate_random_element();
                new_nullifier_preimages.push_back(NullifierPreimage<CT>{
                    dummy_commitment,
                    msg_sender_private_key,
                    false,
                });
                new_nullifiers.push_back(
                    PrivateStateNote<Composer>::compute_dummy_nullifier(dummy_commitment, msg_sender_private_key));
            }
        }
        for (size_t i = 0; i < new_private_state_notes.size(); ++i) {
            new_private_state_notes[i].preimage.input_nullifier = new_nullifiers[i];
            new_commitments.push_back(new_private_state_notes[i].compute_commitment());
        }
    }

    void finalise()
    {
        finalise_private_state_changes();
        private_circuit_public_inputs.set_commitments(new_commitments);
        private_circuit_public_inputs.set_nullifiers(new_nullifiers);
        private_circuit_public_inputs.set_public(composer);
    };
};

} // namespace aztec3::circuits::apps