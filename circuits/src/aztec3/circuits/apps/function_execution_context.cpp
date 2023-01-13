// #include "function_execution_context.hpp"

// #include "contract.hpp"
// #include "oracle_wrapper.hpp"

// #include "notes/note_interface.hpp"

// #include "opcodes/opcodes.hpp"

// #include <aztec3/constants.hpp>

// #include <aztec3/circuits/abis/call_stack_item.hpp>
// #include <aztec3/circuits/abis/function_signature.hpp>
// #include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>

// #include <aztec3/circuits/types/array.hpp>

// #include <common/container.hpp>

// #include <stdlib/types/convert.hpp>

// namespace aztec3::circuits::apps {

// using aztec3::circuits::abis::CallStackItem;
// using aztec3::circuits::abis::CallType;
// using aztec3::circuits::abis::FunctionSignature;
// using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;
// using aztec3::circuits::abis::PrivateCircuitPublicInputs;

// using aztec3::circuits::apps::notes::NoteInterface;
// using aztec3::circuits::apps::opcodes::Opcodes;

// using aztec3::circuits::types::array_push;

// using plonk::stdlib::witness_t;
// using plonk::stdlib::types::CircuitTypes;
// using NT = plonk::stdlib::types::NativeTypes;
// using plonk::stdlib::types::to_nt;

// template <typename Composer>
// CallStackItem<NT, CallType::Private> FunctionExecutionContext<Composer>::get_call_stack_item()
// {
//     const NT::address& actual_contract_address = oracle.native_oracle.get_actual_contract_address();
//     const FunctionSignature<NT>& function_signature = oracle.native_oracle.get_function_signature();

//     return CallStackItem<NT, CallType::Private>{
//         .contract_address = actual_contract_address,
//         .function_signature = function_signature,
//         .public_inputs = get_final_private_circuit_public_inputs(),
//     };
// }

// template <typename Composer>
// std::array<CallStackItem<NT, CallType::Private>, PRIVATE_CALL_STACK_LENGTH> FunctionExecutionContext<
//     Composer>::get_private_call_stack_items()
// {
//     std::array<CallStackItem<NT, CallType::Private>, PRIVATE_CALL_STACK_LENGTH> result;

//     for (size_t i = 0; i < result.size(); ++i) {
//         auto& nested_exec_ctx = nested_private_call_exec_ctxs[i];
//         if (nested_exec_ctx != nullptr) {
//             const NT::address& actual_contract_address =
//                 nested_exec_ctx->oracle.native_oracle.get_actual_contract_address();
//             const FunctionSignature<NT>& function_signature =
//                 nested_exec_ctx->oracle.native_oracle.get_function_signature();

//             result[i] = CallStackItem<NT, CallType::Private>{
//                 .contract_address = actual_contract_address,
//                 .function_signature = function_signature,
//                 .public_inputs = nested_exec_ctx->get_final_private_circuit_public_inputs(),
//             };
//         }
//     }

//     // TODO: do we need to instantiate-with-zeros the structs at the unused indices of `result`?

//     return result;
// }

// template <typename Composer>
// std::array<typename CircuitTypes<Composer>::fr, RETURN_VALUES_LENGTH> FunctionExecutionContext<Composer>::call(
//     typename CircuitTypes<Composer>::address const& external_contract_address,
//     std::string const& external_function_name,
//     std::function<void(FunctionExecutionContext<Composer>&, std::array<NT::fr, ARGS_LENGTH>)> f,
//     std::array<typename CircuitTypes<Composer>::fr, ARGS_LENGTH> const& args)
// {

//     Composer f_composer;

//     // Convert function name to bytes and use the first 4 bytes as the function encoding, for now:
//     std::vector<uint8_t> f_name_bytes(external_function_name.begin(), external_function_name.end());
//     std::vector<uint8_t> f_encoding_bytes(f_name_bytes.begin(), f_name_bytes.begin() + 4);
//     uint32_t f_encoding;
//     memcpy(&f_encoding, f_encoding_bytes.data(), sizeof(f_encoding));

//     const FunctionSignature<NT> f_function_signature{
//         .function_encoding = f_encoding,
//         .is_private = true,
//         .is_constructor = false,
//     };

//     const CallContext<NT> f_call_context{
//         .msg_sender = oracle.get_this_contract_address(), // the sender is `this` contract!
//         .storage_contract_address = external_contract_address,
//         .tx_origin = oracle.get_tx_origin(),
//         .is_delegate_call = false,
//         .is_static_call = false,
//         .is_contract_deployment = false,
//         .reference_block_num = 0,
//     };

