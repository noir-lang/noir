import { AztecAddress } from '@aztec/foundation/aztec-address';
import { padArrayEnd } from '@aztec/foundation/collection';
import { keccak, pedersenHash, pedersenHashBuffer } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { boolToBuffer, numToUInt8, numToUInt16BE, numToUInt32BE } from '@aztec/foundation/serialize';

import { Buffer } from 'buffer';
import chunk from 'lodash.chunk';

import {
  FUNCTION_SELECTOR_NUM_BYTES,
  FUNCTION_TREE_HEIGHT,
  GeneratorIndex,
  PRIVATE_CIRCUIT_PUBLIC_INPUTS_HASH_INPUT_LENGTH,
  PUBLIC_CIRCUIT_PUBLIC_INPUTS_HASH_INPUT_LENGTH,
} from '../constants.gen.js';
import {
  CallContext,
  CompleteAddress,
  ContractDeploymentData,
  ContractStorageRead,
  ContractStorageUpdateRequest,
  FunctionData,
  FunctionLeafPreimage,
  GlobalVariables,
  NewContractData,
  PrivateCallStackItem,
  PrivateCircuitPublicInputs,
  PublicCallStackItem,
  PublicCircuitPublicInputs,
  SideEffect,
  SideEffectLinkedToNoteHash,
  TxContext,
  TxRequest,
  VerificationKey,
} from '../structs/index.js';
import { PublicKey } from '../types/index.js';
import { MerkleTreeCalculator } from './merkle_tree_calculator.js';

/**
 * Computes a hash of a transaction request.
 * @param txRequest - The transaction request.
 * @returns The hash of the transaction request.
 */
export function hashTxRequest(txRequest: TxRequest): Buffer {
  return computeTxHash(txRequest).toBuffer();
}

/**
 * Computes a function selector from a given function signature.
 * @param funcSig - The function signature.
 * @returns The function selector.
 */
export function computeFunctionSelector(funcSig: string): Buffer {
  return keccak(Buffer.from(funcSig)).subarray(0, FUNCTION_SELECTOR_NUM_BYTES);
}

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

/**
 * Computes a function leaf from a given preimage.
 * @param fnLeaf - The function leaf preimage.
 * @returns The function leaf.
 */
