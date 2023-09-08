#pragma once

#include "contract.hpp"
#include "oracle_wrapper.hpp"
#include "notes/note_interface.hpp"
#include "opcodes/opcodes.hpp"

#include "aztec3/circuits/abis/call_stack_item.hpp"
#include "aztec3/circuits/abis/function_data.hpp"
#include "aztec3/circuits/abis/private_circuit_public_inputs.hpp"
#include "aztec3/circuits/abis/types.hpp"
#include "aztec3/circuits/hash.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/types/convert.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::apps {

using aztec3::circuits::abis::CallStackItem;
using aztec3::circuits::abis::FunctionData;
using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;
using aztec3::circuits::abis::PrivateCircuitPublicInputs;
using aztec3::circuits::abis::PrivateTypes;

using aztec3::circuits::apps::notes::NoteInterface;
using aztec3::circuits::apps::opcodes::Opcodes;

using plonk::stdlib::array_push;

using aztec3::utils::types::CircuitTypes;
using plonk::stdlib::witness_t;
using NT = aztec3::utils::types::NativeTypes;
using aztec3::utils::types::to_ct;
using aztec3::utils::types::to_nt;

template <typename Builder> class FunctionExecutionContext {
    using NT = NativeTypes;
    using CT = CircuitTypes<Builder>;
    using fr = typename CT::fr;
    using address = typename CT::address;

    // We restrict only the opcodes to be able to push to the private members of the exec_ctx.
    // This will just help us build better separation of concerns.
    friend class Opcodes<Builder>;

  public:
    Builder& builder;
    OracleWrapperInterface<Builder>& oracle;

    Contract<NT>* contract = nullptr;

    std::array<std::shared_ptr<FunctionExecutionContext<Builder>>, MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL>
        nested_private_call_exec_ctxs;

    // TODO: make this private!
    OptionalPrivateCircuitPublicInputs<CT> private_circuit_public_inputs{};

  private:
    std::vector<std::shared_ptr<NoteInterface<Builder>>> new_notes;
    std::vector<fr> new_commitments;

    // Nullifier preimages can be got from the corresponding Note that they nullify.
    std::vector<std::shared_ptr<NoteInterface<Builder>>> nullified_notes;
    std::vector<fr> new_nullifiers;
    std::vector<fr> nullified_commitments;

    PrivateCircuitPublicInputs<NT> final_private_circuit_public_inputs{};

    bool is_finalised = false;

  public:
    FunctionExecutionContext(Builder& builder, OracleWrapperInterface<Builder>& oracle)
        : builder(builder)
        , oracle(oracle)
        , private_circuit_public_inputs(OptionalPrivateCircuitPublicInputs<CT>::create())
    {
        private_circuit_public_inputs.call_context = oracle.get_call_context();
        private_circuit_public_inputs.contract_deployment_data = oracle.get_contract_deployment_data();
    }

    void register_contract(Contract<NT>* contract)
    {
        if (this->contract != nullptr) {
            throw_or_abort("A contract is already assigned to this FunctionExecutionContext");
        }
        this->contract = contract;
    }

    // TODO: consider making this a debug-only method.
    // Not a reference, because we won't want to allow unsafe access. Hmmm, except it's a vector of pointers, so one can
    // still modify the pointers... But at least the original vector isn't being pushed-to or deleted-from.
    std::vector<std::shared_ptr<NoteInterface<Builder>>> get_new_notes() { return new_notes; }

    std::vector<fr> get_new_nullifiers() { return new_nullifiers; }

    void push_new_note(NoteInterface<Builder>* const note_ptr) { new_notes.push_back(note_ptr); }

    void push_newly_nullified_note(NoteInterface<Builder>* note_ptr) { nullified_notes.push_back(note_ptr); }

    PrivateCircuitPublicInputs<NT> get_final_private_circuit_public_inputs()
    {
        // For safety, only return this if the circuit is complete.
        if (!is_finalised) {
            throw_or_abort("You need to call exec_ctx.finalise() in your circuit first.");
        }
        return final_private_circuit_public_inputs;
    }

    /**
     * @brief Get the call_stack_item representing `this` exec_ctx's function call.
     */
    CallStackItem<NT, PrivateTypes> get_call_stack_item()
    {
        const NT::address& actual_contract_address = oracle.native_oracle.get_actual_contract_address();
        const FunctionData<NT>& function_data = oracle.native_oracle.get_function_data();

        return CallStackItem<NT, PrivateTypes>{
            .contract_address = actual_contract_address,
            .function_data = function_data,
            .public_inputs = get_final_private_circuit_public_inputs(),
        };
    }

    /**
     * @brief Get the call_stack_items of any nested function calls made by this exec_ctx's function.
     */
    std::array<CallStackItem<NT, PrivateTypes>, MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL> get_private_call_stack_items()
    {
        std::array<CallStackItem<NT, PrivateTypes>, MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL> result;

        for (size_t i = 0; i < result.size(); ++i) {
            auto& nested_exec_ctx = nested_private_call_exec_ctxs[i];
            if (nested_exec_ctx != nullptr) {
                const NT::address& actual_contract_address =
                    nested_exec_ctx->oracle.native_oracle.get_actual_contract_address();
                const FunctionData<NT>& function_data = nested_exec_ctx->oracle.native_oracle.get_function_data();

                result[i] = CallStackItem<NT, PrivateTypes>{
                    .contract_address = actual_contract_address,
                    .function_data = function_data,
                    .public_inputs = nested_exec_ctx->get_final_private_circuit_public_inputs(),
                };
            }
        }

        // TODO: do we need to instantiate-with-zeros the structs at the unused indices of `result`?

        return result;
    }

    /**
     * @brief Allows a call to be made to a function of another contract
     *
     * TODO: maybe we want to move some of the code that's in this function into a method in the Opcodes class. Although
     * that class was really shoehorned into existence, and is a bit bleurgh.
     */
    std::array<fr, RETURN_VALUES_LENGTH> call(
        address const& f_contract_address,
        std::string const& f_name,
        std::function<void(FunctionExecutionContext<Builder>&, std::vector<NT::fr>)> f,
        std::vector<fr> const& args)
    {
        // Convert function name to bytes and use the first 4 bytes as the function encoding, for now:
        std::vector<uint8_t> f_name_bytes(f_name.begin(), f_name.end());
        std::vector<uint8_t> f_encoding_bytes(f_name_bytes.begin(), f_name_bytes.begin() + 4);
        uint32_t f_encoding = 0;
        memcpy(&f_encoding, f_encoding_bytes.data(), sizeof(f_encoding));

        fr f_encoding_ct = fr(f_encoding);
        // Important Note: we MUST constrain this function_selector value against a fixed selector value. Without the
        // below line, an attacker could pass any f_encoding as a witness.
        f_encoding_ct.convert_constant_to_fixed_witness(&builder);

        /// @dev The above constraining could alternatively be achieved as follows:
        // fr alternative_f_encoding_ct = fr(to_ct(builder, f_encoding));
        // alternative_f_encoding_ct.fix_witness();

        const FunctionData<CT> f_function_data_ct{
            // Note: we MUST
            .selector =
                {
                    .value = f_encoding_ct,
                },
            .is_private = true,
            .is_constructor = false,
        };

        const CallContext<CT> f_call_context_ct{
            .msg_sender = oracle.get_this_contract_address(),  // the sender is `this` contract!
            .storage_contract_address = f_contract_address,
            .portal_contract_address = 0,  // TODO
            .is_delegate_call = false,
            .is_static_call = false,
            .is_contract_deployment = false,
        };

        NativeOracle f_oracle(oracle.native_oracle.db,
                              f_contract_address.get_value(),
                              f_function_data_ct.template to_native_type<Builder>(),
                              f_call_context_ct.template to_native_type<Builder>(),
                              oracle.get_msg_sender_private_key()
                                  .get_value()  // TODO: consider whether a nested function should even be able to
                                                // access a private key, given that the call is now coming from a
                                                // contract (which cannot own a secret), rather than a human.
        );

        Builder f_builder = Builder();

        OracleWrapperInterface<Builder> f_oracle_wrapper(f_builder, f_oracle);

        // We need an exec_ctx reference which won't go out of scope, so we store a shared_ptr to the newly-created
        // exec_ctx in `this` exec_ctx.
        auto f_exec_ctx = std::make_shared<FunctionExecutionContext<Builder>>(f_builder, f_oracle_wrapper);

        array_push(nested_private_call_exec_ctxs, f_exec_ctx);

        auto native_args = to_nt<Builder>(args);

        // This calls the function `f`, passing the arguments shown.
        // The f_exec_ctx will be populated with all the information about that function's execution.
        std::apply(f, std::forward_as_tuple(*f_exec_ctx, native_args));

        // Remember: the data held in the f_exec_ctc was built with a different builder than that
        // of `this` exec_ctx. So we only allow ourselves to get the native types, so that we can consciously declare
        // circuit types for `this` exec_ctx using `this->builder`.
        auto f_public_inputs_nt = f_exec_ctx->get_final_private_circuit_public_inputs();

        // Since we've made a call to another function, we now need to push a call_stack_item_hash to `this` function's
        // private call stack.
        // Note: we need to constrain some of `this` circuit's variables against f's public inputs:
        // - args
        // - return_values
        // - call_context (TODO: maybe this only needs to be done in the kernel circuit).
        auto f_public_inputs_ct = f_public_inputs_nt.to_circuit_type(builder);

        // Constrain that the arguments of the executed function match those we expect:
        auto args_hash_ct = compute_var_args_hash<CT>(args);
        args_hash_ct.assert_equal(f_public_inputs_ct.args_hash);

        CallStackItem<CT, PrivateTypes> const f_call_stack_item_ct{
            .contract_address = f_contract_address,
            .function_data = f_function_data_ct,
            .public_inputs = f_public_inputs_ct,
        };

        auto call_stack_item_hash = f_call_stack_item_ct.hash();

        array_push<Builder>(private_circuit_public_inputs.private_call_stack, call_stack_item_hash);

        // The return values are implicitly constrained by being returned as circuit types from this method, for
        // further use in the circuit. Note: ALL elements of the return_values array MUST be constrained, even if
        // they're placeholder zeroes.
        return f_public_inputs_ct.return_values;
    }

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
     *
     * TODO: Might need some refactoring. Roles between: Opcodes modifying exec_ctx members; and the exec_ctx directly
     * modifying its members, are somewhat blurred at the moment.
     */
    void finalise_utxos()
    {
        // Copy some vectors, as we can't control whether they'll be pushed-to further, when we call Note methods.
        auto new_nullifiers_copy = new_nullifiers;

        size_t used_nullifiers_count = 0;
        fr next_nullifier;
        std::vector<fr> new_nonces;

        // This is almost a visitor pattern. Call methods on each note. The note will choose what to do.
        for (size_t i = 0; i < new_notes.size(); ++i) {
            NoteInterface<Builder>& note = *new_notes[i];

            if (note.needs_nonce()) {
                const bool next_nullifier_available = new_nullifiers_copy.size() > used_nullifiers_count;

                if (next_nullifier_available) {
                    next_nullifier = new_nullifiers_copy[used_nullifiers_count++];
                    note.set_nonce(next_nullifier);
                } else {
                    const fr new_nonce = note.generate_nonce();
                    new_nonces.push_back(new_nonce);
                }
            }

            new_commitments.push_back(note.get_commitment());
        }

        // Push new_nonces to the end of new_nullifiers:
        std::copy(new_nonces.begin(), new_nonces.end(), std::back_inserter(new_nullifiers));
    }

    void finalise()
    {
        finalise_utxos();
        private_circuit_public_inputs.set_commitments(new_commitments);
        private_circuit_public_inputs.set_nullifiers(new_nullifiers);
        private_circuit_public_inputs.set_nullified_commitments(nullified_commitments);
        private_circuit_public_inputs.set_public(builder);
        final_private_circuit_public_inputs =
            private_circuit_public_inputs.remove_optionality().template to_native_type<Builder>();
        is_finalised = true;
    }
};

}  // namespace aztec3::circuits::apps
