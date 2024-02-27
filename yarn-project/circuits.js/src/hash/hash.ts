import { AztecAddress } from '@aztec/foundation/aztec-address';
import { padArrayEnd } from '@aztec/foundation/collection';
import { pedersenHash, pedersenHashBuffer } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { numToUInt8, numToUInt16BE, numToUInt32BE } from '@aztec/foundation/serialize';

import { Buffer } from 'buffer';
import chunk from 'lodash.chunk';

import {
  ARGS_HASH_CHUNK_COUNT,
  ARGS_HASH_CHUNK_LENGTH,
  FUNCTION_TREE_HEIGHT,
  GeneratorIndex,
} from '../constants.gen.js';
import { MerkleTreeCalculator } from '../merkle/merkle_tree_calculator.js';
import type { FunctionData, SideEffect, SideEffectLinkedToNoteHash } from '../structs/index.js';
import { VerificationKey } from '../structs/verification_key.js';

/**
 * Computes a hash of a given verification key.
 * @param vkBuf - The verification key.
 * @returns The hash of the verification key.
 */
export function hashVK(vkBuf: Buffer) {
  const vk = VerificationKey.fromBuffer(vkBuf);
  const toHash = Buffer.concat([
    numToUInt8(vk.circuitType),
    numToUInt16BE(5), // fr::coset_generator(0)?
    numToUInt32BE(vk.circuitSize),
    numToUInt32BE(vk.numPublicInputs),
    ...Object.values(vk.commitments)
      .map(e => [e.y.toBuffer(), e.x.toBuffer()])
      .flat(),
    // Montgomery form of fr::one()? Not sure. But if so, why?
    Buffer.from('1418144d5b080fcac24cdb7649bdadf246a6cb2426e324bedb94fb05118f023a', 'hex'),
  ]);
  return pedersenHashBuffer(toHash);
  // barretenberg::evaluation_domain eval_domain = barretenberg::evaluation_domain(circuit_size);

  // std::vector<uint8_t> preimage_data;

  // preimage_data.push_back(static_cast<uint8_t>(proof_system::CircuitType(circuit_type)));

  // const uint256_t domain = eval_domain.domain; // montgomery form of circuit_size
  // const uint256_t generator = eval_domain.generator; //coset_generator(0)
  // const uint256_t public_inputs = num_public_inputs;

  // write(preimage_data, static_cast<uint16_t>(uint256_t(generator))); // maybe 1?
  // write(preimage_data, static_cast<uint32_t>(uint256_t(domain))); // try circuit_size
  // write(preimage_data, static_cast<uint32_t>(public_inputs));
  // for (const auto& [tag, selector] : commitments) {
  //     write(preimage_data, selector.y);
  //     write(preimage_data, selector.x);
  // }

  // write(preimage_data, eval_domain.root);  // fr::one()

  // return crypto::pedersen_hash::hash_buffer(preimage_data, hash_index);
}

let functionTreeRootCalculator: MerkleTreeCalculator | undefined;
/**
 * The "zero leaf" of the function tree is the hash of 5 zero fields.
 * TODO: Why can we not just use a zero field as the zero leaf? Complicates things perhaps unnecessarily?
 */
function getFunctionTreeRootCalculator() {
  if (!functionTreeRootCalculator) {
    const functionTreeZeroLeaf = pedersenHash(new Array(5).fill(Buffer.alloc(32))).toBuffer();
    functionTreeRootCalculator = new MerkleTreeCalculator(FUNCTION_TREE_HEIGHT, functionTreeZeroLeaf);
  }
  return functionTreeRootCalculator;
}

/**
 * Computes a function tree from function leaves.
 * @param fnLeaves - The function leaves to be included in the contract function tree.
 * @returns All nodes of the tree.
 */
export function computeFunctionTree(fnLeaves: Fr[]) {
  const leaves = fnLeaves.map(fr => fr.toBuffer());
  return getFunctionTreeRootCalculator()
    .computeTree(leaves)
    .nodes.map(b => Fr.fromBuffer(b));
}

/**
 * Computes a function tree root from function leaves.
 * @param fnLeaves - The function leaves to be included in the contract function tree.
 * @returns The function tree root.
 */
export function computeFunctionTreeRoot(fnLeaves: Fr[]) {
  const leaves = fnLeaves.map(fr => fr.toBuffer());
  return Fr.fromBuffer(getFunctionTreeRootCalculator().computeTreeRoot(leaves));
}

/**
 * Computes a constructor hash.
 * @param functionData - Constructor's function data.
 * @param argsHash - Constructor's arguments hashed.
 * @param constructorVKHash - Hash of the constructor's verification key.
 * @returns The constructor hash.
 */
export function hashConstructor(functionData: FunctionData, argsHash: Fr, constructorVKHash: Buffer): Fr {
  return pedersenHash(
    [functionData.hash().toBuffer(), argsHash.toBuffer(), constructorVKHash],
    GeneratorIndex.CONSTRUCTOR,
  );
}

/**
 * Computes a commitment nonce, which will be used to create a unique commitment.
 * @param nullifierZero - The first nullifier in the tx.
 * @param commitmentIndex - The index of the commitment.
 * @returns A commitment nonce.
 */
