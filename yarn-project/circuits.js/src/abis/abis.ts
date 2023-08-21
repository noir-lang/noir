import { padArrayEnd } from '@aztec/foundation/collection';
import { IWasmModule } from '@aztec/foundation/wasm';

import { Buffer } from 'buffer';
import chunk from 'lodash.chunk';

import {
  abisComputeBlockHash,
  abisComputeBlockHashWithGlobals,
  abisComputeCommitmentNonce,
  abisComputeGlobalsHash,
  abisComputePublicDataTreeIndex,
  abisComputePublicDataTreeValue,
  abisComputeUniqueCommitment,
  abisSiloCommitment,
  abisSiloNullifier,
} from '../cbind/circuits.gen.js';
import {
  AztecAddress,
  FUNCTION_SELECTOR_NUM_BYTES,
  Fr,
  FunctionData,
  FunctionLeafPreimage,
  GlobalVariables,
  NewContractData,
  PrivateCallStackItem,
  PublicCallStackItem,
  PublicKey,
  TxRequest,
  Vector,
} from '../index.js';
import { serializeBufferArrayToVector } from '../utils/serialize.js';

/**
 * Synchronously calls a wasm function.
 * @param wasm - The wasm wrapper.
 * @param fnName - The name of the function to call.
 * @param input - The input buffer or object serializable to a buffer.
 * @param expectedOutputLength - The expected length of the output buffer.
 * @returns The output buffer.
 */
export function wasmSyncCall(
  wasm: IWasmModule,
  fnName: string,
  input:
    | Buffer
    | {
        /**
         * Signature of the target serialization function.
         */
        toBuffer: () => Buffer;
      },
  expectedOutputLength: number,
): Buffer {
  const inputData: Buffer = input instanceof Buffer ? input : input.toBuffer();
  const outputBuf = wasm.call('bbmalloc', expectedOutputLength);
  const inputBuf = wasm.call('bbmalloc', inputData.length);
  wasm.writeMemory(inputBuf, inputData);
  wasm.call(fnName, inputBuf, outputBuf);
  const buf = Buffer.from(wasm.getMemorySlice(outputBuf, outputBuf + expectedOutputLength));
  wasm.call('bbfree', outputBuf);
  wasm.call('bbfree', inputBuf);
  return buf;
}

/**
 * Writes input buffers to wasm memory, calls a wasm function, and returns the output buffer.
 * @param wasm - A module providing low-level wasm access.
 * @param fnName - The name of the function to call.
 * @param inputBuffers - Buffers to write to wasm memory.
 * @param expectedOutputLength - The expected length of the output buffer.
 * @returns The output buffer.
 */
export function inputBuffersToOutputBuffer(
  wasm: IWasmModule,
  fnName: string,
  inputBuffers: Buffer[],
  expectedOutputLength: number,
) {
  const offsets: number[] = [];
  const totalLength = inputBuffers.reduce((total, cur) => {
    offsets.push(total);
    return total + cur.length;
  }, 0);

  const outputBuf = wasm.call('bbmalloc', expectedOutputLength);
  const inputBuf = wasm.call('bbmalloc', totalLength);
  wasm.writeMemory(inputBuf, Buffer.concat(inputBuffers));
  const args = offsets.map(offset => inputBuf + offset);
  wasm.call(fnName, ...args, outputBuf);
  const output = Buffer.from(wasm.getMemorySlice(outputBuf, outputBuf + expectedOutputLength));
  wasm.call('bbfree', inputBuf);
  wasm.call('bbfree', outputBuf);
  return output;
}

/**
 * Computes a hash of a transaction request.
 * @param wasm - A module providing low-level wasm access.
 * @param txRequest - The transaction request.
 * @returns The hash of the transaction request.
 */
export function hashTxRequest(wasm: IWasmModule, txRequest: TxRequest): Buffer {
  wasm.call('pedersen__init');
  return wasmSyncCall(wasm, 'abis__hash_tx_request', txRequest, 32);
}

