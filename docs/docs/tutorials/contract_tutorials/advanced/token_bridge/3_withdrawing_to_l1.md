---
title: Withdrawing to L1
---

This is where we have tokens on Aztec and want to withdraw them back to L1 (i.e. burn them on L2 and mint on L1). Withdrawing from L1 will be public.

## Withdrawing publicly

Go back to your `main.nr` and paste this:

#include_code exit_to_l1_public /noir-projects/noir-contracts/contracts/token_bridge_contract/src/main.nr rust

For this to work we import the `get_withdraw_content_hash` helper function from the `token_portal_content_hash_lib`.

**What’s happening here?**

The `exit_to_l1_public` function enables anyone to withdraw their L2 tokens back to L1 publicly. This is done by burning tokens on L2 and then creating an L2->L1 message.

1. Like with our deposit function, we need to create the L2 to L1 message. The content is the _amount_ to burn, the recipient address, and who can execute the withdraw on the L1 portal on behalf of the user. It can be `0x0` for anyone, or a specified address.
2. `context.message_portal()` passes this content to the [kernel circuit](../../../../aztec/concepts/circuits/kernels/public_kernel.md) which creates the proof for the transaction. The kernel circuit then adds the sender (the L2 address of the bridge + version of aztec) and the recipient (the portal to the L2 address + the chain ID of L1) under the hood, to create the message which gets added as part of the transaction data published by the sequencer and is stored in the outbox for consumption.
3. The `context.message_portal()` takes the recipient and content as input, and will insert a message into the outbox. We set the recipient to be the portal address read from storage of the contract.
4. Finally, you also burn the tokens on L2! Note that it burning is done at the end to follow the check effects interaction pattern. Note that the caller has to first approve the bridge contract to burn tokens on its behalf. Refer to [burn_public function on the token contract](../../token_contract.md#authorizing-token-spends).
   - We burn the tokens from the `msg_sender()`. Otherwise, a malicious user could burn someone else’s tokens and mint tokens on L1 to themselves. One could add another approval flow on the bridge but that might make it complex for other applications to call the bridge.

## Withdrawing Privately

This function works very similarly to the public version, except here we burn user’s private notes. Under the public function in your `main.nr`, paste this:

#include_code exit_to_l1_private /noir-projects/noir-contracts/contracts/token_bridge_contract/src/main.nr rust

#include_code assert_token_is_same /noir-projects/noir-contracts/contracts/token_bridge_contract/src/main.nr rust

Since this is a private method, it can't read what token is publicly stored. So instead the user passes a token address, and `_assert_token_is_same()` checks that this user provided address is same as the one in storage.

Because public functions are executed by the sequencer while private methods are executed locally, all public calls are always done _after_ all private calls are completed. So first the burn would happen and only later the sequencer asserts that the token is same. The sequencer just sees a request to `execute_assert_token_is_same` and therefore has no context on what the appropriate private method was. If the assertion fails, then the kernel circuit will fail to create a proof and hence the transaction will be dropped.

Once again, a user must sign an approval message to let the contract burn tokens on their behalf. The nonce refers to this approval message.

For both the public and private flow, we use the same mechanism to determine the content hash. This is because on L1, things are public anyway. The only different between the two functions is that in the private domain we have to nullify user’s notes where as in the public domain we subtract the balance from the user.

## Withdrawing on L1

After the transaction is completed on L2, the portal must call the outbox to successfully transfer funds to the user on L1. Like with deposits, things can be complex here. For example, what happens if the transaction was done on L2 to burn tokens but can’t be withdrawn to L1? Then the funds are lost forever! How do we prevent this?

Paste this in your `TokenPortal.sol`:

```solidity
#include_code token_portal_withdraw /l1-contracts/test/portals/TokenPortal.sol raw
}
```

Here we reconstruct the L2 to L1 message and check that this message exists on the outbox. If so, we consume it and transfer the funds to the recipient. As part of the reconstruction, the content hash looks similar to what we did in our bridge contract on aztec where we pass the amount and recipient to the hash. This way a malicious actor can’t change the recipient parameter to the address and withdraw funds to themselves.

We also use a `_withCaller` parameter to determine the appropriate party that can execute this function on behalf of the recipient. If `withCaller` is false, then anyone can call the method and hence we use address(0), otherwise only msg.sender should be able to execute. This address should match the `callerOnL1` address we passed in aztec when withdrawing from L2.

We call this pattern _designed caller_ which enables a new paradigm **where we can construct other such portals that talk to the token portal and therefore create more seamless crosschain legos** between L1 and L2.

Before we can compile and use the contract, we need to add two additional functions.

We need a function that lets us read the token value. Paste this into `main.nr`:

#include_code get_token /noir-projects/noir-contracts/contracts/token_bridge_contract/src/main.nr rust

## Compile code

Congratulations, you have written all the contracts we need for this tutorial! Now let's compile them.

Compile your Solidity contracts using hardhat. Run this in the `packages` directory:

```bash
cd l1-contracts
npx hardhat compile
```

And compile your Aztec.nr contracts like this:

```bash
cd aztec-contracts/token_bridge
aztec-nargo compile
```

You may get some unused variable warnings - you can ignore these.

And generate the TypeScript interface for the contract and add it to the test dir. Run this inside `aztec-contracts/token_bridge`:

```bash
aztec codegen target -o ../../src/test/fixtures
```

This will create a TS interface inside `fixtures` dir in our `src/test` folder!

In the next step we will write the TypeScript code to deploy our contracts and call on both L1 and L2 so we can see how everything works together.
