import { padArrayEnd } from '@aztec/foundation/collection';
import { keccak, pedersenHash } from '@aztec/foundation/crypto';
import { numToUInt32BE } from '@aztec/foundation/serialize';
import { IWasmModule } from '@aztec/foundation/wasm';

import { Buffer } from 'buffer';
import chunk from 'lodash.chunk';

import {
  AztecAddress,
  CompleteAddress,
  ContractDeploymentData,
  FUNCTION_SELECTOR_NUM_BYTES,
  Fr,
  FunctionData,
  FunctionLeafPreimage,
  GeneratorIndex,
  GlobalVariables,
  NewContractData,
  PrivateCallStackItem,
  PublicCallStackItem,
  PublicKey,
  TxContext,
  TxRequest,
} from '../index.js';
import { boolToBuffer, serializeBufferArrayToVector } from '../utils/serialize.js';

/**
 * Synchronously calls a wasm function.
 * @param wasm - The wasm wrapper.
 * @param fnName - The name of the function to call.
 * @param input - The input buffer or object serializable to a buffer.
 * @param expectedOutputLength - The expected length of the output buffer.
 * @returns The output buffer.
 */
function wasmSyncCall(
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
 * Computes a hash of a transaction request.
 * @param wasm - A module providing low-level wasm access.
 * @param txRequest - The transaction request.
 * @returns The hash of the transaction request.
 */
export function hashTxRequest(txRequest: TxRequest): Buffer {
  return computeTxHash(txRequest).toBuffer();
}

/**
 * Computes a function selector from a given function signature.
 * @param wasm - A module providing low-level wasm access.
 * @param funcSig - The function signature.
 * @returns The function selector.
 */
export function computeFunctionSelector(funcSig: string): Buffer {
  return keccak(Buffer.from(funcSig)).subarray(0, FUNCTION_SELECTOR_NUM_BYTES);
}

/**
 * Computes a hash of a given verification key.
 * @param wasm - A module providing low-level wasm access.
 * @param vkBuf - The verification key.
 * @returns The hash of the verification key.
 */
export function hashVK(wasm: IWasmModule, vkBuf: Buffer) {
  return wasmSyncCall(wasm, 'abis__hash_vk', vkBuf, 32);
}

/**
 * Computes a function leaf from a given preimage.
 * @param wasm - A module providing low-level wasm access.
 * @param fnLeaf - The function leaf preimage.
 * @returns The function leaf.
 */
export function computeFunctionLeaf(fnLeaf: FunctionLeafPreimage): Fr {
  // return Fr.fromBuffer(wasmSyncCall(wasm, 'abis__compute_function_leaf', fnLeaf, 32));
  return Fr.fromBuffer(
    pedersenHash(
      [
        numToUInt32BE(fnLeaf.functionSelector.value, 32),
        boolToBuffer(fnLeaf.isInternal, 32),
        boolToBuffer(fnLeaf.isPrivate, 32),
        fnLeaf.vkHash.toBuffer(),
        fnLeaf.acirHash.toBuffer(),
      ],
      GeneratorIndex.FUNCTION_LEAF,
    ),
  );
}

/**
 * Computes a function tree root from function leaves.
 * @param wasm - A module providing low-level wasm access.
 * @param fnLeaves - The function leaves to be included in the contract function tree.
 * @returns The function tree root.
 */
export function computeFunctionTreeRoot(wasm: IWasmModule, fnLeaves: Fr[]) {
  const inputVector = serializeBufferArrayToVector(fnLeaves.map(fr => fr.toBuffer()));
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
export function hashConstructor(functionData: FunctionData, argsHash: Fr, constructorVKHash: Buffer): Fr {
  return Fr.fromBuffer(
    pedersenHash(
      [computeFunctionDataHash(functionData).toBuffer(), argsHash.toBuffer(), constructorVKHash],
      GeneratorIndex.CONSTRUCTOR,
    ),
  );
}

/**
 * Computes a complete address.
 * @param wasm - A module providing low-level wasm access.
 * @param deployerPubKey - The pubkey of the contract deployer.
 * @param contractAddrSalt - The salt used as one of the inputs of the contract address computation.
 * @param fnTreeRoot - The function tree root of the contract being deployed.
 * @param constructorHash - The hash of the constructor.
 * @returns The complete address.
 */
export function computeCompleteAddress(
  deployerPubKey: PublicKey,
  contractAddrSalt: Fr,
  fnTreeRoot: Fr,
  constructorHash: Fr,
): CompleteAddress {
  const partialAddress = computePartialAddress(contractAddrSalt, fnTreeRoot, constructorHash);
  return new CompleteAddress(
    computeContractAddressFromPartial(deployerPubKey, partialAddress),
    deployerPubKey,
    partialAddress,
  );
}

/**
 *
 */
function computePartialAddress(contractAddrSalt: Fr, fnTreeRoot: Fr, constructorHash: Fr) {
  return Fr.fromBuffer(
    pedersenHash(
      [
        Fr.ZERO.toBuffer(),
        Fr.ZERO.toBuffer(),
        contractAddrSalt.toBuffer(),
        fnTreeRoot.toBuffer(),
        constructorHash.toBuffer(),
      ],
      GeneratorIndex.PARTIAL_ADDRESS,
    ),
  );
}

/**
 * Computes a contract address from its partial address and the pubkey.
 * @param wasm - A module providing low-level wasm access.
 * @param partial - The salt used as one of the inputs of the contract address computation.
 * @param fnTreeRoot - The function tree root of the contract being deployed.
 * @param constructorHash - The hash of the constructor.
 * @returns The partially constructed contract address.
 */
export function computeContractAddressFromPartial(pubKey: PublicKey, partialAddress: Fr): AztecAddress {
  const result = pedersenHash(
    [pubKey.x.toBuffer(), pubKey.y.toBuffer(), partialAddress.toBuffer()],
    GeneratorIndex.CONTRACT_ADDRESS,
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
export function computeCommitmentNonce(nullifierZero: Fr, commitmentIndex: number): Fr {
  return Fr.fromBuffer(
    pedersenHash([nullifierZero.toBuffer(), numToUInt32BE(commitmentIndex, 32)], GeneratorIndex.COMMITMENT_NONCE),
  );
}

/**
 * Computes a siloed commitment, given the contract address and the commitment itself.
 * A siloed commitment effectively namespaces a commitment to a specific contract.
 * @param wasm - A module providing low-level wasm access.
 * @param contract - The contract address
 * @param innerCommitment - The commitment to silo.
 * @returns A siloed commitment.
 */
export function siloCommitment(contract: AztecAddress, innerCommitment: Fr): Fr {
  return Fr.fromBuffer(
    pedersenHash([contract.toBuffer(), innerCommitment.toBuffer()], GeneratorIndex.SILOED_COMMITMENT),
  );
}

/**
 * Computes a unique commitment. It includes a nonce which contains data that guarantees the commitment will be unique.
 * @param wasm - A module providing low-level wasm access.
 * @param nonce - The contract address.
 * @param siloedCommitment - An siloed commitment.
 * @returns A unique commitment.
 */
export function computeUniqueCommitment(nonce: Fr, siloedCommitment: Fr): Fr {
  return Fr.fromBuffer(pedersenHash([nonce.toBuffer(), siloedCommitment.toBuffer()], GeneratorIndex.UNIQUE_COMMITMENT));
}

/**
 * Computes a siloed nullifier, given the contract address and the inner nullifier.
 * A siloed nullifier effectively namespaces a nullifier to a specific contract.
 * @param wasm - A module providing low-level wasm access.
 * @param contract - The contract address.
 * @param innerNullifier - The nullifier to silo.
 * @returns A siloed nullifier.
 */
export function siloNullifier(contract: AztecAddress, innerNullifier: Fr): Fr {
  return Fr.fromBuffer(pedersenHash([contract.toBuffer(), innerNullifier.toBuffer()], GeneratorIndex.OUTER_NULLIFIER));
}

/**
 * Computes the block hash given the blocks globals and roots.
 * @param wasm - A module providing low-level wasm access.
 * @param globals - The global variables to put into the block hash.
 * @param noteHashTree - The root of the note hash tree.
 * @param nullifierTreeRoot - The root of the nullifier tree.
 * @param contractTreeRoot - The root of the contract tree.
 * @param l1ToL2DataTreeRoot - The root of the l1 to l2 data tree.
 * @param publicDataTreeRoot - The root of the public data tree.
 * @returns The block hash.
 */
export function computeBlockHashWithGlobals(
  globals: GlobalVariables,
  noteHashTreeRoot: Fr,
  nullifierTreeRoot: Fr,
  contractTreeRoot: Fr,
  l1ToL2DataTreeRoot: Fr,
  publicDataTreeRoot: Fr,
): Fr {
  return computeBlockHash(
    computeGlobalsHash(globals),
    noteHashTreeRoot,
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
 * @param noteHashTree - The root of the note hash tree.
 * @param nullifierTreeRoot - The root of the nullifier tree.
 * @param contractTreeRoot - The root of the contract tree.
 * @param l1ToL2DataTreeRoot - The root of the l1 to l2 data tree.
 * @param publicDataTreeRoot - The root of the public data tree.
 * @returns The block hash.
 */
export function computeBlockHash(
  globalsHash: Fr,
  noteHashTreeRoot: Fr,
  nullifierTreeRoot: Fr,
  contractTreeRoot: Fr,
  l1ToL2DataTreeRoot: Fr,
  publicDataTreeRoot: Fr,
): Fr {
  return Fr.fromBuffer(
    pedersenHash(
      [
        globalsHash.toBuffer(),
        noteHashTreeRoot.toBuffer(),
        nullifierTreeRoot.toBuffer(),
        contractTreeRoot.toBuffer(),
        l1ToL2DataTreeRoot.toBuffer(),
        publicDataTreeRoot.toBuffer(),
      ],
      GeneratorIndex.BLOCK_HASH,
    ),
  );
}

/**
 * Computes the globals hash given the globals.
 * @param wasm - A module providing low-level wasm access.
 * @param globals - The global variables to put into the block hash.
 * @returns The globals hash.
 */
export function computeGlobalsHash(globals: GlobalVariables): Fr {
  return Fr.fromBuffer(
    pedersenHash(
      [
        globals.chainId.toBuffer(),
        globals.version.toBuffer(),
        globals.blockNumber.toBuffer(),
        globals.timestamp.toBuffer(),
      ],
      GeneratorIndex.GLOBAL_VARIABLES,
    ),
  );
}

/**
 * Computes a public data tree value ready for insertion.
 * @param wasm - A module providing low-level wasm access.
 * @param value - Raw public data tree value to hash into a tree-insertion-ready value.
 * @returns Value hash into a tree-insertion-ready value.

 */
export function computePublicDataTreeValue(value: Fr): Fr {
  return value;
}

/**
 * Computes a public data tree index from contract address and storage slot.
 * @param wasm - A module providing low-level wasm access.
 * @param contractAddress - Contract where insertion is occurring.
 * @param storageSlot - Storage slot where insertion is occurring.
 * @returns Public data tree index computed from contract address and storage slot.

 */
export function computePublicDataTreeIndex(contractAddress: AztecAddress, storageSlot: Fr): Fr {
  return Fr.fromBuffer(
    pedersenHash([contractAddress.toBuffer(), storageSlot.toBuffer()], GeneratorIndex.PUBLIC_LEAF_INDEX),
  );
}

const ARGS_HASH_CHUNK_SIZE = 32;
const ARGS_HASH_CHUNK_COUNT = 16;

/**
 * Computes the hash of a list of arguments.
 * @param wasm - A module providing low-level wasm access.
 * @param args - Arguments to hash.
 * @returns Pedersen hash of the arguments.
 */
export function computeVarArgsHash(args: Fr[]) {
  if (args.length === 0) return Fr.ZERO;
  if (args.length > ARGS_HASH_CHUNK_SIZE * ARGS_HASH_CHUNK_COUNT)
    throw new Error(`Cannot hash more than ${ARGS_HASH_CHUNK_SIZE * ARGS_HASH_CHUNK_COUNT} arguments`);

  const wasmComputeVarArgs = (args: Fr[]) =>
    Fr.fromBuffer(
      pedersenHash(
        args.map(a => a.toBuffer()),
        GeneratorIndex.FUNCTION_ARGS,
      ),
    );

  let chunksHashes = chunk(args, ARGS_HASH_CHUNK_SIZE).map(c => {
    if (c.length < ARGS_HASH_CHUNK_SIZE) {
      c = padArrayEnd(c, Fr.ZERO, ARGS_HASH_CHUNK_SIZE);
    }
    return wasmComputeVarArgs(c);
  });

  if (chunksHashes.length < ARGS_HASH_CHUNK_COUNT) {
    chunksHashes = padArrayEnd(chunksHashes, Fr.ZERO, ARGS_HASH_CHUNK_COUNT);
  }

  return wasmComputeVarArgs(chunksHashes);
}

/**
 * Computes a contract leaf of the given contract.
 * @param wasm - Relevant WASM wrapper.
 * @param cd - The contract data of the deployed contract.
 * @returns The contract leaf.
 */
export function computeContractLeaf(cd: NewContractData): Fr {
  if (cd.contractAddress.isZero() && cd.portalContractAddress.isZero() && cd.functionTreeRoot.isZero()) {
    return new Fr(0);
  }
  return Fr.fromBuffer(
    pedersenHash(
      [cd.contractAddress.toBuffer(), cd.portalContractAddress.toBuffer(), cd.functionTreeRoot.toBuffer()],
      GeneratorIndex.CONTRACT_LEAF,
    ),
  );
}

/**
 * Computes tx hash of a given transaction request.
 * @param wasm - Relevant WASM wrapper.
 * @param txRequest - The signed transaction request.
 * @returns The transaction hash.
 */
export function computeTxHash(txRequest: TxRequest): Fr {
  return Fr.fromBuffer(
    pedersenHash(
      [
        txRequest.origin.toBuffer(),
        computeFunctionDataHash(txRequest.functionData).toBuffer(),
        txRequest.argsHash.toBuffer(),
        computeTxContextHash(txRequest.txContext).toBuffer(),
      ],
      GeneratorIndex.TX_REQUEST,
    ),
  );
}

/**
 *
 */
function computeFunctionDataHash(functionData: FunctionData): Fr {
  return Fr.fromBuffer(
    pedersenHash(
      [
        functionData.selector.toBuffer(32),
        new Fr(functionData.isInternal).toBuffer(),
        new Fr(functionData.isPrivate).toBuffer(),
        new Fr(functionData.isConstructor).toBuffer(),
      ],
      GeneratorIndex.FUNCTION_DATA,
    ),
  );
}

/**
 *
 */
function computeTxContextHash(txContext: TxContext): Fr {
  return Fr.fromBuffer(
    pedersenHash(
      [
        new Fr(txContext.isFeePaymentTx).toBuffer(),
        new Fr(txContext.isRebatePaymentTx).toBuffer(),
        new Fr(txContext.isContractDeploymentTx).toBuffer(),
        computeContractDeploymentDataHash(txContext.contractDeploymentData).toBuffer(),
        txContext.chainId.toBuffer(),
        txContext.version.toBuffer(),
      ],
      GeneratorIndex.TX_CONTEXT,
    ),
  );
}

/**
 *
 */
function computeContractDeploymentDataHash(data: ContractDeploymentData): Fr {
  return Fr.fromBuffer(
    pedersenHash(
      [
        data.deployerPublicKey.x.toBuffer(),
        data.deployerPublicKey.y.toBuffer(),
        data.constructorVkHash.toBuffer(),
        data.functionTreeRoot.toBuffer(),
        data.contractAddressSalt.toBuffer(),
        data.portalContractAddress.toBuffer(),
      ],
      GeneratorIndex.CONTRACT_DEPLOYMENT_DATA,
    ),
  );
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
  const value = wasmSyncCall(wasm, 'abis__compute_private_call_stack_item_hash', callStackItem, 32);
  return Fr.fromBuffer(value);
  // return Fr.fromBuffer(
  //   pedersenHashWithHashIndex(
  //     [
  //       callStackItem.contractAddress.toBuffer(),
  //       computeFunctionDataHash(callStackItem.functionData).toBuffer(),
  //       computePublicInputsHash(callStackItem.publicInputs).toBuffer(),
  //     ],
  //     GeneratorIndex.CALL_STACK_ITEM,
  //   ),
  // );
}

/**
 * Computes a call stack item hash.
 * @param wasm - Relevant WASM wrapper.
 * @param callStackItem - The call stack item.
 * @returns The call stack item hash.
 */
export function computePublicCallStackItemHash(wasm: IWasmModule, callStackItem: PublicCallStackItem): Fr {
  const value = wasmSyncCall(wasm, 'abis__compute_public_call_stack_item_hash', callStackItem, 32);
  return Fr.fromBuffer(value);
  // return Fr.fromBuffer(
  //   pedersenHashWithHashIndex(
  //     [
  //       callStackItem.contractAddress.toBuffer(),
  //       callStackItem.functionData.toBuffer(),
  //       callStackItem.publicInputs.toBuffer(),
  //     ],
  //     GeneratorIndex.CALL_STACK_ITEM,
  //   ),
  // );
}

/**
 * Computes a secret message hash for sending secret l1 to l2 messages.
 * @param secretMessage - The secret message.
 * @returns
 */
export function computeSecretMessageHash(secretMessage: Fr) {
  return Fr.fromBuffer(pedersenHash([secretMessage.toBuffer()], GeneratorIndex.L1_TO_L2_MESSAGE_SECRET));
}
