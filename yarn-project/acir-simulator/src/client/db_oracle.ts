import { PartialContractAddress, PublicKey } from '@aztec/circuits.js';
import { FunctionAbi } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';

import { CommitmentsDB } from '../index.js';

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
  /** The note's leaf index in the private data tree. Undefined for pending notes. */
  index?: bigint;
}

/**
 * The format that noir uses to get L1 to L2 Messages.
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
 * The format noir uses to get commitments.
 */
export interface CommitmentDataOracleInputs {
  /** The siloed commitment. */
  commitment: Fr;
  /**
   * The path in the merkle tree to the commitment.
   */
  siblingPath: Fr[];
  /**
   * The index of the message commitment in the merkle tree.
   */
  index: bigint;
}

/**
 * The database oracle interface.
 */
export interface DBOracle extends CommitmentsDB {
  getPublicKey(address: AztecAddress): Promise<[PublicKey, PartialContractAddress]>;
  getSecretKey(contractAddress: AztecAddress, pubKey: PublicKey): Promise<Buffer>;
  getNotes(contractAddress: AztecAddress, storageSlot: Fr): Promise<NoteData[]>;
  getFunctionABI(contractAddress: AztecAddress, functionSelector: Buffer): Promise<FunctionAbi>;
  getPortalContractAddress(contractAddress: AztecAddress): Promise<EthAddress>;
}