/**
 * Computes a function selector from a given function signature.
 * @param wasm - A module providing low-level wasm access.
 * @param funcSig - The function signature.
 * @returns The function selector.
 */
export function computeFunctionSelector(wasm: IWasmModule, funcSig: string): Buffer {
  return wasmSyncCall(
    wasm,
    'abis__compute_function_selector',
    // Important - explicit C-string compatibility with a null terminator!
    // In the future we want to move away from this fiddly C-string processing.
    Buffer.from(funcSig + '\0'),
    FUNCTION_SELECTOR_NUM_BYTES,
  );
}

/**
 * Computes a hash of a given verification key.
 * @param wasm - A module providing low-level wasm access.
 * @param vkBuf - The verification key.
 * @returns The hash of the verification key.
 */
export function hashVK(wasm: IWasmModule, vkBuf: Buffer) {
  wasm.call('pedersen__init');
  return wasmSyncCall(wasm, 'abis__hash_vk', vkBuf, 32);
}

/**
 * Computes a function leaf from a given preimage.
 * @param wasm - A module providing low-level wasm access.
 * @param fnLeaf - The function leaf preimage.
 * @returns The function leaf.
 */
export function computeFunctionLeaf(wasm: IWasmModule, fnLeaf: FunctionLeafPreimage): Fr {
  wasm.call('pedersen__init');
  return Fr.fromBuffer(wasmSyncCall(wasm, 'abis__compute_function_leaf', fnLeaf, 32));
}

/**
 * Computes a function tree root from function leaves.
 * @param wasm - A module providing low-level wasm access.
 * @param fnLeaves - The function leaves to be included in the contract function tree.
 * @returns The function tree root.
 */
export function computeFunctionTreeRoot(wasm: IWasmModule, fnLeaves: Fr[]) {
  const inputVector = serializeBufferArrayToVector(fnLeaves.map(fr => fr.toBuffer()));
  wasm.call('pedersen__init');
  const result = wasmSyncCall(wasm, 'abis__compute_function_tree_root', inputVector, 32);
  return Fr.fromBuffer(result);
}

/**
 * Computes a constructor hash.
 * @param wasm - A module providing low-level wasm access.
 * @param functionData - Constructor's function data.
 * @param argsHash - Constructor's arguments hashed.
 * @param constructorVKHash - Hash of the constructor's verification key.
 * @returns The constructor hash.
 */
export function hashConstructor(
  wasm: IWasmModule,
  functionData: FunctionData,
  argsHash: Fr,
  constructorVKHash: Buffer,
): Fr {
  wasm.call('pedersen__init');
  const result = inputBuffersToOutputBuffer(
    wasm,
    'abis__hash_constructor',
    [functionData.toBuffer(), argsHash.toBuffer(), constructorVKHash],
    32,
  );
  return Fr.fromBuffer(result);
}

/**
 * Computes a contract address.
 * @param wasm - A module providing low-level wasm access.
 * @param deployerPubKey - The pubkey of the contract deployer.
 * @param contractAddrSalt - The salt used as one of the inputs of the contract address computation.
 * @param fnTreeRoot - The function tree root of the contract being deployed.
 * @param constructorHash - The hash of the constructor.
 * @returns The contract address.
 */
export function computeContractAddress(
  wasm: IWasmModule,
  deployerPubKey: PublicKey,
  contractAddrSalt: Fr,
  fnTreeRoot: Fr,
  constructorHash: Fr,
): AztecAddress {
  wasm.call('pedersen__init');
  const result = inputBuffersToOutputBuffer(
    wasm,
    'abis__compute_contract_address',
    [deployerPubKey.toBuffer(), contractAddrSalt.toBuffer(), fnTreeRoot.toBuffer(), constructorHash.toBuffer()],
    32,
  );
  return new AztecAddress(result);
}

/**
 * Computes a partial address. Consists of all contract address components except the deployer public key.
 * @param wasm - A module providing low-level wasm access.
 * @param contractAddrSalt - The salt used as one of the inputs of the contract address computation.
 * @param fnTreeRoot - The function tree root of the contract being deployed.
 * @param constructorHash - The hash of the constructor.
 * @returns The partially constructed contract address.
 */
