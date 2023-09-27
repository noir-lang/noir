import { CircuitsWasm, HistoricBlockData, PublicKey } from '@aztec/circuits.js';
import { computeUniqueCommitment, siloCommitment } from '@aztec/circuits.js/abis';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { AuthWitness, AztecNode, CompleteAddress } from '@aztec/types';

import { NoteData, TypedOracle } from '../acvm/index.js';
import { DBOracle } from './db_oracle.js';
import { pickNotes } from './pick_notes.js';

/**
 * The execution context for a client view tx simulation.
 * It only reads data from data sources. Nothing will be updated or created during this simulation.
 */
export class ViewDataOracle extends TypedOracle {
  constructor(
    protected readonly contractAddress: AztecAddress,
    /** Data required to reconstruct the block hash, it contains historic roots. */
    protected readonly historicBlockData: HistoricBlockData,
    /** List of transient auth witnesses to be used during this simulation */
    protected readonly authWitnesses: AuthWitness[],
    protected readonly db: DBOracle,
    protected readonly aztecNode: AztecNode | undefined,
    protected log = createDebugLogger('aztec:simulator:client_view_context'),
  ) {
    super();
  }

  /**
   * Return the secret key of a owner to use in a specific contract.
   * @param owner - The owner of the secret key.
   */
  public getSecretKey(owner: PublicKey) {
    return this.db.getSecretKey(this.contractAddress, owner);
  }

  /**
   * Retrieve the complete address associated to a given address.
   * @param address - Address to fetch the complete address for.
   * @returns A complete address associated with the input address.
   */
  public getPublicKey(address: AztecAddress): Promise<CompleteAddress> {
    return this.db.getCompleteAddress(address);
  }

  /**
   * Returns an auth witness for the given message hash. Checks on the list of transient witnesses
   * for this transaction first, and falls back to the local database if not found.
   * @param messageHash - Hash of the message to authenticate.
   * @returns Authentication witness for the requested message hash.
   */
  public getAuthWitness(messageHash: Fr): Promise<Fr[] | undefined> {
    return Promise.resolve(
      this.authWitnesses.find(w => w.requestHash.equals(messageHash))?.witness ?? this.db.getAuthWitness(messageHash),
    );
  }

  /**
   * Gets some notes for a contract address and storage slot.
   * Returns a flattened array containing real-note-count and note preimages.
   *
   * @remarks
   *
   * Check for pending notes with matching address/slot.
   * Real notes coming from DB will have a leafIndex which
   * represents their index in the private data tree.
   *
   * @param contractAddress - The contract address.
   * @param storageSlot - The storage slot.
   * @param numSelects - The number of valid selects in selectBy and selectValues.
   * @param selectBy - An array of indices of the fields to selects.
   * @param selectValues - The values to match.
   * @param sortBy - An array of indices of the fields to sort.
   * @param sortOrder - The order of the corresponding index in sortBy. (1: DESC, 2: ASC, 0: Do nothing)
   * @param limit - The number of notes to retrieve per query.
   * @param offset - The starting index for pagination.
   * @returns Flattened array of ACVMFields (format expected by Noir/ACVM) containing:
   * count - number of real (non-padding) notes retrieved,
   * contractAddress - the contract address,
   * preimages - the real note preimages retrieved, and
   * paddedZeros - zeros to ensure an array with length returnSize expected by Noir circuit
   */
  public async getNotes(
    storageSlot: Fr,
    numSelects: number,
    selectBy: number[],
    selectValues: Fr[],
    sortBy: number[],
    sortOrder: number[],
    limit: number,
    offset: number,
  ): Promise<NoteData[]> {
    const dbNotes = await this.db.getNotes(this.contractAddress, storageSlot);
    return pickNotes<NoteData>(dbNotes, {
      selects: selectBy.slice(0, numSelects).map((index, i) => ({ index, value: selectValues[i] })),
      sorts: sortBy.map((index, i) => ({ index, order: sortOrder[i] })),
      limit,
      offset,
    });
  }

  /**
   * Fetches a path to prove existence of a commitment in the db, given its contract side commitment (before silo).
   * @param nonce - The nonce of the note.
   * @param innerNoteHash - The inner note hash of the note.
   * @returns 1 if (persistent or transient) note hash exists, 0 otherwise. Value is in ACVMField form.
   */
  public async checkNoteHashExists(nonce: Fr, innerNoteHash: Fr): Promise<boolean> {
    // If nonce is zero, SHOULD only be able to reach this point if note was publicly created
    const wasm = await CircuitsWasm.get();
    let noteHashToLookUp = siloCommitment(wasm, this.contractAddress, innerNoteHash);
    if (!nonce.isZero()) {
      noteHashToLookUp = computeUniqueCommitment(wasm, nonce, noteHashToLookUp);
    }

    const index = await this.db.getCommitmentIndex(noteHashToLookUp);
    return index !== undefined;
  }

  /**
   * Fetches the a message from the db, given its key.
   * @param msgKey - A buffer representing the message key.
   * @returns The l1 to l2 message data
   */
  public async getL1ToL2Message(msgKey: Fr) {
    const message = await this.db.getL1ToL2Message(msgKey);
    return { ...message, root: this.historicBlockData.l1ToL2MessagesTreeRoot };
  }

  /**
   * Retrieves the portal contract address associated with the given contract address.
   * Throws an error if the input contract address is not found or invalid.
   * @param contractAddress - The address of the contract whose portal address is to be fetched.
   * @returns The portal contract address.
   */
  public getPortalContractAddress(contractAddress: AztecAddress) {
    return this.db.getPortalContractAddress(contractAddress);
  }

  /**
   * Read the public storage data.
   * @param startStorageSlot - The starting storage slot.
   * @param numberOfElements - Number of elements to read from the starting storage slot.
   */
  public async storageRead(startStorageSlot: Fr, numberOfElements: number) {
    if (!this.aztecNode) {
      throw new Error('Aztec node is undefined, cannot read storage.');
    }

    const values = [];
    for (let i = 0; i < Number(numberOfElements); i++) {
      const storageSlot = startStorageSlot.value + BigInt(i);
      const value = await this.aztecNode.getPublicStorageAt(this.contractAddress, storageSlot);
      if (value === undefined) {
        throw new Error(`Oracle storage read undefined: slot=${storageSlot.toString(16)}`);
      }

      const frValue = Fr.fromBuffer(value);
      this.log(`Oracle storage read: slot=${storageSlot.toString(16)} value=${frValue}`);
      values.push(frValue);
    }
    return values;
  }
}
