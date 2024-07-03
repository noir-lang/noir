use dep::aztec::protocol_types::{
    abis::function_selector::FunctionSelector, address::AztecAddress,
    constants::{
    GENERATOR_INDEX__AUTHWIT_INNER, GENERATOR_INDEX__AUTHWIT_OUTER, GENERATOR_INDEX__AUTHWIT_NULLIFIER,
    CANONICAL_AUTH_REGISTRY_ADDRESS
},
    hash::pedersen_hash
};
use dep::aztec::{prelude::Deserialize, context::{PrivateContext, PublicContext, gas::GasOpts}, hash::hash_args_array};

/**
 * Authenticaion witness helper library
 *
 * Authentication Witness is a scheme for authenticating actions on Aztec, so users can allow third-parties 
 * (e.g. protocols or other users) to execute an action on their behalf.
 * 
 * This library provides helper functions to manage such witnesses.
 * The authentication witness, is some "witness" (data) that authenticates a `message_hash`.
 * The simplest example of an authentication witness, is a signature. The signature is the "evidence",
 * that the signer has seen the message, agrees with it, and has allowed it.
 * It does not need to be a signature. It could be any kind of "proof" that the message is allowed.
 * Another proof could be knowing some kind of secret, or having some kind of "token" that allows the message.
 *
 * The `message_hash` is a hash of the following structure:
 * hash(consumer, chain_id, version, inner_hash)
 * - consumer: the address of the contract that is "consuming" the message,
 * - chain_id: the chain id of the chain that the message is being consumed on,
 * - version: the version of the chain that the message is being consumed on,
 * - inner_hash: the hash of the "inner" message that is being consumed, this is the "actual" message or action.
 *
 * While the `inner_hash` could be anything, such as showing you signed a specific message, it will often be 
 * a hash of the "action" to approve, along with who made the call. As part of this library, we provide a few 
 * helper functions to deal with such messages.
 *
 * For example, we provide helper function that is used for checking that the message is an encoding of the current call.
 * This can be used to let some contract "allow" another contract to act on its behalf, as long as it can 
 * show that it is acting on behalf of the contract.
 *
 * If we take a case of allowing a contract to transfer tokens on behalf of an account, the `inner_hash` can be 
 * derived as:
 * inner_hash = hash(caller, "transfer", hash(to, amount))
 * 
 * Where the `caller` would be the address of the contract that is trying to transfer the tokens, and `to` and `amount`
 * the arguments for the transfer. 
 * 
 * Note that we have both a `caller` and a `consumer`, the `consumer` will be the contract that is consuming the message,
 * in the case of the transfer, it would be the `Token` contract itself, while the caller, will be the actor that is 
 * allowed to transfer the tokens.
 *
 *
 * The authentication mechanism works differently in public and private contexts. In private, we recall that everything
 * is executed on the user's device, so we can use `oracles` to "ask" the user (not contract) for information. In public
 * we cannot do this, since it is executed by the sequencer (someone else). Therefore we can instead use a "registry"
 * to store the messages that we have approved.
 * 
 * A simple example would be a "token" that is being "pulled" from one account into another. We will first outline 
 * how this would look in private, and then in public later. 
 *
 * Say that a user `Alice` wants to deposit some tokens into a DeFi protocol (say a DEX). 
 * `Alice` would make a `deposit` transaction, that she is executing using her account contract. 
 * The account would call the `DeFi` contract to execute `deposit`, which would try to pull funds from the `Token` 
 * contract. Since the `DeFi` contract is trying to pull funds from an account that is not its own, it needs to 
 * convince the `Token` contract that it is allowed to do so. 
 * 
 * This is where the authentication witness comes in! The `Token` contract computes a `message_hash` from the 
 * `transfer` call, and then asks `Alice Account` contract to verify that the `DeFi` contract is allowed to 
 * execute that call.
 * 
 * `Alice Account` contract can then ask `Alice` if she wants to allow the `DeFi` contract to pull funds from her 
 * account. If she does, she will sign the `message_hash` and return the signature to the `Alice Account` which
 * will validate it and return success to the `Token` contract which will then allow the `DeFi` contract to pull
 * funds from `Alice`.
 * 
 * To ensure that the same "approval" cannot be used multiple times, we also compute a `nullifier` for the
 * authentication witness, and emit it from the `Token` contract (consumer).
 * 
 * Note that we can do this flow as we are in private were we can do oracle calls out from contracts. 
 * 
 *
 *  Person          Contract              Contract               Contract
 *  Alice          Alice Account          Token                   DeFi
 *   |                  |                  |                      |
 *   | Defi.deposit(Token, 1000)           |                      |
 *   |----------------->|                  |                      |
 *   |                  | deposit(Token, 1000)                    |
 *   |                  |---------------------------------------->|
 *   |                  |                  |                      |
 *   |                  |                  | transfer(Alice, Defi, 1000)
 *   |                  |                  |<---------------------|
 *   |                  |                  |                      |
 *   |                  | Check if Defi may call transfer(Alice, Defi, 1000)
 *   |                  |<-----------------|                      |
 *   |                  |                  |                      |
 *   | Please give me AuthWit for DeFi     |                      |
 *   | calling transfer(Alice, Defi, 1000) |                      |
 *   |<-----------------|                  |                      |
 *   |                  |                  |                      |
 *   |                  |                  |                      |
 *   | AuthWit for transfer(Alice, Defi, 1000)                    |
 *   |----------------->|                  |                      |
 *   |                  | AuthWit validity |                      |
 *   |                  |----------------->|                      |
 *   |                  |                  |                      |
 *   |                  |       throw if invalid AuthWit          |
 *   |                  |                  |                      |
 *   |                  |       emit AuthWit nullifier            |
 *   |                  |                  |                      |
 *   |                  |       transfer(Alice, Defi, 1000)       |
 *   |                  |                  |                      |
 *   |                  |                  |                      |
 *   |                  |                  | success              |
 *   |                  |                  |--------------------->|
 *   |                  |                  |                      |
 *   |                  |                  |                      |
 *   |                  |                  |           deposit(Token, 1000)
 *   |                  |                  |                      |
 *   |                  |                  |                      |
 *
 *  
 * If we instead were in public, we cannot do the same flow. Instead we would use an authentication registry to store 
 * the messages that we have approved. 
 * 
 * To approve a message, `Alice Account` can make a `set_authorized` call to the registry, to set a `message_hash`
 * as authorized. This is essentially a mapping from `message_hash` to `true` for `Alice Contract`. Every account
 * has its own map in the registry, so `Alice` cannot approve a message for `Bob`.
 *
 * The `Token` contract can then try to "spend" the approval by calling `consume` on the registry. If the message
 * was approved, the value is updated to `false`, and we return the success flag. For more information on the
 * registry, see `main.nr` in `auth_registry_contract`.
 *
 * Person          Contract              Contract            Contract               Contract
 * Alice          Alice Account          Registry             Token                   DeFi
 *   |                  |                    |                   |                      |
 *   | Registry.set_authorized(..., true)    |                   |                      |
 *   |----------------->|                    |                   |                      |
 *   |                  | set_authorized(..., true)              |                      |
 *   |                  |------------------->|                   |                      |
 *   |                  |                    |                   |                      |
 *   |                  |         set authorized to true         |                      |
 *   |                  |                    |                   |                      |
 *   |                  |                    |                   |                      |
 *   | Defi.deposit(Token, 1000)             |                   |                      |
 *   |----------------->|                    |                   |                      |
 *   |                  | deposit(Token, 1000)                   |                      |
 *   |                  |-------------------------------------------------------------->|
 *   |                  |                    |                   |                      |
 *   |                  |                    |              transfer(Alice, Defi, 1000) |
 *   |                  |                    |                   |<---------------------|
 *   |                  |                    |                   |                      |
 *   |                  |                    | Check if Defi may call transfer(Alice, Defi, 1000)
 *   |                  |                    |<------------------|                      |
 *   |                  |                    |                   |                      |
 *   |                  |          throw if invalid AuthWit      |                      |
 *   |                  |                    |                   |                      |
 *   |                  |                    |                   |                      |
 *   |                  |           set authorized to false      |                      |
 *   |                  |                    |                   |                      |
 *   |                  |                    |                   |                      |
 *   |                  |                    | AuthWit validity  |                      |
 *   |                  |                    |------------------>|                      |
 *   |                  |                    |                   |                      |
 *   |                  |                    |                   | transfer(Alice, Defi, 1000)
 *   |                  |                    |                   |<-------------------->|
 *   |                  |                    |                   |                      |
 *   |                  |                    |                   | success              |
 *   |                  |                    |                   |--------------------->|
 *   |                  |                    |                   |                      |
 *   |                  |                    |                   |     deposit(Token, 1000)
 *   |                  |                    |                   |                      |
 *
 *
 * --- FAQ ---
 * Q:   Why are we using a success flag of `keccak256("IS_VALID()")` instead of just returning a boolean?
 * A:   We want to make sure that we don't accidentally return `true` if there is a collision in the function selector.
 *      By returning a hash of `IS_VALID()`, it becomes very unlikely that there is both a colission and we return 
 *      a success flag.
 * 
 * Q:   Why are we using static calls?
 * A:   We are using static calls to ensure that the account contract cannot re-enter. If it was a normal call, it
 *      could make a new call and do a re-entry attack. Using a static ensures that it cannot update any state.
 *
 * Q:   Would it not be cheaper to use a nullifier instead of updating state in public?
 * A:   At a quick glance, a public state update + nullifier is 96 bytes, but two state updates are 128, so it would be 
 *      cheaper to use a nullifier, if this is the way it would always be done. However, if both the approval and the 
 *      consumption is done in the same transaction, then we will be able to squash the updates (only final tx state diff is posted to DA), and now it is cheaper.
 * 
 * Q:   Why is the chain id and the version part of the message hash?
 * A:   The chain id and the version is part of the message hash to ensure that the message is only valid on a specific
 *      chain to avoid a case where the same message could be used across multiple chains. 
 */

