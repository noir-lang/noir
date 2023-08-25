import {
  MAX_NEW_COMMITMENTS_PER_TX,
  MAX_NEW_CONTRACTS_PER_TX,
  MAX_NEW_L2_TO_L1_MSGS_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
} from '@aztec/circuits.js';
import { serializeToBuffer } from '@aztec/circuits.js/utils';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, numToUInt32BE } from '@aztec/foundation/serialize';

import times from 'lodash.times';

import { ContractData } from './contract_data.js';
import { PublicDataWrite } from './public_data_write.js';
import { TxHash } from './tx/tx_hash.js';

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
     * New commitments created by the transaction.
     */
    public newCommitments: Fr[],
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
     * New contracts leafs created by the transaction to be inserted into the contract tree.
     */
    public newContracts: Fr[],
    /**
     * New contract data created by the transaction.
     */
    public newContractData: ContractData[],
    /**
     * The unique identifier of the block containing the transaction.
     */
    public blockHash: Buffer,
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
      reader.readArray(MAX_NEW_COMMITMENTS_PER_TX, Fr),
      reader.readArray(MAX_NEW_NULLIFIERS_PER_TX, Fr),
      reader.readArray(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataWrite),
      reader.readArray(MAX_NEW_L2_TO_L1_MSGS_PER_TX, Fr),
      reader.readArray(MAX_NEW_CONTRACTS_PER_TX, Fr),
      reader.readArray(MAX_NEW_CONTRACTS_PER_TX, ContractData),
      reader.readBytes(32),
      reader.readNumber(),
    );
  }

  /**
   * Serializes the Tx object into a Buffer.
   * @returns Buffer representation of the Tx object.
   */
  toBuffer() {
    return serializeToBuffer([
      this.newCommitments,
      this.newNullifiers,
      this.newPublicDataWrites,
      this.newL2ToL1Msgs,
      this.newContracts,
      this.newContractData,
      this.blockHash,
      numToUInt32BE(this.blockNumber),
    ]);
  }

  static random() {
    return new L2Tx(
      times(MAX_NEW_COMMITMENTS_PER_TX, Fr.random),
      times(MAX_NEW_NULLIFIERS_PER_TX, Fr.random),
      times(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataWrite.random),
      times(MAX_NEW_L2_TO_L1_MSGS_PER_TX, Fr.random),
      times(MAX_NEW_CONTRACTS_PER_TX, Fr.random),
      times(MAX_NEW_CONTRACTS_PER_TX, ContractData.random),
      Fr.random().toBuffer(),
      123,
    );
  }
}
