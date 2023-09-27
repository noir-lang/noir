import { EthAddress, FunctionSelector } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';

import { MessageLoadOracleInputs } from '../acvm/index.js';

/**
 * Database interface for providing access to public state.
 */
export interface PublicStateDB {
  /**
   * Reads a value from public storage, returning zero if none.
   * @param contract - Owner of the storage.
   * @param slot - Slot to read in the contract storage.
   * @returns The current value in the storage slot.
   */
  storageRead(contract: AztecAddress, slot: Fr): Promise<Fr>;

  /**
   * Records a write to public storage.
   * @param contract - Owner of the storage.
   * @param slot - Slot to read in the contract storage.
   * @param newValue - The new value to store.
   * @returns Nothing.
   */
  storageWrite(contract: AztecAddress, slot: Fr, newValue: Fr): Promise<void>;
}

/**
 * Database interface for providing access to public contract data.
 */
export interface PublicContractsDB {
  /**
   * Returns the brillig (public bytecode) of a function.
   * @param address - The contract address that owns this function.
   * @param selector - The selector for the function.
   * @returns The bytecode or undefined if not found.
   */
  getBytecode(address: AztecAddress, selector: FunctionSelector): Promise<Buffer | undefined>;

  /**
   * Returns whether a function is internal or not.
   * @param address - The contract address that owns this function.
   * @param selector - The selector for the function.
   * @returns The `isInternal` flag found, undefined if not found.
   */
  getIsInternal(address: AztecAddress, selector: FunctionSelector): Promise<boolean | undefined>;

  /**
   * Returns the portal contract address for an L2 address.
   * @param address - The L2 contract address.
   * @returns The portal contract address or undefined if not found.
   */
  getPortalContractAddress(address: AztecAddress): Promise<EthAddress | undefined>;
}

/** Database interface for providing access to commitment tree and l1 to l2 messages tree (append only data trees). */
export interface CommitmentsDB {
  /**
   * Gets a confirmed L1 to L2 message for the given message key.
   * TODO(Maddiaa): Can be combined with aztec-node method that does the same thing.
   * @param msgKey - The message Key.
   * @returns - The l1 to l2 message object
   */
  getL1ToL2Message(msgKey: Fr): Promise<MessageLoadOracleInputs>;

  /**
   * Gets the index of a commitment in the private data tree.
   * @param commitment - The commitment.
   * @returns - The index of the commitment. Undefined if it does not exist in the tree.
   */
  getCommitmentIndex(commitment: Fr): Promise<bigint | undefined>;
}