global IS_VALID_SELECTOR = 0xabf64ad4; // 4 first bytes of keccak256("IS_VALID()")

/** 
 * Assert that `on_behalf_of` have authorized the current call with a valid authentication witness
 * 
 * Computing the `inner_hash` using the `msg_sender`, `selector` and `args_hash` and then making a call out to the 
 * `on_behalf_of` contract to verify that the `inner_hash` is valid.
 * 
 * @param on_behalf_of The address that have authorized the current call
 */
// docs:start:assert_current_call_valid_authwit
pub fn assert_current_call_valid_authwit(context: &mut PrivateContext, on_behalf_of: AztecAddress) {
    let inner_hash = compute_inner_authwit_hash([context.msg_sender().to_field(), context.selector().to_field(), context.args_hash]);
    assert_inner_hash_valid_authwit(context, on_behalf_of, inner_hash);
}
// docs:end:assert_current_call_valid_authwit

/** 
 * Assert that a specific `inner_hash` is valid for the `on_behalf_of` address
 * 
 * Used as an internal function for `assert_current_call_valid_authwit` and can be used as a standalone function when
 * the `inner_hash` is from a different source, e.g., say a block of text etc.
 * 
 * @param on_behalf_of The address that have authorized the current call
 * @param inner_hash The hash of the message to authorize  
 */