export function computePartialAddress(
  wasm: IWasmModule,
  contractAddrSalt: Fr,
  fnTreeRoot: Fr,
  constructorHash: Fr,
): Fr {
  wasm.call('pedersen__init');
  const result = inputBuffersToOutputBuffer(
    wasm,
    'abis__compute_partial_address',
    [contractAddrSalt.toBuffer(), fnTreeRoot.toBuffer(), constructorHash.toBuffer()],
    32,
  );
  return Fr.fromBuffer(result);
}

/**
 * Computes a contract address from its partial address and the pubkey.
 * @param wasm - A module providing low-level wasm access.
 * @param partial - The salt used as one of the inputs of the contract address computation.
 * @param fnTreeRoot - The function tree root of the contract being deployed.
 * @param constructorHash - The hash of the constructor.
 * @returns The partially constructed contract address.
 */
export function computeContractAddressFromPartial(
  wasm: IWasmModule,
  pubKey: PublicKey,
  partialAddress: Fr,
): AztecAddress {
  wasm.call('pedersen__init');
  const result = inputBuffersToOutputBuffer(
    wasm,
    'abis__compute_contract_address_from_partial',
    [pubKey.toBuffer(), partialAddress.toBuffer()],
    32,
  );
  return new AztecAddress(result);
}

/**
 * Computes a commitment nonce, which will be used to create a unique commitment.
 * @param wasm - A module providing low-level wasm access.
 * @param nullifierZero - The first nullifier in the tx.
 * @param commitmentIndex - The index of the commitment.
 * @returns A commitment nonce.
 */
export function computeCommitmentNonce(wasm: IWasmModule, nullifierZero: Fr, commitmentIndex: number): Fr {
  wasm.call('pedersen__init');
  return abisComputeCommitmentNonce(wasm, nullifierZero, new Fr(commitmentIndex));
}

/**
 * Computes a siloed commitment, given the contract address and the commitment itself.
 * A siloed commitment effectively namespaces a commitment to a specific contract.
 * @param wasm - A module providing low-level wasm access.
 * @param contract - The contract address
 * @param uniqueCommitment - The commitment to silo.
 * @returns A siloed commitment.
 */
export function siloCommitment(wasm: IWasmModule, contract: AztecAddress, uniqueCommitment: Fr): Fr {
  wasm.call('pedersen__init');
  return abisSiloCommitment(wasm, contract, uniqueCommitment);
}

/**
 * Computes a unique commitment. It includes a nonce which contains data that guarantees the commiment will be unique.
 * @param wasm - A module providing low-level wasm access.
 * @param nonce - The contract address.
 * @param siloedCommitment - An siloed commitment.
 * @returns A unique commitment.
 */
export function computeUniqueCommitment(wasm: IWasmModule, nonce: Fr, siloedCommitment: Fr): Fr {
  wasm.call('pedersen__init');
  return abisComputeUniqueCommitment(wasm, nonce, siloedCommitment);
}

/**
 * Computes a siloed nullifier, given the contract address and the inner nullifier.
 * A siloed nullifier effectively namespaces a nullifier to a specific contract.
 * @param wasm - A module providing low-level wasm access.
 * @param contract - The contract address.
 * @param innerNullifier - The nullifier to silo.
 * @returns A siloed nullifier.
 */
export function siloNullifier(wasm: IWasmModule, contract: AztecAddress, innerNullifier: Fr): Fr {
  wasm.call('pedersen__init');
  return abisSiloNullifier(wasm, contract, innerNullifier);
}

/**
 * Computes the block hash given the blocks globals and roots.
 * @param wasm - A module providing low-level wasm access.
 * @param globals - The global variables to put into the block hash.
 * @param privateDataTree - The root of the private data tree.
 * @param nullifierTreeRoot - The root of the nullifier tree.
 * @param contractTreeRoot - The root of the contract tree.
 * @param l1ToL2DataTreeRoot - The root of the l1 to l2 data tree.
 * @param publicDataTreeRoot - The root of the public data tree.
 * @returns The block hash.
 */