export function computeFunctionLeaf(fnLeaf: FunctionLeafPreimage): Fr {
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

let functionTreeRootCalculator: MerkleTreeCalculator | undefined;
/**
 * The "zero leaf" of the function tree is the hash of 5 zero fields.
 * TODO: Why can we not just use a zero field as the zero leaf? Complicates things perhaps unnecessarily?
 */
function getFunctionTreeRootCalculator() {
  if (!functionTreeRootCalculator) {
    const functionTreeZeroLeaf = pedersenHash(new Array(5).fill(Buffer.alloc(32)));
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
    .map(b => Fr.fromBuffer(b));
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
  return Fr.fromBuffer(
    pedersenHash(
      [computeFunctionDataHash(functionData).toBuffer(), argsHash.toBuffer(), constructorVKHash],
      GeneratorIndex.CONSTRUCTOR,
    ),
  );
}

/**
 * Computes a complete address.
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
 * @param contract - The contract address.
 * @param innerNullifier - The nullifier to silo.
 * @returns A siloed nullifier.
 */
export function siloNullifier(contract: AztecAddress, innerNullifier: Fr): Fr {
  return Fr.fromBuffer(pedersenHash([contract.toBuffer(), innerNullifier.toBuffer()], GeneratorIndex.OUTER_NULLIFIER));
}

/**
 * Computes the block hash given the blocks globals and roots.
 * @param globals - The global variables to put into the block hash.
 * @param noteHashTree - The root of the note hash tree.
 * @param nullifierTreeRoot - The root of the nullifier tree.
 * @param contractTreeRoot - The root of the contract tree.
 * @param l1ToL2DataTreeRoot - The root of the l1 to l2 data tree.
 * @param publicDataTreeRoot - The root of the public data tree.
 * @returns The block hash.
 */
// TODO(#3941)
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
 * @param globals - The global variables to put into the block hash.
 * @returns The globals hash.
 * TODO: move this to GlobalVariables?
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
  return Fr.fromBuffer(
    pedersenHash([contractAddress.toBuffer(), storageSlot.toBuffer()], GeneratorIndex.PUBLIC_LEAF_INDEX),
  );
}

const ARGS_HASH_CHUNK_SIZE = 32;
const ARGS_HASH_CHUNK_COUNT = 16;

/**
 * Computes the hash of a list of arguments.
 * @param args - Arguments to hash.
 * @returns Pedersen hash of the arguments.
 */
export function computeVarArgsHash(args: Fr[]) {
  if (args.length === 0) {
    return Fr.ZERO;
  }
  if (args.length > ARGS_HASH_CHUNK_SIZE * ARGS_HASH_CHUNK_COUNT) {
    throw new Error(`Cannot hash more than ${ARGS_HASH_CHUNK_SIZE * ARGS_HASH_CHUNK_COUNT} arguments`);
  }

  let chunksHashes = chunk(args, ARGS_HASH_CHUNK_SIZE).map(c => {
    if (c.length < ARGS_HASH_CHUNK_SIZE) {
      c = padArrayEnd(c, Fr.ZERO, ARGS_HASH_CHUNK_SIZE);
    }
    return Fr.fromBuffer(
      pedersenHash(
        c.map(a => a.toBuffer()),
        GeneratorIndex.FUNCTION_ARGS,
      ),
    );
  });

  if (chunksHashes.length < ARGS_HASH_CHUNK_COUNT) {
    chunksHashes = padArrayEnd(chunksHashes, Fr.ZERO, ARGS_HASH_CHUNK_COUNT);
  }

  return Fr.fromBuffer(
    pedersenHash(
      chunksHashes.map(a => a.toBuffer()),
      GeneratorIndex.FUNCTION_ARGS,
    ),
  );
}

/**
 * Computes a contract leaf of the given contract.
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
 *
 */
function computeCallContextHash(input: CallContext) {
  return pedersenHash(
    [
      input.msgSender.toBuffer(),
      input.storageContractAddress.toBuffer(),
      input.portalContractAddress.toBuffer(),
      input.functionSelector.toBuffer(),
      boolToBuffer(input.isDelegateCall, 32),
      boolToBuffer(input.isStaticCall, 32),
      boolToBuffer(input.isContractDeployment, 32),
      numToUInt32BE(input.startSideEffectCounter, 32),
    ],
    GeneratorIndex.CALL_CONTEXT,
  );
}

/**
 *
 */
function computePrivateInputsHash(input: PrivateCircuitPublicInputs) {
  const toHash = [
    computeCallContextHash(input.callContext),
    input.argsHash.toBuffer(),
    ...input.returnValues.map(fr => fr.toBuffer()),
    ...input.readRequests
      .map(rr => rr.toFields())
      .flat()
      .map(fr => fr.toBuffer()),
    ...input.newCommitments
      .map(n => n.toFields())
      .flat()
      .map(fr => fr.toBuffer()),
    ...input.newNullifiers
      .map(n => n.toFields())
      .flat()
      .map(fr => fr.toBuffer()),
    ...input.privateCallStackHashes.map(fr => fr.toBuffer()),
    ...input.publicCallStackHashes.map(fr => fr.toBuffer()),
    ...input.newL2ToL1Msgs.map(fr => fr.toBuffer()),
    input.endSideEffectCounter.toBuffer(),
    ...input.encryptedLogsHash.map(fr => fr.toBuffer()),
    ...input.unencryptedLogsHash.map(fr => fr.toBuffer()),
    input.encryptedLogPreimagesLength.toBuffer(),
    input.unencryptedLogPreimagesLength.toBuffer(),
    input.blockHeader.noteHashTreeRoot.toBuffer(),
    input.blockHeader.nullifierTreeRoot.toBuffer(),
    input.blockHeader.contractTreeRoot.toBuffer(),
    input.blockHeader.l1ToL2MessageTreeRoot.toBuffer(),
    input.blockHeader.archiveRoot.toBuffer(),
    input.blockHeader.publicDataTreeRoot.toBuffer(),
    input.blockHeader.globalVariablesHash.toBuffer(),
    computeContractDeploymentDataHash(input.contractDeploymentData).toBuffer(),
    input.chainId.toBuffer(),
    input.version.toBuffer(),
  ];
  if (toHash.length != PRIVATE_CIRCUIT_PUBLIC_INPUTS_HASH_INPUT_LENGTH) {
    throw new Error(
      `Incorrect number of input fields when hashing PrivateCircuitPublicInputs ${toHash.length}, ${PRIVATE_CIRCUIT_PUBLIC_INPUTS_HASH_INPUT_LENGTH}`,
    );
  }
  return pedersenHash(toHash, GeneratorIndex.PRIVATE_CIRCUIT_PUBLIC_INPUTS);
}

/**
 * Computes a call stack item hash.
 * @param callStackItem - The call stack item.
 * @returns The call stack item hash.
 */
export function computePrivateCallStackItemHash(callStackItem: PrivateCallStackItem): Fr {
  return Fr.fromBuffer(
    pedersenHash(
      [
        callStackItem.contractAddress.toBuffer(),
        computeFunctionDataHash(callStackItem.functionData).toBuffer(),
        computePrivateInputsHash(callStackItem.publicInputs),
      ],
      GeneratorIndex.CALL_STACK_ITEM,
    ),
  );
}

/**
 *
 */
function computeContractStorageUpdateRequestHash(input: ContractStorageUpdateRequest) {
  return pedersenHash(
    [input.storageSlot.toBuffer(), input.oldValue.toBuffer(), input.newValue.toBuffer()],
    GeneratorIndex.PUBLIC_DATA_UPDATE_REQUEST,
  );
}

/**
 *
 */
function computeContractStorageReadsHash(input: ContractStorageRead) {
  return pedersenHash([input.storageSlot.toBuffer(), input.currentValue.toBuffer()], GeneratorIndex.PUBLIC_DATA_READ);
}
/**
 *
 */
export function computeCommitmentsHash(input: SideEffect) {
  return pedersenHash([input.value.toBuffer(), input.counter.toBuffer()], GeneratorIndex.SIDE_EFFECT);
}

/**
 *
 */
export function computeNullifierHash(input: SideEffectLinkedToNoteHash) {
  return pedersenHash(
    [input.value.toBuffer(), input.noteHash.toBuffer(), input.counter.toBuffer()],
    GeneratorIndex.SIDE_EFFECT,
  );
}

/**
 *
 */
export function computePublicInputsHash(input: PublicCircuitPublicInputs) {
  const toHash = [
    computeCallContextHash(input.callContext),
    input.argsHash.toBuffer(),
    ...input.returnValues.map(fr => fr.toBuffer()),
    ...input.contractStorageUpdateRequests.map(computeContractStorageUpdateRequestHash),
    ...input.contractStorageReads.map(computeContractStorageReadsHash),
    ...input.publicCallStackHashes.map(fr => fr.toBuffer()),
    ...input.newCommitments.map(computeCommitmentsHash),
    ...input.newNullifiers.map(computeNullifierHash),
    ...input.newL2ToL1Msgs.map(fr => fr.toBuffer()),
    ...input.unencryptedLogsHash.map(fr => fr.toBuffer()),
    input.unencryptedLogPreimagesLength.toBuffer(),
    input.blockHeader.noteHashTreeRoot.toBuffer(),
    input.blockHeader.nullifierTreeRoot.toBuffer(),
    input.blockHeader.contractTreeRoot.toBuffer(),
    input.blockHeader.l1ToL2MessageTreeRoot.toBuffer(),
    input.blockHeader.archiveRoot.toBuffer(),
    input.blockHeader.publicDataTreeRoot.toBuffer(),
    input.blockHeader.globalVariablesHash.toBuffer(),
    input.proverAddress.toBuffer(),
  ];
  if (toHash.length != PUBLIC_CIRCUIT_PUBLIC_INPUTS_HASH_INPUT_LENGTH) {
    throw new Error('Incorrect number of input fields when hashing PublicCircuitPublicInputs');
  }
  return pedersenHash(toHash, GeneratorIndex.PUBLIC_CIRCUIT_PUBLIC_INPUTS);
}

/**
 * Computes a call stack item hash.
 * @param callStackItem - The call stack item.
 * @returns The call stack item hash.
 */
export function computePublicCallStackItemHash({
  contractAddress,
  functionData,
  publicInputs,
  isExecutionRequest,
}: PublicCallStackItem): Fr {
  if (isExecutionRequest) {
    const { callContext, argsHash } = publicInputs;
    publicInputs = PublicCircuitPublicInputs.empty();
    publicInputs.callContext = callContext;
    publicInputs.argsHash = argsHash;
  }

  return Fr.fromBuffer(
    pedersenHash(
      [
        contractAddress.toBuffer(),
        computeFunctionDataHash(functionData).toBuffer(),
        computePublicInputsHash(publicInputs),
      ],
      GeneratorIndex.CALL_STACK_ITEM,
    ),
  );
}

/**
 * Computes a secret message hash for sending secret l1 to l2 messages.
 * @param secretMessage - The secret message.
 * @returns
 */
export function computeSecretMessageHash(secretMessage: Fr) {
  return Fr.fromBuffer(pedersenHash([secretMessage.toBuffer()], GeneratorIndex.L1_TO_L2_MESSAGE_SECRET));
}
