import { ContractData, LogType, PublicDataWrite, TxL2Logs } from '@aztec/circuit-types';
import {
  Fr,
  MAX_NEW_CONTRACTS_PER_TX,
  MAX_NEW_L2_TO_L1_MSGS_PER_TX,
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
} from '@aztec/circuits.js';
import { makeTuple } from '@aztec/foundation/array';
import { padArrayEnd } from '@aztec/foundation/collection';
import { sha256 } from '@aztec/foundation/crypto';
import { BufferReader, Tuple, serializeArrayOfBufferableToVector } from '@aztec/foundation/serialize';

export class TxEffect {
  constructor(
    /**
     * The note hashes to be inserted into the note hash tree.
     */
    public newNoteHashes: Tuple<Fr, typeof MAX_NEW_NOTE_HASHES_PER_TX>,
    /**
     * The nullifiers to be inserted into the nullifier tree.
     */
    public newNullifiers: Tuple<Fr, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    /**
     * The L2 to L1 messages to be inserted into the messagebox on L1.
     */
    public newL2ToL1Msgs: Tuple<Fr, typeof MAX_NEW_L2_TO_L1_MSGS_PER_TX>,
    /**
     * The public data writes to be inserted into the public data tree.
     */
    public newPublicDataWrites: Tuple<PublicDataWrite, typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
    /**
     * The leaves of the new contract data that will be inserted into the contracts tree.
     */
    public contractLeaves: Tuple<Fr, typeof MAX_NEW_CONTRACTS_PER_TX>,
    /**
     * The contract data of the new contracts.
     */
    public contractData: Tuple<ContractData, typeof MAX_NEW_CONTRACTS_PER_TX>,
    /**
     * The logs of the txEffect
     */
    public encryptedLogs: TxL2Logs,
    public unencryptedLogs: TxL2Logs,
  ) {}

  toBuffer(): Buffer {
    const nonZeroNoteHashes = this.newNoteHashes.filter(h => !h.isZero());
    const nonZeroNullifiers = this.newNullifiers.filter(h => !h.isZero());
    const nonZeroL2ToL1Msgs = this.newL2ToL1Msgs.filter(h => !h.isZero());
    const nonZeroPublicDataWrites = this.newPublicDataWrites.filter(h => !h.isEmpty());
    const nonZeroContractLeaves = this.contractLeaves.filter(h => !h.isZero());
    const nonZeroContractData = this.contractData.filter(h => !h.isEmpty());

    return Buffer.concat([
      serializeArrayOfBufferableToVector(nonZeroNoteHashes, 1),
      serializeArrayOfBufferableToVector(nonZeroNullifiers, 1),
      serializeArrayOfBufferableToVector(nonZeroL2ToL1Msgs, 1),
      serializeArrayOfBufferableToVector(nonZeroPublicDataWrites, 1),
      serializeArrayOfBufferableToVector(nonZeroContractLeaves, 1),
      // We don't prefix the contract data with the length because we already have that info before contract leaves
      ...nonZeroContractData.map(x => x.toBuffer()),
      this.encryptedLogs.toBuffer(),
      this.unencryptedLogs.toBuffer(),
    ]);
  }

  /**
   * Deserializes the L2Tx object from a Buffer.
   * @param buffer - Buffer or BufferReader object to deserialize.
   * @returns An instance of L2Tx.
   */
  static fromBuffer(buffer: Buffer | BufferReader): TxEffect {
    const reader = BufferReader.asReader(buffer);

    const nonZeroNoteHashes = reader.readVectorUint8Prefix(Fr);
    const nonZeroNullifiers = reader.readVectorUint8Prefix(Fr);
    const nonZeroL2ToL1Msgs = reader.readVectorUint8Prefix(Fr);
    const nonZeroPublicDataWrites = reader.readVectorUint8Prefix(PublicDataWrite);

    const nonZeroContractLeaves = reader.readVectorUint8Prefix(Fr);

    const numContracts = nonZeroContractLeaves.length;
    const nonZeroContractData = reader.readArray(numContracts, ContractData);

    return new TxEffect(
      padArrayEnd(nonZeroNoteHashes, Fr.ZERO, MAX_NEW_NOTE_HASHES_PER_TX),
      padArrayEnd(nonZeroNullifiers, Fr.ZERO, MAX_NEW_NULLIFIERS_PER_TX),
      padArrayEnd(nonZeroL2ToL1Msgs, Fr.ZERO, MAX_NEW_L2_TO_L1_MSGS_PER_TX),
      padArrayEnd(nonZeroPublicDataWrites, PublicDataWrite.empty(), MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX),
      padArrayEnd(nonZeroContractLeaves, Fr.ZERO, MAX_NEW_CONTRACTS_PER_TX),
      padArrayEnd(nonZeroContractData, ContractData.empty(), MAX_NEW_CONTRACTS_PER_TX),
      TxL2Logs.fromBuffer(reader),
      TxL2Logs.fromBuffer(reader),
    );
  }

  hash() {
    const noteHashesBuffer = Buffer.concat(this.newNoteHashes.map(x => x.toBuffer()));
    const nullifiersBuffer = Buffer.concat(this.newNullifiers.map(x => x.toBuffer()));
    const newL2ToL1MsgsBuffer = Buffer.concat(this.newL2ToL1Msgs.map(x => x.toBuffer()));
    const publicDataUpdateRequestsBuffer = Buffer.concat(this.newPublicDataWrites.map(x => x.toBuffer()));
    const encryptedLogsHashKernel0 = this.encryptedLogs.hash();
    const unencryptedLogsHashKernel0 = this.unencryptedLogs.hash();

    if (MAX_NEW_CONTRACTS_PER_TX !== 1) {
      throw new Error('Only one contract per transaction is supported for now.');
    }

    const inputValue = Buffer.concat([
      noteHashesBuffer,
      nullifiersBuffer,
      newL2ToL1MsgsBuffer,
      publicDataUpdateRequestsBuffer,
      this.contractLeaves[0].toBuffer(),
      this.contractData[0].contractAddress.toBuffer(),
      // TODO(#3938): make portal address 20 bytes here when updating the hashing
      this.contractData[0].portalContractAddress.toBuffer32(),
      encryptedLogsHashKernel0,
      unencryptedLogsHashKernel0,
    ]);

    return sha256(inputValue);
  }

  static random(
    numPrivateCallsPerTx = 2,
    numPublicCallsPerTx = 3,
    numEncryptedLogsPerCall = 2,
    numUnencryptedLogsPerCall = 1,
  ): TxEffect {
    return new TxEffect(
      makeTuple(MAX_NEW_NOTE_HASHES_PER_TX, Fr.random),
      makeTuple(MAX_NEW_NULLIFIERS_PER_TX, Fr.random),
      makeTuple(MAX_NEW_L2_TO_L1_MSGS_PER_TX, Fr.random),
      makeTuple(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataWrite.random),
      makeTuple(MAX_NEW_CONTRACTS_PER_TX, Fr.random),
      makeTuple(MAX_NEW_CONTRACTS_PER_TX, ContractData.random),
      TxL2Logs.random(numPrivateCallsPerTx, numEncryptedLogsPerCall, LogType.ENCRYPTED),
      TxL2Logs.random(numPublicCallsPerTx, numUnencryptedLogsPerCall, LogType.UNENCRYPTED),
    );
  }
}
