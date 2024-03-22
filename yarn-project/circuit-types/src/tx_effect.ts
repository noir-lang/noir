import { LogType, PublicDataWrite, TxHash, TxL2Logs } from '@aztec/circuit-types';
import {
  Fr,
  MAX_NEW_L2_TO_L1_MSGS_PER_TX,
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  RevertCode,
} from '@aztec/circuits.js';
import { assertRightPadded, makeTuple } from '@aztec/foundation/array';
import { padArrayEnd } from '@aztec/foundation/collection';
import { sha256 } from '@aztec/foundation/crypto';
import {
  BufferReader,
  Tuple,
  assertLength,
  serializeArrayOfBufferableToVector,
  truncateAndPad,
} from '@aztec/foundation/serialize';

import { inspect } from 'util';

export class TxEffect {
  constructor(
    /**
     * Whether the transaction reverted during public app logic.
     */
    public revertCode: RevertCode,
    /**
     * The note hashes to be inserted into the note hash tree.
     */
    public noteHashes: Tuple<Fr, typeof MAX_NEW_NOTE_HASHES_PER_TX>,
    /**
     * The nullifiers to be inserted into the nullifier tree.
     */
    public nullifiers: Tuple<Fr, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    /**
     * The L2 to L1 messages to be inserted into the messagebox on L1.
     */
    public l2ToL1Msgs: Tuple<Fr, typeof MAX_NEW_L2_TO_L1_MSGS_PER_TX>,
    /**
     * The public data writes to be inserted into the public data tree.
     */
    public publicDataWrites: Tuple<PublicDataWrite, typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
    /**
     * The logs of the txEffect
     */
    public encryptedLogs: TxL2Logs,
    public unencryptedLogs: TxL2Logs,
  ) {}

  toBuffer(): Buffer {
    const nonZeroNoteHashes = this.noteHashes.filter(h => !h.isZero());
    const nonZeroNullifiers = this.nullifiers.filter(h => !h.isZero());
    const nonZeroL2ToL1Msgs = this.l2ToL1Msgs.filter(h => !h.isZero());
    const nonZeroPublicDataWrites = this.publicDataWrites.filter(h => !h.isEmpty());

    return Buffer.concat([
      this.revertCode.toBuffer(),
      serializeArrayOfBufferableToVector(nonZeroNoteHashes, 1),
      serializeArrayOfBufferableToVector(nonZeroNullifiers, 1),
      serializeArrayOfBufferableToVector(nonZeroL2ToL1Msgs, 1),
      serializeArrayOfBufferableToVector(nonZeroPublicDataWrites, 1),
      this.encryptedLogs.toBuffer(),
      this.unencryptedLogs.toBuffer(),
    ]);
  }

  /**
   * Deserializes the TxEffect object from a Buffer.
   * @param buffer - Buffer or BufferReader object to deserialize.
   * @returns An instance of TxEffect.
   */
  static fromBuffer(buffer: Buffer | BufferReader): TxEffect {
    const reader = BufferReader.asReader(buffer);

    const revertCode = RevertCode.fromBuffer(reader);
    const nonZeroNoteHashes = reader.readVectorUint8Prefix(Fr);
    const nonZeroNullifiers = reader.readVectorUint8Prefix(Fr);
    const nonZeroL2ToL1Msgs = reader.readVectorUint8Prefix(Fr);
    const nonZeroPublicDataWrites = reader.readVectorUint8Prefix(PublicDataWrite);

    return new TxEffect(
      revertCode,
      padArrayEnd(nonZeroNoteHashes, Fr.ZERO, MAX_NEW_NOTE_HASHES_PER_TX),
      padArrayEnd(nonZeroNullifiers, Fr.ZERO, MAX_NEW_NULLIFIERS_PER_TX),
      padArrayEnd(nonZeroL2ToL1Msgs, Fr.ZERO, MAX_NEW_L2_TO_L1_MSGS_PER_TX),
      padArrayEnd(nonZeroPublicDataWrites, PublicDataWrite.empty(), MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX),
      TxL2Logs.fromBuffer(reader),
      TxL2Logs.fromBuffer(reader),
    );
  }