pub fn assert_inner_hash_valid_authwit(context: &mut PrivateContext, on_behalf_of: AztecAddress, inner_hash: Field) {
    // We perform a static call here and not a standard one to ensure that the account contract cannot re-enter.
    let result: Field = context.static_call_private_function(
        on_behalf_of,
        FunctionSelector::from_signature("verify_private_authwit(Field)"),
        [inner_hash]
    ).unpack_into();
    assert(result == IS_VALID_SELECTOR, "Message not authorized by account");
    // Compute the nullifier, similar computation to the outer hash, but without the chain_id and version.
    // Those should already be handled in the verification, so we just need something to nullify, that allow same inner_hash for multiple actors.
    let nullifier = compute_authwit_nullifier(on_behalf_of, inner_hash);
    context.push_nullifier(nullifier, 0);
}

/** 
 * Assert that `on_behalf_of` have authorized the current call in the authentication registry
 * 
 * Computing the `inner_hash` using the `msg_sender`, `selector` and `args_hash` and then making a call out to the 
 * `on_behalf_of` contract to verify that the `inner_hash` is valid.
 * 
 * Note that the authentication registry will take the `msg_sender` into account as the consumer, so this will only
 * work if the `msg_sender` is the same as the `consumer` when the `message_hash` was inserted into the registry.
 * 
 * @param on_behalf_of The address that have authorized the current call
 */
// docs:start:assert_current_call_valid_authwit_public
pub fn assert_current_call_valid_authwit_public(context: &mut PublicContext, on_behalf_of: AztecAddress) {
    let inner_hash = compute_inner_authwit_hash(
        [(*context).msg_sender().to_field(), (*context).selector().to_field(), (*context).get_args_hash()]
    );
    assert_inner_hash_valid_authwit_public(context, on_behalf_of, inner_hash);
}
// docs:end:assert_current_call_valid_authwit_public

/** 
 * Assert that `on_behalf_of` have authorized a speicifc `inner_hash` in the authentication registry
 * 
 * Computing the `inner_hash` using the `msg_sender`, `selector` and `args_hash` and then making a call out to the 
 * `on_behalf_of` contract to verify that the `inner_hash` is valid.
 *
 * Note that the authentication registry will take the `msg_sender` into account as the consumer, so this will only
 * work if the `msg_sender` is the same as the `consumer` when the `message_hash` was inserted into the registry.
 * 
 * @param on_behalf_of The address that have authorized the the `inner_hash`
 */
