## Events
Events in Aztec work similarly to Ethereum events in the sense that they are a way for contracts to communicate with the outside world.
They are emitted by contracts and stored inside each instance of an AztecNode.
> Aztec events are currently represented as raw data and are not ABI encoded.
> ABI encoded events are a feature that will be added in the future.

Unlike on Ethereum, there are 2 types of events supported by Aztec: encrypted and unencrypted.

### Encrypted Events
Encrypted events can only be emitted by private functions and are encrypted using a public key of a recipient.
For this reason it is necessary to register a recipient in the Aztec RPC Server before encrypting the events for them.
Recipients can be registered using the Aztec CLI or Aztec.js:

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

<Tabs groupId="events">
<TabItem value="cli" label="Aztec CLI">

```bash
aztec-cli register-recipient --address 0x147392a39e593189902458f4303bc6e0a39128c5a1c1612f76527a162d36d529 --public-key 0x26e193aef4f83c70651485b5526c6d01a36d763223ab24efd1f9ff91b394ac0c20ad99d0ef669dc0dde8d5f5996c63105de8e15c2c87d8260b9e6f02f72af622 --partial-address 0x200e9a6c2d2e8352012e51c6637659713d336405c29386c7c4ac56779ab54fa7
```

</TabItem>
<TabItem value="js" label="Aztec.js">

```ts
const aztecAddress = AztecAddress.fromString("0x147392a39e593189902458f4303bc6e0a39128c5a1c1612f76527a162d36d529");
const publicKey = Point.fromString("0x26e193aef4f83c70651485b5526c6d01a36d763223ab24efd1f9ff91b394ac0c20ad99d0ef669dc0dde8d5f5996c63105de8e15c2c87d8260b9e6f02f72af622");
const partialAddress = Fr.fromString("0x200e9a6c2d2e8352012e51c6637659713d336405c29386c7c4ac56779ab54fa7");

const completeAddress = CompleteAddress.create(aztecAddress, publicKey, partialKey); 
await aztecRpc.registerRecipient(completeAddress);
```

</TabItem>
</Tabs>

> **NOTE**: If a note recipient is one of the accounts inside the Aztec RPC Server, we don't need to register it as a recipient because we already have the public key available.

> At this point the Sandbox only enables the emitting of encrypted note preimages through encrypted events.
> In the future we will allow emitting arbitrary information.
> (If you currently emit arbitrary information, Aztec RPC Server will fail to decrypt, process and store this data, so it will not be queryable).

To emit encrypted logs first import the `emit_encrypted_log` utility function inside your contract:

#include_code encrypted_import /yarn-project/noir-libs/value-note/src/utils.nr rust

Then you can call the function:

#include_code encrypted /yarn-project/noir-libs/value-note/src/utils.nr rust


### Unencrypted Events
Unencrypted events are events which can be read by anyone.
They can be emitted by both public and private functions.

:::danger

Emitting unencrypted events from private function is a significant privacy leak and it should be considered by the developer whether it is acceptable.

:::

Once emitted, unencrypted events are stored in AztecNode and can be queried by anyone:
<Tabs groupId="events">
<TabItem value="cli" label="Aztec CLI">

```bash
aztec-cli get-logs --from 5 --limit 1
```

</TabItem>
<TabItem value="js" label="Aztec.js">

#include_code logs /yarn-project/end-to-end/src/e2e_public_token_contract.test.ts typescript

</TabItem>
</Tabs>

To emit unencrypted logs first import the `emit_unencrypted_log` utility function inside your contract:

#include_code unencrypted_import /yarn-project/noir-contracts/src/contracts/private_token_contract/src/main.nr rust

Then you can call the function:

#include_code unencrypted /yarn-project/noir-contracts/src/contracts/private_token_contract/src/main.nr rust

### Costs

All event data is pushed to Ethereum as calldata by the sequencer and for this reason the cost of emitting an event is non-trivial.

> Note: the cost of submitting calldata to Ethereum is currently 4 gas per byte. Currently, in the Sandbox, an encypted note has a fixed overhead of 4 field elements (to broadcast an ephemeral public key, a contract address, and a storage slot); plus a variable number of field elements depending on the type of note being emitted.
> A `ValueNote`, for example, currently uses 3 fields elements (plus the fixed overhead of 4). That's roughly `7 * 32 = 224` bytes of information, costing roughly 896 gas.

> There are plans to compress encrypted note data further.
> There are plans to adopt EIP-4844 blobs to reduce the cost of data submission further.

## Processing events
Both the encrypted and unencrypted events are stored in AztecNode.
Unencrypted logs can be queried by anyone as we described above in the [Unencrypted Events](#unencrypted-events) section.

Encrypted logs need to first be decrypted:

### Decrypting
One function of Aztec RPC Server is constantly loading encrypted logs from AztecNode and trying to decrypt them.
When new encrypted logs are obtained, the Aztec RPC Server will try to decrypt them using the private encryption key of all the accounts registered inside Aztec RPC Server.
If the decryption is successful, the Aztec RPC Server will store the decrypted note inside a database.
If the decryption fails, the specific log will be discarded.

For the Aztec RPC Server to successfully process the decrypted note we need to compute the note's 'note hash' and 'nullifier'.
Aztec.nr enables smart contract developers to design custom notes, meaning developers can also customise how a note's note hash and nullifier should be computed. Because of this customisability, and because there will be a potentially-unlimited number of smart contracts deployed to Aztec, an Aztec RPR Server needs to be 'taught' how to compute the custom note hashes and nullifiers for a particular contract. Therefore, developers will need to implement a `compute_note_hash_and_nullifier` function inside their contracts.
Every time a new note is successfully decrypted, the Aztec RPC Server will expect the existence of a `compute_note_hash_and_nullifier` function, which must teach it how to correctly process the new note.

This is an example implementation inside the `PrivateTokenContract`:

#include_code compute_note_hash_and_nullifier /yarn-project/noir-contracts/src/contracts/private_token_contract/src/main.nr rust