  hash() {
    // must correspond with compute_tx_effects_hash() in nr
    // and TxsDecoder.sol decode()
    assertLength(this.noteHashes, MAX_NEW_NOTE_HASHES_PER_TX);
    assertRightPadded(this.noteHashes, Fr.isZero);
    const noteHashesBuffer = Buffer.concat(this.noteHashes.map(x => x.toBuffer()));

    assertLength(this.nullifiers, MAX_NEW_NULLIFIERS_PER_TX);
    assertRightPadded(this.nullifiers, Fr.isZero);
    const nullifiersBuffer = Buffer.concat(this.nullifiers.map(x => x.toBuffer()));

    assertLength(this.l2ToL1Msgs, MAX_NEW_L2_TO_L1_MSGS_PER_TX);
    assertRightPadded(this.l2ToL1Msgs, Fr.isZero);
    const newL2ToL1MsgsBuffer = Buffer.concat(this.l2ToL1Msgs.map(x => x.toBuffer()));

    assertLength(this.publicDataWrites, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX);
    assertRightPadded(this.publicDataWrites, PublicDataWrite.isEmpty);
    const publicDataUpdateRequestsBuffer = Buffer.concat(this.publicDataWrites.map(x => x.toBuffer()));
    const encryptedLogsHashKernel0 = this.encryptedLogs.hash();
    const unencryptedLogsHashKernel0 = this.unencryptedLogs.hash();

    const inputValue = Buffer.concat([
      this.revertCode.toHashPreimage(),
      noteHashesBuffer,
      nullifiersBuffer,
      newL2ToL1MsgsBuffer,
      publicDataUpdateRequestsBuffer,
      encryptedLogsHashKernel0,
      unencryptedLogsHashKernel0,
    ]);

    return truncateAndPad(sha256(inputValue));
  }

  static random(
    numPrivateCallsPerTx = 2,
    numPublicCallsPerTx = 3,
    numEncryptedLogsPerCall = 2,
    numUnencryptedLogsPerCall = 1,
  ): TxEffect {
    return new TxEffect(
      RevertCode.random(),
      makeTuple(MAX_NEW_NOTE_HASHES_PER_TX, Fr.random),
      makeTuple(MAX_NEW_NULLIFIERS_PER_TX, Fr.random),
      makeTuple(MAX_NEW_L2_TO_L1_MSGS_PER_TX, Fr.random),
      makeTuple(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataWrite.random),
      TxL2Logs.random(numPrivateCallsPerTx, numEncryptedLogsPerCall, LogType.ENCRYPTED),
      TxL2Logs.random(numPublicCallsPerTx, numUnencryptedLogsPerCall, LogType.UNENCRYPTED),
    );
  }

  static empty(): TxEffect {
    return new TxEffect(
      RevertCode.OK,
      makeTuple(MAX_NEW_NOTE_HASHES_PER_TX, Fr.zero),
      makeTuple(MAX_NEW_NULLIFIERS_PER_TX, Fr.zero),
      makeTuple(MAX_NEW_L2_TO_L1_MSGS_PER_TX, Fr.zero),
      makeTuple(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataWrite.empty),
      TxL2Logs.empty(),
      TxL2Logs.empty(),
    );
  }

  /**
   * Returns a string representation of the TxEffect object.
   */
  toString(): string {
    return this.toBuffer().toString('hex');
  }

  [inspect.custom]() {
    // print out the non-empty fields

    return `TxEffect { 
      revertCode: ${this.revertCode},
      note hashes: [${this.noteHashes.map(h => h.toString()).join(', ')}],
      nullifiers: [${this.nullifiers.map(h => h.toString()).join(', ')}],
      l2ToL1Msgs: [${this.l2ToL1Msgs.map(h => h.toString()).join(', ')}],
      publicDataWrites: [${this.publicDataWrites.map(h => h.toString()).join(', ')}],
      encryptedLogs: ${JSON.stringify(this.encryptedLogs.toJSON())},
      unencryptedLogs: ${JSON.stringify(this.unencryptedLogs.toJSON())}
     }`;
  }

  /**
   * Deserializes an TxEffect object from a string.
   * @param str - String to deserialize.
   * @returns An instance of TxEffect.
   */
  static fromString(str: string) {
    return TxEffect.fromBuffer(Buffer.from(str, 'hex'));
  }

  get txHash(): TxHash {
    return new TxHash(this.nullifiers[0].toBuffer());
  }
}
