---
title: Deposit (Shield)
---

Deposit assets from Ethereum to Aztec.

Review the [general page on deposits](../../developers/deposit) for a higher level review of how deposits work on Aztec.

The SDK comes with a `DepositController` that makes it easy to create and track deposit transactions from Ethereum to Aztec.

Deposits require sending Ethereum transactions from a user's account to the Aztec deposit contract. Any Etheruem account can make a deposit to any Aztec account.

An example to help clarify. Say Alice is an entity on L1 that wants to make a deposit of 10 DAI into Bob's account on Aztec. She knows that Bob owns the bob.eth ENS (Ethereum Name Service) name, so she can make a deposit like (ens inserted instead of address for clarity):

```js
aztecRollupContract.depositPendingFunds(1, 10e18, bob.eth, bytes32(0));
```

After this transaction is executed, there is now 10 DAI in Bob's pending balance on Aztec. Bob then creates a deposit proof (only on L2, no L1 transaction), where he uses the 10 DAI. Bob now has the 10 DAI in an Aztec account and he never sent a L1 tx himself. If there have been multiple deposits, he can make the deposit proof of the sum of them if he wants to, e.g., for 3 deposits of 10 DAI he could make a deposit proof spending all 30 DAI.

The SDK simplifies making deposits to the Aztec rollup contract as well as generating the proofs for claiming pending deposits.

You can find the interface for the `DepositController` class [here](../types/sdk/DefiController).

### Controller Setup

```ts
AztecSdk.createDepositController(
    depositor: EthAddress,          
    value: AssetValue,               
    fee: AssetValue,                  
    recipient: GrumpkinAddress,  
    recipientSpendingKeyRequired?: boolean,
    provider?: EthereumProvider)
        : Promise<DepositController>
```

### Controller Inputs

| Arguments | Type | Description |
| --------- | ---- | ----------- |
| depositor | [EthAddress](../types/barretenberg/EthAddress) | Ethereum account making the deposit. |
| value | [AssetValue](../types/barretenberg/AssetValue) | Type and amount of deposit. |
| fee | [AssetValue](../types/barretenberg/AssetValue) | Type and amount for the Aztec transaction fee. |
| recipient | [GrupmkinAddress](../types/barretenberg/GrumpkinAddress) | The account public key of the Aztec account. |
| recipientSpendingKeyRequired? | boolean | Optional flag that specifies whether the recipient account should already be registered. |
| provider? | [EthereumProvider](../types/barretenberg/EthereumProvider) | Optional Ethereum Provider. When unspecified it defaults to the provider used in setup (`createAztecSdk`). |

#### Returns

| Return Type | Description |
| --------- | ----------- |
| [DepositController](../types/sdk/DepositController) | A user instance with apis bound to the user's account id. |

### Executing a Deposit

The complete deposit flow for Ether using the `DepositController` looks like this: 

```ts
const tokenAssetId = sdk.getAssetIdBySymbol('ETH');
const tokenDepositFee = (await sdk.getDepositFees(tokenAssetId))[
    settlementTime
];
const tokenAssetValue: AssetValue = {
    assetId: tokenAssetId,
    value: tokenQuantity,
};
const tokenDepositController = sdk.createDepositController(
    depositor,
    tokenAssetValue,
    tokenDepositFee,
    recipient,
    true,
);
await tokenDepositController.createProof();
await tokenDepositController.sign();
// check if there are pending deposits
if (
    (await tokenDepositController.getPendingFunds()) < tokenAssetValue.value
) {
    if (asset === "dai") {
        if (
            (await tokenDepositController.getPublicAllowance()) <
            tokenAssetValue.value
        ) {
            await tokenDepositController.approve();
            await tokenDepositController.awaitApprove();
        }
    }
    await tokenDepositController.depositFundsToContract();
    await tokenDepositController.awaitDepositFundsToContract();
}
let txId = await tokenDepositController.send();
```

Not all ERC-20s (specifically DAI) have correctly implemented the permit spec. So in the case of DAI you call `depositFundsToContractWithNonStandardPermit` instead of `depositFundsToContract`. Note that the DAI contract has hardcoded chain ids for Ethereum networks ([source](https://github.com/makerdao/developerguides/blob/master/dai/how-to-use-permit-function/how-to-use-permit-function.md#permit-in-the-dai-contract)), so this method won't work on the Aztec testnet.

#### Required Approvals

When depositing an ERC-20 token like DAI, you will need to approve Aztec as an authorized spender before depositing. The `DepositController` includes a method for this, `DepositController.approve()` which will request approval for the amount required for the deposit.

### Advanced Usage

#### Depositing from a Smart Contract

A smart contract can deposit assets into Aztec and specify an externally owned account as the `owner` which enables that Ethereum account to sign a message and claim the deposit on Aztec. But this deposit method reveals some data about the recipient in the Ethereum L1 transaction and may not have the privacy guarantees you are looking for.

To hide the recipient when the depositing account is a smart contract, the `owner` input can be set to the depositing smart contract address when calling the `depositPendingFunds` on the [rollup processor contract](https://github.com/AztecProtocol/aztec-connect/blob/b0a71b01c5f1aa3b4b9a61d417aa4c479e01ef47/blockchain/contracts/interfaces/IRollupProcessor.sol#L33) and include a `proofHash` input. This input specifies which transaction id on Aztec is authorized to claim the pending deposit, so it hides the intended recipient. This transaction id (`proofHash`) must be computed beforehand and passed to `depositPendingFunds`. When the `depositPendingFunds` transaction has settled on Ethereum, the corresponding proof must be generated and sent to the Aztec sequencer to make the deposit.

This deposit method using the `proofHash` is also useful when a smart contract (e.g. multisig) is depositing assets to an Aztec native multisig account and a single Ethereum EOA cannot be trusted. This is necessary in these cases because there is no way to create a signature to generate a proof that corresponds to a smart contract address on Ethereum.

To get the transaction id (`proofHash`), create the `tokenDepositController`, as shown above, with the desired inputs. The `depositor` is the address of the smart contract doing the deposit.

Once you have the `tokenDepositController`:

```js
await tokenDepositController.createProof();
const proofHash = await tokenDepositController.getProofHash();
```

You do not need to call `sign()` on the DepositController, like in the previous example.

Once the `depositPendingFunds` transaction settles on Ethereum, send the funds to the Aztec account designated in the deposit proof by sending the deposit proof to the Aztec sequencer with `await tokenDepositController.send()`. The proof could be saved from before or regenerated from the same inputs.
