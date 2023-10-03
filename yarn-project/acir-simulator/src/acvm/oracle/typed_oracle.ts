import { PrivateCallStackItem, PublicCallRequest } from '@aztec/circuits.js';
import { FunctionSelector } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr, GrumpkinScalar } from '@aztec/foundation/fields';
import { CompleteAddress, PublicKey, UnencryptedL2Log } from '@aztec/types';

/**
 * Information about a note needed during execution.
 */
export interface NoteData {
  /** The contract address of the note. */
  contractAddress: AztecAddress;
  /** The storage slot of the note. */
  storageSlot: Fr;
  /** The nonce of the note. */
  nonce: Fr;
  /** The preimage of the note */
  preimage: Fr[];
  /** The inner note hash of the note. */
  innerNoteHash: Fr;
  /** The corresponding nullifier of the note. Undefined for pending notes. */
  siloedNullifier?: Fr;
  /** The note's leaf index in the private data tree. Undefined for pending notes. */
  index?: bigint;
}

/**
 * The partial data for L1 to L2 Messages provided by other data sources.
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
 * The data required by Aztec.nr to validate L1 to L2 Messages.
 */
export interface L1ToL2MessageOracleReturnData extends MessageLoadOracleInputs {
  /**
   * The current root of the l1 to l2 message tree.
   */
  root: Fr;
}

/**
 * Oracle with typed parameters and typed return values.
 * Methods that require read and/or write will have to be implemented based on the context (public, private, or view)
 * and are unavailable by default.
 */
export abstract class TypedOracle {
  computeSelector(signature: string): Fr {
    return FunctionSelector.fromSignature(signature).toField();
  }

  getRandomField(): Fr {
    return Fr.random();
  }

  packArguments(_args: Fr[]): Promise<Fr> {
    throw new Error('Not available.');
  }

  getSecretKey(_owner: PublicKey): Promise<GrumpkinScalar> {
    throw new Error('Not available.');
  }

  getPublicKey(_address: AztecAddress): Promise<CompleteAddress> {
    throw new Error('Not available.');
  }

  getAuthWitness(_messageHash: Fr): Promise<Fr[] | undefined> {
    throw new Error('Not available.');
  }

  getNotes(
    _storageSlot: Fr,
    _numSelects: number,
    _selectBy: number[],
    _selectValues: Fr[],
    _sortBy: number[],
    _sortOrder: number[],
    _limit: number,
    _offset: number,
  ): Promise<NoteData[]> {
    throw new Error('Not available.');
  }

  notifyCreatedNote(_storageSlot: Fr, _preimage: Fr[], _innerNoteHash: Fr): void {
    throw new Error('Not available.');
  }

  notifyNullifiedNote(_innerNullifier: Fr, _innerNoteHash: Fr): Promise<void> {
    throw new Error('Not available.');
  }

  getL1ToL2Message(_msgKey: Fr): Promise<L1ToL2MessageOracleReturnData> {
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

  emitEncryptedLog(_contractAddress: AztecAddress, _storageSlot: Fr, _publicKey: PublicKey, _preimage: Fr[]): void {
    throw new Error('Not available.');
  }

  emitUnencryptedLog(_log: UnencryptedL2Log): void {
    throw new Error('Not available.');
  }

  callPrivateFunction(
    _targetContractAddress: AztecAddress,
    _functionSelector: FunctionSelector,
    _argsHash: Fr,
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
  ): Promise<PublicCallRequest> {
    throw new Error('Not available.');
  }
}