export function computeCommitmentNonce(nullifierZero: Fr, commitmentIndex: number): Fr {
  return pedersenHash([nullifierZero.toBuffer(), numToUInt32BE(commitmentIndex, 32)], GeneratorIndex.NOTE_HASH_NONCE);
}

/**
 * Computes a siloed commitment, given the contract address and the commitment itself.
 * A siloed commitment effectively namespaces a commitment to a specific contract.
 * @param contract - The contract address
 * @param innerNoteHash - The commitment to silo.
 * @returns A siloed commitment.
 */
export function siloNoteHash(contract: AztecAddress, innerNoteHash: Fr): Fr {
  return pedersenHash([contract.toBuffer(), innerNoteHash.toBuffer()], GeneratorIndex.SILOED_NOTE_HASH);
}

/**
 * Computes a unique commitment. It includes a nonce which contains data that guarantees the commitment will be unique.
 * @param nonce - The contract address.
 * @param siloedCommitment - An siloed commitment.
 * @returns A unique commitment.
 */
export function computeUniqueCommitment(nonce: Fr, siloedCommitment: Fr): Fr {
  return pedersenHash([nonce.toBuffer(), siloedCommitment.toBuffer()], GeneratorIndex.UNIQUE_NOTE_HASH);
}

/**
 * Computes a siloed nullifier, given the contract address and the inner nullifier.
 * A siloed nullifier effectively namespaces a nullifier to a specific contract.
 * @param contract - The contract address.
 * @param innerNullifier - The nullifier to silo.
 * @returns A siloed nullifier.
 */
export function siloNullifier(contract: AztecAddress, innerNullifier: Fr): Fr {
  return pedersenHash([contract.toBuffer(), innerNullifier.toBuffer()], GeneratorIndex.OUTER_NULLIFIER);
}

/**
 * Computes a public data tree value ready for insertion.
 * @param value - Raw public data tree value to hash into a tree-insertion-ready value.
 * @returns Value hash into a tree-insertion-ready value.

 */
export function computePublicDataTreeValue(value: Fr): Fr {
  return value;
}

/**
 * Computes a public data tree index from contract address and storage slot.
 * @param contractAddress - Contract where insertion is occurring.
 * @param storageSlot - Storage slot where insertion is occurring.
 * @returns Public data tree index computed from contract address and storage slot.

 */
export function computePublicDataTreeLeafSlot(contractAddress: AztecAddress, storageSlot: Fr): Fr {
  return pedersenHash([contractAddress.toBuffer(), storageSlot.toBuffer()], GeneratorIndex.PUBLIC_LEAF_INDEX);
}

/**
 * Computes the hash of a list of arguments.
 * @param args - Arguments to hash.
 * @returns Pedersen hash of the arguments.
 */
export function computeVarArgsHash(args: Fr[]) {
  if (args.length === 0) {
    return Fr.ZERO;
  }
  const maxLen = ARGS_HASH_CHUNK_LENGTH * ARGS_HASH_CHUNK_COUNT;
  if (args.length > maxLen) {
    // TODO(@spalladino): This should throw instead of warning. And we should implement
    // the same check on the Noir side, which is currently missing.
    args = args.slice(0, maxLen);
    createDebugLogger('aztec:circuits:abis').warn(`Hashing ${args.length} args exceeds max of ${maxLen}`);
  }

  let chunksHashes = chunk(args, ARGS_HASH_CHUNK_LENGTH).map(c => {
    if (c.length < ARGS_HASH_CHUNK_LENGTH) {
      c = padArrayEnd(c, Fr.ZERO, ARGS_HASH_CHUNK_LENGTH);
    }
    return pedersenHash(
      c.map(a => a.toBuffer()),
      GeneratorIndex.FUNCTION_ARGS,
    );
  });

  if (chunksHashes.length < ARGS_HASH_CHUNK_COUNT) {
    chunksHashes = padArrayEnd(chunksHashes, Fr.ZERO, ARGS_HASH_CHUNK_COUNT);
  }

  return pedersenHash(
    chunksHashes.map(a => a.toBuffer()),
    GeneratorIndex.FUNCTION_ARGS,
  );
}

export function computeCommitmentsHash(input: SideEffect) {
  return pedersenHash([input.value.toBuffer(), input.counter.toBuffer()], GeneratorIndex.SIDE_EFFECT);
}

export function computeNullifierHash(input: SideEffectLinkedToNoteHash) {
  return pedersenHash(
    [input.value.toBuffer(), input.noteHash.toBuffer(), input.counter.toBuffer()],
    GeneratorIndex.SIDE_EFFECT,
  );
}

/**
 * Given a secret, it computes its pedersen hash - used to send l1 to l2 messages
 * @param secret - the secret to hash - secret could be generated however you want e.g. `Fr.random()`
 * @returns the hash
 */
export function computeMessageSecretHash(secretMessage: Fr) {
  return pedersenHash([secretMessage.toBuffer()], GeneratorIndex.L1_TO_L2_MESSAGE_SECRET);
}