//     NativeOracle f_oracle(oracle.oracle.db,
//                           external_contract_address.get_value(),
//                           f_function_signature,
//                           f_call_context,
//                           oracle.get_msg_sender_private_key()
//                               .get_value() // TODO: consider whether a nested function should even be able to access
//                               a
//                                            // private key, given that the call is now coming from a contract (which
//                                            // cannot own a secret), rather than a human.
//     );
//     OracleWrapperInterface<Composer> f_oracle_wrapper(f_composer, f_oracle);

//     // We need an exec_ctx reference which won't go out of scope, so we store a shared_ptr to the newly created
//     // exec_ctx in `this` exec_ctx.
//     auto f_exec_ctx = std::make_shared<FunctionExecutionContext<Composer>>(f_composer, f_oracle_wrapper);

//     array_push(nested_private_call_exec_ctxs, f_exec_ctx);

//     auto native_args = to_nt<Composer>(args);

//     // This calls the function `f`, passing the arguments shown.
//     // The f_exec_ctx will be populated with all the information about that function's execution.
//     std::apply(f, std::forward_as_tuple(*f_exec_ctx, native_args));

//     // Remember: the data held in the f_exec_ctc was built with a different composer than that
//     // of `this` exec_ctx. So we only allow ourselves to get the native types, so that we can consciously declare
//     // circuit types for `this` exec_ctx using `this->composer`.
//     auto& f_public_inputs_nt = f_exec_ctx->final_private_circuit_public_inputs;

//     // Since we've made a call to another function, we now need to push a call_stack_item_hash to `this` function's
//     // private call stack.
//     // Note: we need to constrain some of `this` circuit's variables against f's public inputs:
//     // - args
//     // - return_values
//     // - call_context (TODO: maybe this only needs to be done in the kernel circuit).
//     auto f_public_inputs_ct = f_public_inputs_nt.to_circuit_type(composer);

//     for (size_t i = 0; i < f_public_inputs_ct.args.size(); ++i) {
//         args[i].assert_equal(f_public_inputs_ct.args[i]);
//     }

//     auto call_stack_item_hash = f_public_inputs_ct.hash();

//     array_push<Composer>(private_circuit_public_inputs.private_call_stack, call_stack_item_hash);

//     // The return values are implicitly constrained by being returned as circuit types from this method, for
//     // further use in the circuit. Note: ALL elements of the return_values array MUST be constrained, even if
//     // they're placeholder zeroes.
//     return f_public_inputs_ct.return_values;
// }

// template <typename Composer> void FunctionExecutionContext<Composer>::finalise_utxos()
// {
//     // Copy some vectors, as we can't control whether they'll be pushed-to further, when we call Note methods.
//     auto new_nullifiers_copy = new_nullifiers;

//     size_t used_nullifiers_count = 0;
//     fr next_nullifier;
//     std::vector<fr> new_nonces;

//     // This is almost a visitor pattern. Call methods on each note. The note will choose what to do.
//     for (size_t i = 0; i < new_notes.size(); ++i) {
//         NoteInterface<Composer>& note = *new_notes[i];

//         if (note.needs_nonce()) {
//             const bool next_nullifier_available = new_nullifiers_copy.size() > used_nullifiers_count;

//             if (next_nullifier_available) {
//                 next_nullifier = new_nullifiers_copy[used_nullifiers_count++];
//                 note.set_nonce(next_nullifier);
//             } else {
//                 const fr new_nonce = note.generate_nonce();
//                 new_nonces.push_back(new_nonce);
//             }
//         }

//         new_commitments.push_back(note.get_commitment());
//     }

//     // Push new_nonces to the end of new_nullifiers:
//     std::copy(new_nonces.begin(), new_nonces.end(), std::back_inserter(new_nullifiers));
// }

// template <typename Composer> void FunctionExecutionContext<Composer>::finalise()
// {
//     finalise_utxos();
//     private_circuit_public_inputs.set_commitments(new_commitments);
//     private_circuit_public_inputs.set_nullifiers(new_nullifiers);
//     private_circuit_public_inputs.set_public(composer);
//     final_private_circuit_public_inputs =
//         private_circuit_public_inputs.remove_optionality().template to_native_type<Composer>();
// };

// } // namespace aztec3::circuits::apps