pub fn assert_inner_hash_valid_authwit_public(context: &mut PublicContext, on_behalf_of: AztecAddress, inner_hash: Field) {
    let result: Field = context.call_public_function(
        AztecAddress::from_field(CANONICAL_AUTH_REGISTRY_ADDRESS),
        FunctionSelector::from_signature("consume((Field),Field)"),
        [on_behalf_of.to_field(), inner_hash].as_slice(),
        GasOpts::default()
    ).deserialize_into();
    assert(result == IS_VALID_SELECTOR, "Message not authorized by account");
}

/**
 * Compute the `message_hash` from a function call to be used by an authentication witness 
 * 
 * Useful for when you need a non-account contract to approve during execution. For example if you need a contract
 * to make a call to nested contract, e.g., contract A wants to exit token T to L1 using bridge B, so it needs to allow
 * B to transfer T on its behalf.
 * 
 * @param caller The address of the contract that is calling the function, in the example above, this would be B
 * @param consumer The address of the contract that is consuming the message, in the example above, this would be T
 * @param chain_id The chain id of the chain that the message is being consumed on
 * @param version The version of the chain that the message is being consumed on
 * @param selector The function selector of the function that is being called
 * @param args The arguments of the function that is being called
 */
// docs:start:compute_authwit_message_hash_from_call
pub fn compute_authwit_message_hash_from_call<N>(
    caller: AztecAddress,
    consumer: AztecAddress,
    chain_id: Field,
    version: Field,
    selector: FunctionSelector,
    args: [Field; N]
) -> Field {
    let args_hash = hash_args_array(args);
    let inner_hash = compute_inner_authwit_hash([caller.to_field(), selector.to_field(), args_hash]);
    compute_authwit_message_hash(consumer, chain_id, version, inner_hash)
}
// docs:end:compute_authwit_message_hash_from_call

/**
 * Computes the `inner_hash` of the authentication witness
 * 
 * This is used internally, but also useful in cases where you want to compute the `inner_hash` for a specific message
 * that is not necessarily a call, but just some "bytes" or text.
 * 
 * @param args The arguments to hash
 */
pub fn compute_inner_authwit_hash<N>(args: [Field; N]) -> Field {
    pedersen_hash(args, GENERATOR_INDEX__AUTHWIT_INNER)
}

/** 
 * Computs the `authwit_nullifier` for a specific `on_behalf_of` and `inner_hash`
 * 
 * Using the `on_behalf_of` and the `inner_hash` to ensure that the nullifier is siloed for a specific `on_behalf_of`.
 * 
 * @param on_behalf_of The address that have authorized the the `inner_hash`
 * @param inner_hash The hash of the message to authorize
 */
pub fn compute_authwit_nullifier(on_behalf_of: AztecAddress, inner_hash: Field) -> Field {
    pedersen_hash(
        [on_behalf_of.to_field(), inner_hash],
        GENERATOR_INDEX__AUTHWIT_NULLIFIER
    )
}

/**
 * Computes the `message_hash` for the authentication witness
 * 
 * @param consumer The address of the contract that is consuming the message
 * @param chain_id The chain id of the chain that the message is being consumed on
 * @param version The version of the chain that the message is being consumed on
 * @param inner_hash The hash of the "inner" message that is being consumed
 */
pub fn compute_authwit_message_hash(consumer: AztecAddress, chain_id: Field, version: Field, inner_hash: Field) -> Field {
    pedersen_hash(
        [
        consumer.to_field(),
        chain_id,
        version,
        inner_hash
    ],
        GENERATOR_INDEX__AUTHWIT_OUTER
    )
}

/**
 * Helper function to set the authorization status of a message hash
 * 
 * Wraps a public call to the authentication registry to set the authorization status of a `message_hash`
 * 
 * @param message_hash The hash of the message to authorize
 * @param authorize True if the message should be authorized, false if it should be revoked
 */
pub fn set_authorized(context: &mut PublicContext, message_hash: Field, authorize: bool) {
    context.call_public_function(
        AztecAddress::from_field(CANONICAL_AUTH_REGISTRY_ADDRESS),
        FunctionSelector::from_signature("set_authorized(Field,bool)"),
        [message_hash, authorize as Field].as_slice(),
        GasOpts::default()
    ).assert_empty();
}

/**
 * Helper function to reject all authwits
 * 
 * Wraps a public call to the authentication registry to set the `reject_all` flag
 *
 * @param reject True if all authwits should be rejected, false otherwise 
 */
pub fn set_reject_all(context: &mut PublicContext, reject: bool) {
    context.call_public_function(
        AztecAddress::from_field(CANONICAL_AUTH_REGISTRY_ADDRESS),
        FunctionSelector::from_signature("set_reject_all(bool)"),
        [context.this_address().to_field(), reject as Field].as_slice(),
        GasOpts::default()
    ).assert_empty();
}