export function computeBlockHashWithGlobals(
  wasm: IWasmModule,
  globals: GlobalVariables,
  privateDataTreeRoot: Fr,
  nullifierTreeRoot: Fr,
  contractTreeRoot: Fr,
  l1ToL2DataTreeRoot: Fr,
  publicDataTreeRoot: Fr,
): Fr {
  wasm.call('pedersen__init');
  return abisComputeBlockHashWithGlobals(
    wasm,
    globals,
    privateDataTreeRoot,
    nullifierTreeRoot,
    contractTreeRoot,
    l1ToL2DataTreeRoot,
    publicDataTreeRoot,
  );
}

/**
 * Computes the block hash given the blocks globals and roots.
 * @param wasm - A module providing low-level wasm access.
 * @param globalsHash - The global variables hash to put into the block hash.
 * @param privateDataTree - The root of the private data tree.
 * @param nullifierTreeRoot - The root of the nullifier tree.
 * @param contractTreeRoot - The root of the contract tree.
 * @param l1ToL2DataTreeRoot - The root of the l1 to l2 data tree.
 * @param publicDataTreeRoot - The root of the public data tree.
 * @returns The block hash.
 */
export function computeBlockHash(
  wasm: IWasmModule,
  globalsHash: Fr,
  privateDataTreeRoot: Fr,
  nullifierTreeRoot: Fr,
  contractTreeRoot: Fr,
  l1ToL2DataTreeRoot: Fr,
  publicDataTreeRoot: Fr,
): Fr {
  wasm.call('pedersen__init');
  return abisComputeBlockHash(
    wasm,
    globalsHash,
    privateDataTreeRoot,
    nullifierTreeRoot,
    contractTreeRoot,
    l1ToL2DataTreeRoot,
    publicDataTreeRoot,
  );
}

/**
 * Computes the globals hash given the globals.
 * @param wasm - A module providing low-level wasm access.
 * @param globals - The global variables to put into the block hash.
 * @returns The globals hash.
 */
export function computeGlobalsHash(wasm: IWasmModule, globals: GlobalVariables): Fr {
  wasm.call('pedersen__init');
  return abisComputeGlobalsHash(wasm, globals);
}

/**
 * Computes a public data tree value ready for insertion.
 * @param wasm - A module providing low-level wasm access.
 * @param value - Raw public data tree value to hash into a tree-insertion-ready value.
 * @returns Value hash into a tree-insertion-ready value.

 */
export function computePublicDataTreeValue(wasm: IWasmModule, value: Fr): Fr {
  wasm.call('pedersen__init');
  return abisComputePublicDataTreeValue(wasm, value);
}

/**
 * Computes a public data tree index from contract address and storage slot.
 * @param wasm - A module providing low-level wasm access.
 * @param contractAddress - Contract where insertion is occurring.
 * @param storageSlot - Storage slot where insertion is occuring.
 * @returns Public data tree index computed from contract address and storage slot.

 */
export function computePublicDataTreeIndex(wasm: IWasmModule, contractAddress: Fr, storageSlot: Fr): Fr {
  wasm.call('pedersen__init');
  return abisComputePublicDataTreeIndex(wasm, contractAddress, storageSlot);
}

const ARGS_HASH_CHUNK_SIZE = 32;
const ARGS_HASH_CHUNK_COUNT = 16;

/**
 * Computes the hash of a list of arguments.
 * @param wasm - A module providing low-level wasm access.
 * @param args - Arguments to hash.
 * @returns Pedersen hash of the arguments.
 */
