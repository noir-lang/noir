# Keys!


THIS IS OUT OF DATE

## Types of Keys

In order to read data from and send data to the Aztec Network, a user will need to have the following keys:

- **Ethereum key**: Used for signing the `TxRequest`.
- **Decryption key**: Used for decrypting data.
- **Nullifier key**: Used for creating nullifier.

> Q: not sure about dictating the derivation of a nullifier key in the way shown in this diagram.

![](https://hackmd.io/_uploads/BJUkpupz3.png)


## Ethereum Key

A user will use an Ethereum key to sign the `TxRequest`. And the `callContext.msgSender` should be the public key of that Ethereum key. Kernel circuit checks that `TxRequest.signer == callContext.msgSender`. This ensures the `msgSender` is what it claims to be.

A contract can then refer to `msgSender` and perform crucial checks. For example:

```rust
fn my_function() {
    require(whitelist[call_context.msg_sender] == true);
}
```

An Ethereum key never leaves the Ethereum wallet. A dapp and the AztecRPCServer only see the signature and the public key.


## Decryption Key

Each account has one decryption key. It will try to decrypt all encrypted notes using this decryption key. If it successfully decrypts a piece of data, it can see the contract address, storage slot, and the decrypted information.

When sending a note to an account, the sender will need to know the public key of the recipient's decryption key in order to encrypt the data for that account.

Decryption keys are derived and managed by the key store. The AztecRPCServer requests the key from the injected key store when an account is connected. The key should never leave the AztecRPCServer (the account state, to be more specific), and it should never be exposed to the contracts.

> Open Q: could we use the Ethereum secret key as the decryption key (which can then be used to derive a shared secret, which can then be used as an AES encryption key)?

> Open Q: should we use AES encryption?


## Nullifier Key

Most privacy protocols use a secret key when generating a nullifier for a note. In applications such as zk.money or zcash, this ensures that only relevant parties who know the secret key can spend a note. In protocols like zexe, such a nullifier is used more generally to nullify a note (what they call 'records'). And so, we want contracts on Aztec to have the option of using a secret key to generate a nullifer. 

In Aztec, a contract can get a secret key (for some public key) from the data oracle. It can then use the given key, or modify it, to create a nullifier. For example:

```rust
global Generator = [0x1234, 0x5678];

fn compute_nullifier(self, note_hash: Field) -> Field {
    let secret_key = oracle.get_secret_key(self.owner);
    let nullifier_key = secret_key * Generator;
    pedersen_hash([note_hash, nullifier_key])
}
```

Because a contract can see the secret key, it's recommended to use a different secret key for each contract. If a malfunctioning contract exposes that secret key to the outputted public inputs, the user doesn't have to abandon their entire account just because this key is compromised.

A key store can create one master secret key for an account, and then derives a new key using the contract address when requested by a contract.

Note that not all nullifiers need to use a secret key. A contract can create a constant nullifier, for example, to guarantee that an action has been executed and can't be done again. Some nullifier schemes are now designed to not need a 'hot' secret key within the circuit (e.g. [plume](https://eprint.iacr.org/2022/1255.pdf)).


## How to link the Ethereum key and other keys?

A contract can define a state variable that maps an Ethereum key to a decryption public key:

```rust
global key_mappings: Mapping<Field, PublicSingleton<Note<Field>>;

fn register(public_key: Field) {
    let eth_address = derive_eth_address(call_data.msg_sender);
    key_mappings[eth_address] = public_key;
}
```

A function can then look up the public key using an Ethereum address:

```rust
fn transfer(amount: Field, recipient: EthAddress) {
    let note_recipient_public_key = key_mappings[recipient];
    // Use that public key when constructing the new note.
    
    let note_sender_public_key = key_mappings[call_context.msg_sender];
    let secret_key = oracle.get_secret_key(note_sender_public_key);
    // Use that secret key to compute the nullifier.
}
```

What `key_mappings[recipient]` does is calling the oracle to get the public key for the recipient, from the public/private data tree (depending on whether the mapping is public or private). A membership check will happen to ensure an account note (public_key, recipient) has been created via the above `register` function.