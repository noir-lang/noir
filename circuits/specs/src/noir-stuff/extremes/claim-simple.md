A real-world (ish) example of making an Aztec-Connect-like claim.

# High-level

```js
/********************************************************************************
 * ON-CHAIN SOLIDITY:
 *******************************************************************************/
// THIS FUNCTION IS SOLIDITY, IMPORTED SOMEHOW...
// Suppose this is deployed at L1 Portal Contract address 0xabc123.
contract L1_Portal_Contract {
    function l1_swap_a_for_b(uint total_deposit_amount_a) {
        // do an on-chain swap of amount_a for some amount_b
        return total_amount_b; // This result is added to a results tree through
                               // Aztec 3's architecture.
    }
}

/********************************************************************************
 * CIRCUITS:
 *******************************************************************************/

// Suppose this is deployed at contract address 0xdef456.
/**
 * @dev This EXAMPLE contract is a hugely simplified version of a defi bridge.
 * Specifically, it facilitates swaps from asset A to asset B (but not the other
 * was around, for simplicity!).
 * The main illustration of this example is the `secret storage slot` pointer ideas.
 */
contract My_Contract_1 {
    secret mapping(address => uint) token_balances_a; // user => balance
    secret mapping(address => uint) token_balances_b; // user => balance
    // interaction_id => (secret_slot => amount):
    mapping(uint => uint) total_deposits_a; // interaction_id => amount
    mapping(uint => uint) claimable_amounts_a; // interaction_id => amount
    mapping(uint => uint) claimable_amounts_b; // interaction_id => amount
    uint interaction_id;
    uint deposit_counter; // A counter for the currently pending interaction's num
                          // pending deposits so far.
    struct DepositLog = {
        uint deposited_amount;
        uint* token_balance_a_addend_slot; // PLACEHOLDER SYNTAX (discussion welcome!)
        uint* token_balance_b_addend_slot; // "pointer" decorator (like c++).
        // Points to a 'secret storage slot': the pedersen hash of a 
        // `(storageSlot, owner, secret)` tuple.
        //
        // The motivation behind this syntax is to allow some _other_ user to inject
        // a value into a secret storage slot, without them learning the owner,
        // secret, or (in the case of mappings/arrays) the storageSlot.
    }
    mapping(uint => mapping(secret_slot => DepositLog)) deposit_logs_a; 


    // Private function because it edits a private state.
    function user_deposit_a(amount_a) {
        token_balances_a[msg_sender] -= amount_a;
        add_pending_deposit_a(
            amount_a,
            &token_balances_a[msg_sender], // PLACEHOLDER SYNTAX (discussion welcome!)
            &token_balances_b[msg_sender]  // Like the "address of" operator of c++.
            // Or, more accurately, the "secret storage slot of" operator.
            //
            // The motivation behind this syntax is to allow some _other_ user to inject
            // a value into a secret storage slot, without them learning the owner,
            // secret, or (in the case of mappings/arrays) the storageSlot.
            //
            // In this example, since `token_balances_a[msg_sender]` is a 
            // 'partitioned' state (a UTXO state), this `&` operator will return the 
            // pedersen hash of a `storageSlot, owner, secret` tuple, provided by the
            // caller.
            //
            // This `&` operator would only be usable on secret states within a private
            // function.
        );
    }

    // Public function because it edits a public state.
    function add_pending_deposit_a(
        uint amount_a,
        uint* token_balance_a_addend_slot,
        uint* token_balance_b_addend_slot
    ) internal {
        deposit_logs_a[interaction_id][deposit_counter++] = DepositLog(
            amount_a,
            token_balance_a_addend_slot,
            token_balance_b_addend_slot
        );

        total_deposits_a[interaction_id] += amount_a;
    }

    function process_pending_deposits_a() external {
        uint total_deposit_amount_a = total_deposits_a[interaction_id];
        
        // An L2 function can _only_ make L1 calls to its own L1 Portal Contract
        // (and not to any other L1 contract).
        // Here's some pseudocode for a special 'L1Promise' type.
        // Kind of like Rust's. Kind of like handling a JS Promise.
        // Please make the syntax better!
        // An L1 call (from L2) always must return an L1Promise.
        L1Promise promise = L1_Portal_Contract.l1_swap_a_for_b(total_deposit_amount_a);

        promise.then(
            result => permit_claims_b(interaction_id, result[0]),
            // On success, the result will effectively be an array of values.
            // Note: a result value cannot be passed to a 'secret' arg's position.
            permit_claims_a(interaction_id, total_deposit_amount_a), 
            // On failure, nothing is returned by L1
            // (not even an error message), so the failure
            // callback is executed with existing args.
        );

        ++interaction_id;
    }

    // TODO: discuss this. This _would_ technically have to be called by a user
    // or rollup provider, but we only want them to be able to call it as part of a
    // callback.
    // I put the word 'internal', but that's not quite right, perhaps.
    function permit_claims_b(
        uint historic_interaction_id,
        uint total_amount_b
    ) internal {
        claimable_amounts_b[historic_interaction_id] += total_amount_b;
    }

    function permit_claims_a(
        uint historic_interaction_id,
        uint total_amount_a
    ) internal {
        claimable_amounts_a[historic_interaction_id] += total_amount_a;
    }

    function make_claim_a(
        uint historic_interaction_id,
        uint historic_deposit_counter,
    ) external {
        DepositLog deposit_log = 
            deposit_logs_a[historic_interaction_id][historic_deposit_counter];
        deposited_amount_a = deposit_log.deposited_amount;

        uint claimable_amount_a =
            claimable_amounts_a[historic_interaction_id] *
            (
                deposited_amount_a /
                total_deposits_a[historic_interaction_id]
            );

        // PLACEHOLDER SYNTAX (discussion welcome!)
        // "contents of" operator (like the c++ dereference operator).
        // 'Completes' the pedersen commitment by adding a value to the 
        // 'secret storage slot' pointed-to by the pointer.
        *deposit_log.token_balance_a_addend_slot = claimable_amount_a;

        // Prevent user from claiming again:
        delete deposit_logs_a[historic_interaction_id][historic_deposit_counter];
    }

    function make_claim_b(
        uint historic_interaction_id,
        uint historic_deposit_counter,
    ) external {
        DepositLog deposit_log = 
            deposit_logs_a[historic_interaction_id][historic_deposit_counter];
        deposited_amount_a = deposit_log.deposited_amount;

        uint claimable_amount_b =
            claimable_amounts_b[historic_interaction_id] *
            (
                deposited_amount_a /
                total_deposits_a[historic_interaction_id]
            );

        // PLACEHOLDER SYNTAX (discussion welcome!)
        // "contents of" operator (like the c++ dereference operator).
        // 'Completes' the pedersen commitment by adding a value to the 
        // 'secret storage slot' pointed-to by the pointer.
        *deposit_log.token_balance_b_addend_slot = claimable_amount_b;

        // Prevent user from claiming again:
        delete deposit_logs_a[historic_interaction_id][historic_deposit_counter];
    }
}
```

# Low-level

## `user_deposit_a()`

TODO. Very big, complex example. Will take time.