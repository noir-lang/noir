import {
  CompleteAddress,
  MerkleTreeId,
  Note,
  NullifierMembershipWitness,
  PublicDataWitness,
  PublicKey,
  UnencryptedL2Log,
} from '@aztec/circuit-types';
import { BlockHeader, GrumpkinPrivateKey, PrivateCallStackItem, PublicCallRequest } from '@aztec/circuits.js';
import { FunctionSelector } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';

/**
 * A pair of public key and secret key.
 */
export interface KeyPair {
  /**
   * Public key.
   */
  publicKey: PublicKey;
  /**
   * Secret Key.
   */
  secretKey: GrumpkinPrivateKey;
}

/**
 * Information about a note needed during execution.
 */
export interface NoteData {
  /** The note. */
  note: Note;
  /** The contract address of the note. */
  contractAddress: AztecAddress;
  /** The storage slot of the note. */
  storageSlot: Fr;
  /** The nonce of the note. */
  nonce: Fr;
  /** The inner note hash of the note. */
  innerNoteHash: Fr;
  /** The corresponding nullifier of the note. Undefined for pending notes. */
  siloedNullifier?: Fr;
  /** The note's leaf index in the note hash tree. Undefined for pending notes. */
  index?: bigint;
}

/**
 * The data for L1 to L2 Messages provided by other data sources.
 */
export interface MessageLoadOracleInputs {
  /**
   * An collapsed array of fields containing all of the l1 to l2 message components.
   * `l1ToL2Message.toFieldArray()` -\> [sender, chainId, recipient, version, content, secretHash, deadline, fee]
   */
  message: Fr[];
  /**
   * The path in the merkle tree to the message.
   */
  siblingPath: Fr[];
  /**
   * The index of the message commitment in the merkle tree.
   */
  index: bigint;
}

/**
 * Oracle with typed parameters and typed return values.
 * Methods that require read and/or write will have to be implemented based on the context (public, private, or view)
 * and are unavailable by default.
 */
export abstract class TypedOracle {
  getRandomField(): Fr {
    return Fr.random();
  }

  packArguments(_args: Fr[]): Promise<Fr> {
    throw new Error('Not available.');
  }

  getNullifierKeyPair(_accountAddress: AztecAddress): Promise<KeyPair> {
    throw new Error('Not available.');
  }

  getPublicKeyAndPartialAddress(_address: AztecAddress): Promise<Fr[] | undefined> {
    throw new Error('Not available.');
  }

  getMembershipWitness(_blockNumber: number, _treeId: MerkleTreeId, _leafValue: Fr): Promise<Fr[] | undefined> {
    throw new Error('Not available.');
  }

  getSiblingPath(_blockNumber: number, _treeId: MerkleTreeId, _leafIndex: Fr): Promise<Fr[]> {
    throw new Error('Not available.');
  }

  getNullifierMembershipWitness(_blockNumber: number, _nullifier: Fr): Promise<NullifierMembershipWitness | undefined> {
    throw new Error('Not available.');
  }

  getPublicDataTreeWitness(_blockNumber: number, _leafSlot: Fr): Promise<PublicDataWitness | undefined> {
    throw new Error('Not available.');
  }

  getLowNullifierMembershipWitness(
    _blockNumber: number,
    _nullifier: Fr,
  ): Promise<NullifierMembershipWitness | undefined> {
    throw new Error('Not available.');
  }

  getBlockHeader(_blockNumber: number): Promise<BlockHeader | undefined> {
    throw new Error('Not available.');
  }

  // TODO(#3564) - Nuke this oracle and inject the number directly to context
  getNullifierRootBlockNumber(_nullifierTreeRoot: Fr): Promise<number | undefined> {
    throw new Error('Not available.');
  }

  getCompleteAddress(_address: AztecAddress): Promise<CompleteAddress> {
    throw new Error('Not available.');
  }

  getAuthWitness(_messageHash: Fr): Promise<Fr[] | undefined> {
    throw new Error('Not available.');
  }

  popCapsule(): Promise<Fr[]> {
    throw new Error('Not available.');
  }

  getNotes(
    _storageSlot: Fr,
    _numSelects: number,
    _selectBy: number[],
    _selectValues: Fr[],
    _selectComparators: number[],
    _sortBy: number[],
    _sortOrder: number[],
    _limit: number,
    _offset: number,
  ): Promise<NoteData[]> {
    throw new Error('Not available.');
  }

  notifyCreatedNote(_storageSlot: Fr, _note: Fr[], _innerNoteHash: Fr): void {
    throw new Error('Not available.');
  }

  notifyNullifiedNote(_innerNullifier: Fr, _innerNoteHash: Fr): Promise<void> {
    throw new Error('Not available.');
  }

  checkNullifierExists(_innerNullifier: Fr): Promise<boolean> {
    throw new Error('Not available.');
  }

  getL1ToL2Message(_msgKey: Fr): Promise<MessageLoadOracleInputs> {
    throw new Error('Not available.');
  }

  getPortalContractAddress(_contractAddress: AztecAddress): Promise<EthAddress> {
    throw new Error('Not available.');
  }

  storageRead(_startStorageSlot: Fr, _numberOfElements: number): Promise<Fr[]> {
    throw new Error('Not available.');
  }

  storageWrite(_startStorageSlot: Fr, _values: Fr[]): Promise<Fr[]> {
    throw new Error('Not available.');
  }

  emitEncryptedLog(_contractAddress: AztecAddress, _storageSlot: Fr, _publicKey: PublicKey, _log: Fr[]): void {
    throw new Error('Not available.');
  }

  emitUnencryptedLog(_log: UnencryptedL2Log): void {
    throw new Error('Not available.');
  }

  callPrivateFunction(
    _targetContractAddress: AztecAddress,
    _functionSelector: FunctionSelector,
    _argsHash: Fr,
    _sideffectCounter: number,
  ): Promise<PrivateCallStackItem> {
    throw new Error('Not available.');
  }

  callPublicFunction(
    _targetContractAddress: AztecAddress,
    _functionSelector: FunctionSelector,
    _argsHash: Fr,
  ): Promise<Fr[]> {
    throw new Error('Not available.');
  }

  enqueuePublicFunctionCall(
    _targetContractAddress: AztecAddress,
    _functionSelector: FunctionSelector,
    _argsHash: Fr,
    _sideffectCounter: number,
  ): Promise<PublicCallRequest> {
    throw new Error('Not available.');
  }
}
