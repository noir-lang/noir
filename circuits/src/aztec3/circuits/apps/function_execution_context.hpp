#pragma once

#include "contract.hpp"
#include "nullifier_preimage.hpp"
#include "oracle_wrapper.hpp"
#include "private_state_note.hpp"

#include "notes/note_interface.hpp"

#include "opcodes/opcodes.hpp"

#include <aztec3/constants.hpp>
#include <aztec3/circuits/abis/function_signature.hpp>
#include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>

#include <common/container.hpp>

#include <stdlib/types/convert.hpp>

// #include <memory>

// #include "private_state_var.hpp"
// #include "function_declaration.hpp"
// #include "l1_function_interface.hpp"

namespace aztec3::circuits::apps {

using aztec3::circuits::abis::FunctionSignature;
using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;

using aztec3::circuits::apps::notes::NoteInterface;

using aztec3::circuits::apps::opcodes::Opcodes;

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using NT = plonk::stdlib::types::NativeTypes;

template <typename Composer> class FunctionExecutionContext {
    typedef CircuitTypes<Composer> CT;
    typedef typename CT::fr fr;

    // We restrict only the opcodes to be able to push to the private members of the exec_ctx.
    // This will just help us build better separation of concerns.
    friend class Opcodes<Composer>;

  public:
    Composer& composer;
    OracleWrapperInterface<Composer>& oracle;

    Contract<Composer>* contract = nullptr;

  private:
    OptionalPrivateCircuitPublicInputs<CT> private_circuit_public_inputs;

    std::vector<std::shared_ptr<NoteInterface<Composer>>> new_notes;
    std::vector<fr> new_commitments;

    // Nullifier preimages can be got from the corresponding Note that they nullify.
    std::vector<NoteInterface<Composer>*> nullified_notes;
    std::vector<fr> new_nullifiers;

  public:
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

    // TODO: consider making this a debug-only method.
    // Not a reference, because we won't want to allow unsafe access. Hmmm, except it's a vector of pointers, so one can
    // still modify the pointers... But at least the original vector isn't being pushed-to or deleted-from.
    std::vector<std::shared_ptr<NoteInterface<Composer>>> get_new_notes() { return new_notes; }
    std::vector<fr> get_new_nullifiers() { return new_nullifiers; }

    void push_new_note(NoteInterface<Composer>* const note_ptr) { new_notes.push_back(note_ptr); }

    void push_newly_nullified_note(NoteInterface<Composer>* note_ptr) { nullified_notes.push_back(note_ptr); }

    /**
     * @brief This is an important optimisation, to save on the number of emitted nullifiers.
     *
     * A nullifier is ideal to serve as a nonce for a new note commitment, because its uniqueness is enforced by the
     * Rollup circuit. But we won't know how many non-dummy nullifiers we have at our disposal (to inject into
     * commitments) until the end of the function.
     *
     * Or to put it another way, at the time we want to create a new commitment (during a function's execution), we
     * would need a nonce. We could certainly query the `exec_ctx` for any nullifiers which have already been created
     * earlier in this function's execution, and we could use one of those. But there might not-yet have been any
     * nullifiers created within the function. Now, at that point, we _could_ generate a dummy nullifier and use that as
     * a nonce. But that uses up a precious slot in the circuit's nullifiers array (part of the circuit's public inputs
     * abi). And it might be the case that later in the function, a load of non-dummy nullifiers get created. So as an
     * optimisation, it would be better if we could use _those_ nullifiers, so as to minimise dummy values in the
     * circuit's public inputs.
     *
     * And so, we provide the option here of deferring the injection of nonces into note_preimages (and hence deferring
     * the computation of each new note commitment) until the very end of the function's execution, when we know how
     * many non-dummy nullifiers we have to play with. If we find this circuit is creating more new commitments than new
     * nullifiers, we can generate some dummy nullifiers at this stage to make up the difference.
     *
     * Note: Using a nullifier as a nonce is a very common and widely-applicable pattern. So much so that it feels
     * acceptable to have this function execute regardless of the underlying Note types being used by the circuit.
     *
     * Note: It's up to the implementer of a custom Note type to decide how a nonce is derived, via the `set_nonce()
     * override` method dictated by the NoteInterface.
     *
     * Note: Not all custom Note types will need a nonce of this kind in their NotePreimage. But they can simply
     * implement an empty body in the `set_nonce() override`.
     */
    void finalise_utxos()
    {
        // if (new_notes.size() > nullified_notes.size()) {
        //     // We've created more commitments than nullifiers so far. But we want to inject an input_nullifier into
        //     each
        //     // new commitment. So, let's create more dummy nullifiers.
        //     const auto& msg_sender_private_key = oracle.get_msg_sender_private_key();
        //     for (size_t i = new_nullifier_preimages.size(); i < new_private_state_notes.size(); ++i) {
        //         auto dummy_commitment = oracle.generate_random_element();
        //         new_nullifier_preimages.push_back(NullifierPreimage<CT>{
        //             dummy_commitment,
        //             msg_sender_private_key,
        //             false,
        //         });
        //         new_nullifiers.push_back(
        //             PrivateStateNote<Composer>::compute_dummy_nullifier(dummy_commitment, msg_sender_private_key));
        //     }
        // }

        // Copy some vectors, as we can't control whether they'll be pushed-to further, when we call Note methods.
        // auto new_commitments_copy = new_commitments;
        // auto new_nullifiers_copy = new_nullifiers;
        // auto new_notes_copy = new_notes;

        // size_t used_nullifiers_count = 0;
        // bool next_nullifier_available = false;
        // bool next_nullifier_used = false;
        // std::optional<fr> next_nullifier;
        // std::vector<fr> new_nonces;

        // for (size_t i = 0; i < new_notes.size(); ++i) {
        //     const& note = new_notes_copy[i];

        //     next_nullifier_available = nullified_notes.size() > used_nullifiers_count;

        //     if (next_nullifier_available) {
        //         next_nullifier = nullified_notes[used_nullifiers_count++].nullifier;
        //         next_nullifier_used = new_notes[i].set_nonce(next_nullifier);
        //     }

        //     if (!next_nullifier_used) {
        //         next_nullifier = nullified_notes.size() > used_nullifiers_count
        //                              ? nullified_notes[used_nullifiers_count++].nullifier
        //                              : std::nullopt; // Indicates that all the existing non-dummy nullifiers have
        //                                              // all been used-up.
        //     } else {
        //         std::optional<fr> new_nonce = new_notes[i].generate_nonce();
        //         if (new_nonce) {
        //             new_nonces.push_back(*new_nonce);
        //         }
        //     }

        //     new_commitments.push_back(new_notes[i].get_commitment());

        //     // Push new_nonces to the end of new_nullifiers:
        //     std::copy(new_nonces.begin(), new_nonces.end(), std::back_inserter(new_nullifiers));
    }

    void finalise()
    {
        finalise_utxos();
        private_circuit_public_inputs.set_commitments(new_commitments);
        private_circuit_public_inputs.set_nullifiers(new_nullifiers);
        private_circuit_public_inputs.set_public(composer);
    };
};

} // namespace aztec3::circuits::apps