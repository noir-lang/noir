import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { FunctionAbi } from '@aztec/noir-contracts';

/**
 * The format that noir contracts use to get notes.
 */
export interface NoteLoadOracleInputs {
  /**
   * The preimage of the note.
   */
  preimage: Fr[];
  /**
   * The path in the merkle tree to the note.
   */
  siblingPath: Fr[];
  /**
   * The index of the note in the merkle tree.
   */
  index: bigint;
}

/**
 * The database oracle interface.
 */
export interface DBOracle {
  getSecretKey(contractAddress: AztecAddress, address: AztecAddress): Promise<Buffer>;
  getNotes(
    contractAddress: AztecAddress,
    storageSlot: Fr,
    limit: number,
    offset: number,
  ): Promise<{
    /** How many notes actually returned. */
    count: number;
    /** The notes. */
    notes: NoteLoadOracleInputs[];
  }>;
  getFunctionABI(contractAddress: AztecAddress, functionSelector: Buffer): Promise<FunctionAbi>;
  getPortalContractAddress(contractAddress: AztecAddress): Promise<EthAddress>;
}
