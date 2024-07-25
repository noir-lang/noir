import {
  type CompleteAddress,
  type MerkleTreeId,
  type Note,
  type NoteStatus,
  type NullifierMembershipWitness,
  type PublicDataWitness,
  type PublicKey,
  type SiblingPath,
  type UnencryptedL2Log,
} from '@aztec/circuit-types';
import { type Header, type KeyValidationRequest, type L1_TO_L2_MSG_TREE_HEIGHT } from '@aztec/circuits.js';
import { type FunctionSelector, type NoteSelector } from '@aztec/foundation/abi';
import { type AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { type ContractInstance } from '@aztec/types/contracts';

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

export class MessageLoadOracleInputs<N extends number> {
  constructor(
    /** The index of the message commitment in the merkle tree. */
    public index: bigint,
    /** The path in the merkle tree to the message. */
    public siblingPath: SiblingPath<N>,
  ) {}

  toFields(): Fr[] {
    return [new Fr(this.index), ...this.siblingPath.toFields()];
  }
}

class OracleMethodNotAvailableError extends Error {
  constructor(methodName: string) {
    super(`Oracle method ${methodName} is not available.`);
  }
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

  packArgumentsArray(_args: Fr[]): Promise<Fr> {
    throw new OracleMethodNotAvailableError('packArgumentsArray');
  }

  packReturns(_returns: Fr[]): Promise<Fr> {
    throw new OracleMethodNotAvailableError('packReturns');
  }

  unpackReturns(_returnsHash: Fr): Promise<Fr[]> {
    throw new OracleMethodNotAvailableError('unpackReturns');
  }

  getBlockNumber(): Promise<number> {
    throw new OracleMethodNotAvailableError('getBlockNumber');
  }

  getContractAddress(): Promise<AztecAddress> {
    throw new OracleMethodNotAvailableError('getContractAddress');
  }

  getChainId(): Promise<Fr> {
    throw new OracleMethodNotAvailableError('getChainId');
  }

  getVersion(): Promise<Fr> {
    throw new OracleMethodNotAvailableError('getVersion');
  }

  getKeyValidationRequest(_pkMHash: Fr): Promise<KeyValidationRequest> {
    throw new OracleMethodNotAvailableError('getKeyValidationRequest');
  }

  getContractInstance(_address: AztecAddress): Promise<ContractInstance> {
    throw new OracleMethodNotAvailableError('getContractInstance');
  }

  getMembershipWitness(_blockNumber: number, _treeId: MerkleTreeId, _leafValue: Fr): Promise<Fr[] | undefined> {
    throw new OracleMethodNotAvailableError('getMembershipWitness');
  }

  getSiblingPath(_blockNumber: number, _treeId: MerkleTreeId, _leafIndex: Fr): Promise<Fr[]> {
    throw new OracleMethodNotAvailableError('getSiblingPath');
  }

  getNullifierMembershipWitness(_blockNumber: number, _nullifier: Fr): Promise<NullifierMembershipWitness | undefined> {
    throw new OracleMethodNotAvailableError('getNullifierMembershipWitness');
  }

  getPublicDataTreeWitness(_blockNumber: number, _leafSlot: Fr): Promise<PublicDataWitness | undefined> {
    throw new OracleMethodNotAvailableError('getPublicDataTreeWitness');
  }

  getLowNullifierMembershipWitness(
    _blockNumber: number,
    _nullifier: Fr,
  ): Promise<NullifierMembershipWitness | undefined> {
    throw new OracleMethodNotAvailableError('getLowNullifierMembershipWitness');
  }

  getHeader(_blockNumber: number): Promise<Header | undefined> {
    throw new OracleMethodNotAvailableError('getHeader');
  }

  getCompleteAddress(_account: AztecAddress): Promise<CompleteAddress> {
    throw new OracleMethodNotAvailableError('getCompleteAddress');
  }

  getAuthWitness(_messageHash: Fr): Promise<Fr[] | undefined> {
    throw new OracleMethodNotAvailableError('getAuthWitness');
  }

  popCapsule(): Promise<Fr[]> {
    throw new OracleMethodNotAvailableError('popCapsule');
  }

  getNotes(
    _storageSlot: Fr,
    _numSelects: number,
    _selectByIndexes: number[],
    _selectByOffsets: number[],
    _selectByLengths: number[],
    _selectValues: Fr[],
    _selectComparators: number[],
    _sortByIndexes: number[],
    _sortByOffsets: number[],
    _sortByLengths: number[],
    _sortOrder: number[],
    _limit: number,
    _offset: number,
    _status: NoteStatus,
  ): Promise<NoteData[]> {
    throw new OracleMethodNotAvailableError('getNotes');
  }

  notifyCreatedNote(
    _storageSlot: Fr,
    _noteTypeId: NoteSelector,
    _note: Fr[],
    _innerNoteHash: Fr,
    _counter: number,
  ): void {
    throw new OracleMethodNotAvailableError('notifyCreatedNote');
  }

  notifyNullifiedNote(_innerNullifier: Fr, _innerNoteHash: Fr, _counter: number): Promise<void> {
    throw new OracleMethodNotAvailableError('notifyNullifiedNote');
  }

  checkNullifierExists(_innerNullifier: Fr): Promise<boolean> {
    throw new OracleMethodNotAvailableError('checkNullifierExists');
  }

  getL1ToL2MembershipWitness(
    _contractAddress: AztecAddress,
    _messageHash: Fr,
    _secret: Fr,
  ): Promise<MessageLoadOracleInputs<typeof L1_TO_L2_MSG_TREE_HEIGHT>> {
    throw new OracleMethodNotAvailableError('getL1ToL2MembershipWitness');
  }

  storageRead(
    _contractAddress: Fr,
    _startStorageSlot: Fr,
    _blockNumber: number,
    _numberOfElements: number,
  ): Promise<Fr[]> {
    throw new OracleMethodNotAvailableError('storageRead');
  }

  storageWrite(_startStorageSlot: Fr, _values: Fr[]): Promise<Fr[]> {
    throw new OracleMethodNotAvailableError('storageWrite');
  }

  emitEncryptedEventLog(
    _contractAddress: AztecAddress,
    _randomness: Fr,
    _encryptedEvent: Buffer,
    _counter: number,
  ): void {
    throw new OracleMethodNotAvailableError('emitEncryptedEventLog');
  }

  emitEncryptedNoteLog(_noteHashCounter: number, _encryptedNote: Buffer, _counter: number): void {
    throw new OracleMethodNotAvailableError('emitEncryptedNoteLog');
  }

  computeEncryptedEventLog(
    _contractAddress: AztecAddress,
    _randomness: Fr,
    _eventTypeId: Fr,
    _ovKeys: KeyValidationRequest,
    _ivpkM: PublicKey,
    _recipient: AztecAddress,
    _preimage: Fr[],
  ): Buffer {
    throw new OracleMethodNotAvailableError('computeEncryptedEventLog');
  }

  computeEncryptedNoteLog(
    _contractAddress: AztecAddress,
    _storageSlot: Fr,
    _noteTypeId: NoteSelector,
    _ovKeys: KeyValidationRequest,
    _ivpkM: PublicKey,
    _recipient: AztecAddress,
    _preimage: Fr[],
  ): Buffer {
    throw new OracleMethodNotAvailableError('computeEncryptedNoteLog');
  }

  emitUnencryptedLog(_log: UnencryptedL2Log, _counter: number): void {
    throw new OracleMethodNotAvailableError('emitUnencryptedLog');
  }

  emitContractClassUnencryptedLog(_log: UnencryptedL2Log, _counter: number): Fr {
    throw new OracleMethodNotAvailableError('emitContractClassUnencryptedLog');
  }

  callPrivateFunction(
    _targetContractAddress: AztecAddress,
    _functionSelector: FunctionSelector,
    _argsHash: Fr,
    _sideEffectCounter: number,
    _isStaticCall: boolean,
    _isDelegateCall: boolean,
  ): Promise<{ endSideEffectCounter: Fr; returnsHash: Fr }> {
    throw new OracleMethodNotAvailableError('callPrivateFunction');
  }

  callPublicFunction(
    _targetContractAddress: AztecAddress,
    _functionSelector: FunctionSelector,
    _argsHash: Fr,
    _sideEffectCounter: number,
    _isStaticCall: boolean,
    _isDelegateCall: boolean,
  ): Promise<Fr[]> {
    throw new OracleMethodNotAvailableError('callPublicFunction');
  }

  enqueuePublicFunctionCall(
    _targetContractAddress: AztecAddress,
    _functionSelector: FunctionSelector,
    _argsHash: Fr,
    _sideEffectCounter: number,
    _isStaticCall: boolean,
    _isDelegateCall: boolean,
  ): Promise<void> {
    throw new OracleMethodNotAvailableError('enqueuePublicFunctionCall');
  }

  setPublicTeardownFunctionCall(
    _targetContractAddress: AztecAddress,
    _functionSelector: FunctionSelector,
    _argsHash: Fr,
    _sideEffectCounter: number,
    _isStaticCall: boolean,
    _isDelegateCall: boolean,
  ): Promise<void> {
    throw new OracleMethodNotAvailableError('setPublicTeardownFunctionCall');
  }

  aes128Encrypt(_input: Buffer, _initializationVector: Buffer, _key: Buffer): Buffer {
    throw new OracleMethodNotAvailableError('encrypt');
  }

  debugLog(_message: string, _fields: Fr[]): void {
    throw new OracleMethodNotAvailableError('debugLog');
  }
}
