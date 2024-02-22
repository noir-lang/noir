import {
  MAX_NEW_CONTRACTS_PER_TX,
  MAX_NEW_L2_TO_L1_MSGS_PER_TX,
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  Vector,
} from '@aztec/circuits.js';
import { times } from '@aztec/foundation/collection';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, numToUInt32BE } from '@aztec/foundation/serialize';

import { ContractData } from './contract_data.js';
import { PublicDataWrite } from './public_data_write.js';
import { TxHash } from './tx/tx_hash.js';

/**
 * The string encoding used for serializing L2Tx objects to strings.
 */
const STRING_ENCODING: BufferEncoding = 'hex';

/**
 * Represents an L2 transaction.
 */
export class L2Tx {
  /**
   * The transaction's hash.
   * Note: It's the first nullifier emitted by the kernel circuit.
   */
  public readonly txHash: TxHash;

  constructor(
    /**
     * New note hashes created by the transaction.
     */
    public newNoteHashes: Fr[],
    /**
     * New nullifiers created by the transaction.
     */
    public newNullifiers: Fr[],
    /**
     * New public data writes created by the transaction.
     */
    public newPublicDataWrites: PublicDataWrite[],
    /**
     * New L2 to L1 messages created by the transaction.
     */
    public newL2ToL1Msgs: Fr[],
    /**
     * New contracts leaves created by the transaction to be inserted into the contract tree.
     */
    public newContracts: Fr[],
    /**
     * New contract data created by the transaction.
     */
    public newContractData: ContractData[],
    /**
     * The unique identifier of the block containing the transaction.
     */
    public blockHash: Fr,
    /**
     * The block number in which the transaction was included.
     */
    public blockNumber: number,
  ) {
    this.txHash = new TxHash(this.newNullifiers[0].toBuffer());
  }

  /**
   * Deserializes the L2Tx object from a Buffer.
   * @param buffer - Buffer or BufferReader object to deserialize.
   * @returns An instance of L2Tx.
   */
  static fromBuffer(buffer: Buffer | BufferReader): L2Tx {
    const reader = BufferReader.asReader(buffer);
    return new L2Tx(
      reader.readVector(Fr),
      reader.readVector(Fr),
      reader.readVector(PublicDataWrite),
      reader.readVector(Fr),
      reader.readVector(Fr),
      reader.readVector(ContractData),
      Fr.fromBuffer(reader),
      reader.readNumber(),
    );
  }

  /**
   * Deserializes an L2Tx object from a string.
   * @param str - String to deserialize.
   * @returns An instance of L2Tx.
   */
  static fromString(str: string) {
    return L2Tx.fromBuffer(Buffer.from(str, STRING_ENCODING));
  }

  /**
   * Serializes the Tx object into a Buffer.
   * @returns Buffer representation of the Tx object.
   */
  toBuffer() {
    return Buffer.concat([
      new Vector(this.newNoteHashes).toBuffer(),
      new Vector(this.newNullifiers).toBuffer(),
      new Vector(this.newPublicDataWrites).toBuffer(),
      new Vector(this.newL2ToL1Msgs).toBuffer(),
      new Vector(this.newContracts).toBuffer(),
      new Vector(this.newContractData).toBuffer(),
      this.blockHash.toBuffer(),
      numToUInt32BE(this.blockNumber),
    ]);
  }

  /**
   * Returns a string representation of the L2Tx object.
   */
  toString(): string {
    return this.toBuffer().toString(STRING_ENCODING);
  }

  static random() {
    const rand = (min: number, max: number) => Math.floor(Math.random() * max) + min;
    return new L2Tx(
      times(rand(0, MAX_NEW_NOTE_HASHES_PER_TX), Fr.random),
      times(rand(1, MAX_NEW_NULLIFIERS_PER_TX), Fr.random),
      times(rand(0, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX), PublicDataWrite.random),
      times(rand(0, MAX_NEW_L2_TO_L1_MSGS_PER_TX), Fr.random),
      times(rand(0, MAX_NEW_CONTRACTS_PER_TX), Fr.random),
      times(rand(0, MAX_NEW_CONTRACTS_PER_TX), ContractData.random),
      Fr.random(),
      123,
    );
  }
}
