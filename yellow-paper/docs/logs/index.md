---
title: Logs
---

Logs on Aztec are similar to logs on Ethereum, serving the purpose of enabling smart contracts to convey arbitrary data to external entities. This communication occurs through three distinct types of logs:

- [Unencrypted log](#unencrypted-log).
- [Encrypted log](#encrypted-log).
- [Encrypted note preimage](#encrypted-note-preimage).

These logs are generated in the course of contract function executions and play a pivotal role in aiding users to comprehend and leverage the received block data while facilitating interaction with the network.

## Requirements

1. **Availability**: The logs get published.

   A rollup proof won't be accepted by the rollup contract if the log preimages are not available. Similarly, a sequencer cannot accept a transaction unless log preimages accompany the transaction data.

2. **Immutability**: A log cannot be modified once emitted.

   The protocol ensures that once a proof is generated at any stage (for a function, transaction, or block), the emitted logs are finalized. In other words, only the original log preimages can generate the committed hashes in the proof. Any party can use this attribute to verify that the provided log preimages are not tempered.

3. **Integrity**: A contract cannot impersonate another contract.

   Every log is emitted by a specific contract, necessitating the identification of the contract address. This information is crucial for subsequent interactions with the contract or for interpreting the received data. The protocol ensures that the source contract's address for a log can be verified, while also preventing the forging of the addresses.

## Log Hash

### Hash Function

The protocol uses **SHA256** as the hash function for logs, and then reduces the 256-bit result to 253 bits for representation as a field element.

Throughout this page, `hash(value)` is an abbreviated form of: `truncate_to_field(SHA256(value))`

### Hashing

Regardless of the log type, the hash is derived from an array of fields, calculated as:

`hash(log_preimage[0], log_preimage[1], ..., log_preimage[N - 1])`

Here, _log_preimage_ is an array of field elements of length _N_, representing the data to be broadcasted.

#### Emitting Logs from Function Circuits

A function can emit an arbitrary number of logs, provided they don't exceed the specified [limit]. The function circuits must compute a hash for each log, and push all the hashes into the public inputs for further processing by the protocol circuits.

#### Aggregation in Protocol Circuits

To minimize the on-chain verification data size, protocol circuits aggregate log hashes. The end result is a single hash within the root rollup proof, encompassing all logs of the same type.

Each protocol circuit outputs two values for each log type:

- _`accumulated_logs_hash`_: A hash representing all logs.
- _`accumulated_logs_length`_: The total length of all log preimages.

In cases where two proofs are combined to form a single proof, the _accumulated_logs_hash_ and _accumulated_logs_length_ from the two child proofs must be merged into one accumulated value:

- _`accumulated_logs_hash = hash(proof_0.accumulated_logs_hash, proof_1.accumulated_logs_hash)`_
  - If either hash is zero, the new hash will be _`proof_0.accumulated_logs_hash | proof_1.accumulated_logs_hash`_.
- _`accumulated_logs_length = proof_0.accumulated_logs_length + proof_1.accumulated_logs_length`_

For private and public kernel circuits, beyond aggregating logs from a function call, they ensure that the contract's address emitting the logs is linked to the _logs_hash_. For more details, refer to the "Hashing" sections in [Unencrypted Log](#hashing-1), [Encrypted Log](#hashing-2), and [Encrypted Note Preimage](#hashing-3).

### Encoding

1. The encoded logs data of a transaction is a flatten array of all logs data within the transaction:

   _`tx_logs_data = [number_of_logs, ...log_data_0, ...log_data_1, ...]`_

   The format of _log_data_ varies based on the log type. For specifics, see the "Encoding" sections in [Unencrypted Log](#encoding-1), [Encrypted Log](#encoding-2), and [Encrypted Note Preimage](#encoding-3).

2. The encoded logs data of a block is a flatten array of a collection of the above _tx_logs_data_, with hints facilitating hashing replay in a binary tree structure:

   _`block_logs_data = [number_of_branches, number_of_transactions, ...tx_logs_data_0, ...tx_logs_data_1, ...]`_

   - _number_of_transactions_ is the number of leaves in the left-most branch, restricted to either _1_ or _2_.
   - _number_of_branches_ is the depth of the parent node of the left-most leaf.

Here is a step-by-step example to construct the _block_logs_data_:

1. A rollup, _R01_, merges two transactions: _tx0_ containing _tx_logs_data_0_, and _tx1_ containing _tx_logs_data_1_:

   ```mermaid
   flowchart BT
       tx0((tx0))
       tx1((tx1))
       R01((R01))
       tx0 --- R01
       tx1 --- R01
   ```

   _block_logs_data_: _`[0, 2, ...tx_logs_data_0, ...tx_logs_data_1]`_

   Where _0_ is the depth of the node _R01_, and _2_ is the number of aggregated _tx_logs_data_ of _R01_.

2. Another rollup, _R23_, merges two transactions: _tx3_ containing _tx_logs_data_3_, and _tx2_ without any logs:

   ```mermaid
   flowchart BT
       tx2((tx2))
       tx3((tx3))
       R23((R23))
       tx2 -. no logs .- R23
       tx3 --- R23
   ```

   _block_logs_data_: _`[0, 1, ...tx_logs_data_3]`_

   Here, the number of aggregated _tx_logs_data_ is _1_.

3. A rollup, _RA_, merges the two rollups _R01_ and _R23_:

   ```mermaid
   flowchart BT
      tx0((tx0))
      tx1((tx1))
      R01((R01))
      tx0 --- R01
      tx1 --- R01
      tx2((tx2))
      tx3((tx3))
      R23((R23))
      tx2 -.- R23
      tx3 --- R23
      RA((RA))
      R01 --- RA
      R23 --- RA
   ```

   _block_logs_data_: _`[1, 2, ...tx_logs_data_0, ...tx_logs_data_1, 0, 1, ...tx_logs_data_3]`_

   The result is the _block_logs_data_ of _R01_ concatenated with the _block_logs_data_ of _R23_, with the _number_of_branches_ of _R01_ incremented by _1_. The updated value of _number_of_branches_ (_0 + 1_) is also the depth of the node _R01_.

4. A rollup, _RB_, merges the above rollup _RA_ and another rollup _R45_:

   ```mermaid
   flowchart BT
     tx0((tx0))
      tx1((tx1))
      R01((R01))
      tx0 --- R01
      tx1 --- R01
      tx2((tx2))
      tx3((tx3))
      R23((R23))
      tx2 -.- R23
      tx3 --- R23
      RA((RA))
      R01 --- RA
      R23 --- RA
      tx4((tx4))
      tx5((tx5))
      R45((R45))
      tx4 --- R45
      tx5 --- R45
      RB((RB))
      RA --- RB
      R45 --- RB
   ```

   _block_logs_data_: _`[2, 2, ...tx_logs_data_0, ...tx_logs_data_1, 0, 1, ...tx_logs_data_3, 0, 2, ...tx_logs_data_4, ...tx_logs_data_5]`_

   The result is the concatenation of the _block_logs_data_ from both rollups, with the _number_of_branches_ of the left-side rollup, _RA_, incremented by _1_.

### Verification

Upon receiving a proof and its encoded logs data, the entity can ensure the correctness of the provided _block_logs_data_ by verifying that the _accumulated_logs_hash_ in the proof can be derived from it:

```js
const accumulated_logs_hash = compute_accumulated_logs_hash(block_logs_data);
assert(accumulated_logs_hash == proof.accumulated_logs_hash);
assert(block_logs_data.accumulated_logs_length == proof.accumulated_logs_length);

function compute_accumulated_logs_hash(logs_data) {
  const number_of_branches = logs_data.read_u32();

  const number_of_transactions = logs_data.read_u32();
  let res = hash_tx_logs_data(logs_data);
  if number_of_transactions == 2 {
    res = hash(res, hash_tx_logs_data(logs_data));
  }

  for (let i = 0; i < number_of_branches; ++i) {
    const res_right = compute_accumulated_logs_hash(logs_data);
    res = hash(res, res_right);
  }

  return res;
}

function hash_tx_logs_data(logs_data) {
  const number_of_logs = logs_data.read_u32();
  let res = hash_log_data(logs_data);
  for (let i = 1; i < number_of_logs; ++i) {
    const log_hash = hash_log_data(logs_data);
    res = hash(res, log_hash);
  }
  return res;
}
```

The _accumulated_logs_length_ in _block_logs_data_ is computed during the processing of each _logs_data_ within _hash_log_data()_. The implementation of _hash_log_data_ varies depending on the type of the logs being processed. Refer to the "Verification" sections in [Unencrypted Log](#verification-1), [Encrypted Log](#verification-2), and [Encrypted Note Preimage](#verification-3) for details.

## Unencrypted Log

Unencrypted logs are used to communicate public information out of smart contracts. They can be emitted from both public and private functions.

:::info
Emitting unencrypted logs from private functions may pose a privacy leak. However, in-protocol restrictions are intentionally omitted to allow for potentially valuable use cases, such as custom encryption schemes utilizing Fully Homomorphic Encryption (FHE), and similar scenarios.
:::

### Hashing

Following the iterations for all private or public calls, the tail kernel circuits hash each log hash with the contract contract before computing the _accumulated_logs_hash_.

1. Hash the _contract_address_ to each _log_hash_:

   - _`log_hash_a = hash(log_hash_a, contract_address_a)`_
   - Repeat the process for all _log_hashes_ in the transaction.

2. Accumulate all the hashes and output the final hash to the public inputs:

   - _`accumulated_logs_hash = hash(logs_hash_a, logs_hash_b)`_
     - For tail public kernel circuit, it begins with _`accumulated_logs_hash = hash(accumulated_logs_hash, logs_hash_a)`_ if the _accumulated_logs_hash_ outputted from the tail private kernel circuit is not empty.
   - _`accumulated_logs_hash = hash(accumulated_logs_hash, logs_hash_c)`_
   - Repeat the process until all _logs_hashes_ are collectively hashed.

### Encoding

The following represents the encoded data for an unencrypted log:

_`log_data = [log_preimage_length, contract_address, ...log_preimage]`_

### Verification

```js
function hash_log_data(logs_data) {
  const log_preimage_length = logs_data.read_u32();
  logs_data.accumulated_logs_length += log_preimage_length;
  const contract_address = logs_data.read_field();
  const log_preimage = logs_data.read_fields(log_preimage_length);
  const log_hash = hash(...log_preimage);
  return hash(log_hash, contract_address);
}
```

## Encrypted Log

Encrypted logs contain information encrypted using the recipient's key. They can only be emitted from private functions. This restriction is due to the necessity of obtaining a secret for log encryption, which is challenging to manage privately in a public domain.

### Hashing

Private kernel circuits ensure the association of the contract address with each encrypted _log_hash_. However, unlike unencrypted logs, submitting encrypted log preimages with their contract address poses a significant privacy risk. Therefore, instead of using the _contract_address_, a _contract_address_tag_ is generated for each encrypted _log_hash_.

The _contract_address_tag_ is a hash of the _contract_address_ and a random value _randomness_, computed as:

_`contract_address_tag = hash(contract_address, randomness)`_.

Here, _randomness_ is generated in the private function circuit and supplied to the private kernel circuit. The value must be included in the preimage for encrypted log generation. The private function circuit is responsible for ensuring that the _randomness_ differs for every encrypted log to avoid potential information linkage based on identical _contract_address_tag_.

After successfully decrypting an encrypted log, one can use the _randomness_ in the log preimage, hash it with the _contract_address_, and verify it against the _contract_address_tag_ to ascertain that the log originated from the specified contract.

1. Hash the _contract_address_tag_ to each _log_hash_:

   - _`contract_address_tag_a = hash(contract_address_a, randomness)`_
   - _`log_hash_a = hash(log_hash_a, contract_address_tag_a)`_
   - Repeat the process for all _log_hashes_ in the transaction.

2. Accumulate all the hashes and outputs the final hash to the public inputs:

   - _`accumulated_logs_hash = hash(log_hash_a, log_hash_b)`_
   - _`accumulated_logs_hash = hash(accumulated_logs_hash, log_hash_c)`_
   - Repeat the process until all _logs_hashes_ are collectively hashed.

### Encoding

The following represents the encoded data for an unencrypted log:

_`log_data = [log_preimage_length, contract_address_tag, ...log_preimage]`_

### Verification

```js
function hash_log_data(logs_data) {
  const log_preimage_length = logs_data.read_u32();
  logs_data.accumulated_logs_length += log_preimage_length;
  const contract_address_tag = logs_data.read_field();
  const log_preimage = logs_data.read_fields(log_preimage_length);
  const log_hash = hash(...log_preimage);
  return hash(log_hash, contract_address_tag);
}
```

## Encrypted Note Preimage

Similar to [encrypted logs](#encrypted-log), encrypted note preimages are data that only entities possessing the keys can decrypt to view the plaintext. Unlike encrypted logs, each encrypted note preimage can be linked to a note, whose note hash can be found in the block data.

> Note that a note can be "shared" to one or more recipients by emitting one or more encrypted note preimages. However, this is not mandatory, and there may be no encrypted preimages emitted for a note if the information can be obtain through alternative means.

### Hashing

As each encrypted note preimage can be associated with a note in the same transaction, enforcing a _contract_address_tag_ is unnecessary. Instead, by calculating the _note_hash_ using the decrypted note preimage, hashed with the _contract_address_, and verify it against the block data, the recipient can confirm that the note was emitted from the specified contract.

The kernel circuit simply accumulates all the hashes:

- _`accumulated_logs_hash = hash(log_hash_a, log_hash_b)`_
- _`accumulated_logs_hash = hash(accumulated_logs_hash, log_hash_c)`_
- Repeat the process until all _logs_hashes_ are collectively hashed.

### Encoding

The following represents the encoded data for an unencrypted note preimage:

_`log_data = [log_preimage_length, ...log_preimage]`_

### Verification

```js
function hash_log_data(logs_data) {
  const log_preimage_length = logs_data.read_u32();
  logs_data.accumulated_logs_length += log_preimage_length;
  const log_preimage = logs_data.read_fields(log_preimage_length);
  return hash(...log_preimage);
}
```

## Log Encryption

Refer to [Private Message Delivery](../private-message-delivery/) for detailed information on generating encrypted data.