export function computeVarArgsHash(wasm: IWasmModule, args: Fr[]): Promise<Fr> {
  if (args.length === 0) return Promise.resolve(Fr.ZERO);
  if (args.length > ARGS_HASH_CHUNK_SIZE * ARGS_HASH_CHUNK_COUNT)
    throw new Error(`Cannot hash more than ${ARGS_HASH_CHUNK_SIZE * ARGS_HASH_CHUNK_COUNT} arguments`);
  wasm.call('pedersen__init');

  const wasmComputeVarArgs = (args: Fr[]) =>
    Fr.fromBuffer(wasmSyncCall(wasm, 'abis__compute_var_args_hash', new Vector(args), 32));

  let chunksHashes = chunk(args, ARGS_HASH_CHUNK_SIZE).map(c => {
    if (c.length < ARGS_HASH_CHUNK_SIZE) {
      c = padArrayEnd(c, Fr.ZERO, ARGS_HASH_CHUNK_SIZE);
    }
    return wasmComputeVarArgs(c);
  });

  if (chunksHashes.length < ARGS_HASH_CHUNK_COUNT) {
    chunksHashes = padArrayEnd(chunksHashes, Fr.ZERO, ARGS_HASH_CHUNK_COUNT);
  }

  return Promise.resolve(wasmComputeVarArgs(chunksHashes));
}

/**
 * Computes a contract leaf of the given contract.
 * @param wasm - Relevant WASM wrapper.
 * @param cd - The contract data of the deployed contract.
 * @returns The contract leaf.
 */
export function computeContractLeaf(wasm: IWasmModule, cd: NewContractData): Fr {
  wasm.call('pedersen__init');
  const value = wasmSyncCall(wasm, 'abis__compute_contract_leaf', cd, 32);
  return Fr.fromBuffer(value);
}

/**
 * Computes tx hash of a given transaction request.
 * @param wasm - Relevant WASM wrapper.
 * @param txRequest - The signed transaction request.
 * @returns The transaction hash.
 */
export function computeTxHash(wasm: IWasmModule, txRequest: TxRequest): Fr {
  wasm.call('pedersen__init');
  const value = wasmSyncCall(wasm, 'abis__compute_transaction_hash', txRequest, 32);
  return Fr.fromBuffer(value);
}

/**
 * Computes a call stack item hash.
 * @param wasm - Relevant WASM wrapper.
 * @param callStackItem - The call stack item.
 * @returns The call stack item hash.
 */
export function computeCallStackItemHash(
  wasm: IWasmModule,
  callStackItem: PrivateCallStackItem | PublicCallStackItem,
): Fr {
  if (callStackItem instanceof PrivateCallStackItem) {
    return computePrivateCallStackItemHash(wasm, callStackItem);
  } else if (callStackItem instanceof PublicCallStackItem) {
    return computePublicCallStackItemHash(wasm, callStackItem);
  } else {
    throw new Error(`Unexpected call stack item type`);
  }
}

/**
 * Computes a call stack item hash.
 * @param wasm - Relevant WASM wrapper.
 * @param callStackItem - The call stack item.
 * @returns The call stack item hash.
 */
export function computePrivateCallStackItemHash(wasm: IWasmModule, callStackItem: PrivateCallStackItem): Fr {
  wasm.call('pedersen__init');
  const value = wasmSyncCall(wasm, 'abis__compute_private_call_stack_item_hash', callStackItem, 32);
  return Fr.fromBuffer(value);
}

/**
 * Computes a call stack item hash.
 * @param wasm - Relevant WASM wrapper.
 * @param callStackItem - The call stack item.
 * @returns The call stack item hash.
 */
export function computePublicCallStackItemHash(wasm: IWasmModule, callStackItem: PublicCallStackItem): Fr {
  wasm.call('pedersen__init');
  const value = wasmSyncCall(wasm, 'abis__compute_public_call_stack_item_hash', callStackItem, 32);
  return Fr.fromBuffer(value);
}

/**
 * Computes a secret message hash for sending secret l1 to l2 messages.
 * @param secretMessage - The secret message.
 * @returns
 */
export function computeSecretMessageHash(wasm: IWasmModule, secretMessage: Fr) {
  wasm.call('pedersen__init');
  const value = wasmSyncCall(wasm, 'abis__compute_message_secret_hash', secretMessage, 32);
  return Fr.fromBuffer(value);
}
