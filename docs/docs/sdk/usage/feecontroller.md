---
title: Fee Controller
---

Pay transaction fees for other accounts.

High level overview of creating paying transaction fees for another user:

### Context

Alice is generating transaction proofs that will be sent to the [sequencer](../../glossary.md#sequencer) by Bob. Bob will pay the transaction fees to process Alice's transaction proofs.

### Process

1. Alice sets up the transaction creation [controller](../overview.md#controllers).
   1. `registerController = aztecSdk.createRegisterController(...)`
2. Alice generates the transaction proof using the controller.
   1. `registerController.createProof()`
3. Alice exports the proof data and sends it to Bob.
   1. `proofTxs = registerController.exportProofTxs()`
4. Bob queries the Aztec client for the current transaction fee rate.
   1. `fee = aztecSdk.getProofTxsFees(assetId, proofTxs)`
5. Bob creates a [FeeController](../types/sdk/FeeController.md), generates the proof and sends the transactions to the [sequencer](./../../glossary.md#sequencer).
   1. `feeController = aztecSdk.createFeeController(alicesUserId, bobsSigner, proofTxs, fee)`
   2. `feeController.createProof()`
   3. `feeController.send()`
