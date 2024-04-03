import { EncryptedTxL2Logs, PublicDataWrite, TxHash, UnencryptedTxL2Logs } from '@aztec/circuit-types';
import {
  Fr,
  MAX_NEW_L2_TO_L1_MSGS_PER_TX,
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  RevertCode,
} from '@aztec/circuits.js';
import { makeTuple } from '@aztec/foundation/array';
import { sha256 } from '@aztec/foundation/crypto';
import {
  BufferReader,
  serializeArrayOfBufferableToVector,
  serializeToBuffer,
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
    public noteHashes: Fr[],
    /**
     * The nullifiers to be inserted into the nullifier tree.
     */
    public nullifiers: Fr[],
    /**
     * The L2 to L1 messages to be inserted into the messagebox on L1.
     */
    public l2ToL1Msgs: Fr[],
    /**
     * The public data writes to be inserted into the public data tree.
     */
    public publicDataWrites: PublicDataWrite[],
    /**
     * The logs of the txEffect
     */
    public encryptedLogs: EncryptedTxL2Logs,
    public unencryptedLogs: UnencryptedTxL2Logs,
  ) {
    // TODO(#4638): Clean this up once we have isDefault() everywhere --> then we don't have to deal with 2 different
    // functions (isZero and isEmpty)
    if (noteHashes.length > MAX_NEW_NOTE_HASHES_PER_TX) {
      throw new Error(`Too many note hashes: ${noteHashes.length}, max: ${MAX_NEW_NOTE_HASHES_PER_TX}`);
    }
    noteHashes.forEach(h => {
      if (h.isZero()) {
        throw new Error('Note hash is zero');
      }
    });

    if (nullifiers.length > MAX_NEW_NULLIFIERS_PER_TX) {
      throw new Error(`Too many nullifiers: ${nullifiers.length}, max: ${MAX_NEW_NULLIFIERS_PER_TX}`);
    }
    nullifiers.forEach(h => {
      if (h.isZero()) {
        throw new Error('Nullifier is zero');
      }
    });

    if (l2ToL1Msgs.length > MAX_NEW_L2_TO_L1_MSGS_PER_TX) {
      throw new Error(`Too many L2 to L1 messages: ${l2ToL1Msgs.length}, max: ${MAX_NEW_L2_TO_L1_MSGS_PER_TX}`);
    }
    l2ToL1Msgs.forEach(h => {
      if (h.isZero()) {
        throw new Error('L2 to L1 message is zero');
      }
    });

    if (publicDataWrites.length > MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX) {
      throw new Error(
        `Too many public data writes: ${publicDataWrites.length}, max: ${MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX}`,
      );
    }
    publicDataWrites.forEach(h => {
      if (h.isEmpty()) {
        throw new Error('Public data write is empty');
      }
    });
  }

  toBuffer(): Buffer {
    return serializeToBuffer([
      this.revertCode,
      serializeArrayOfBufferableToVector(this.noteHashes, 1),
      serializeArrayOfBufferableToVector(this.nullifiers, 1),
      serializeArrayOfBufferableToVector(this.l2ToL1Msgs, 1),
      serializeArrayOfBufferableToVector(this.publicDataWrites, 1),
      this.encryptedLogs,
      this.unencryptedLogs,
    ]);
  }

  /**
   * Deserializes the TxEffect object from a Buffer.
   * @param buffer - Buffer or BufferReader object to deserialize.
   * @returns An instance of TxEffect.
   */
  static fromBuffer(buffer: Buffer | BufferReader): TxEffect {
    const reader = BufferReader.asReader(buffer);

    return new TxEffect(
      RevertCode.fromBuffer(reader),
      reader.readVectorUint8Prefix(Fr),
      reader.readVectorUint8Prefix(Fr),
      reader.readVectorUint8Prefix(Fr),
      reader.readVectorUint8Prefix(PublicDataWrite),
      reader.readObject(EncryptedTxL2Logs),
      reader.readObject(UnencryptedTxL2Logs),
    );
  }

  /**
   * Computes the hash of the TxEffect object.
   * @returns The hash of the TxEffect object.
   * @dev This function must correspond with compute_tx_effects_hash() in Noir and TxsDecoder.sol decode().
   */
  hash() {
    const padBuffer = (buf: Buffer, length: number) => Buffer.concat([buf, Buffer.alloc(length - buf.length)]);

    const noteHashesBuffer = padBuffer(
      serializeToBuffer(this.noteHashes),
      Fr.SIZE_IN_BYTES * MAX_NEW_NOTE_HASHES_PER_TX,
    );
    const nullifiersBuffer = padBuffer(
      serializeToBuffer(this.nullifiers),
      Fr.SIZE_IN_BYTES * MAX_NEW_NULLIFIERS_PER_TX,
    );
    const l2ToL1MsgsBuffer = padBuffer(
      serializeToBuffer(this.l2ToL1Msgs),
      Fr.SIZE_IN_BYTES * MAX_NEW_L2_TO_L1_MSGS_PER_TX,
    );
    const publicDataWritesBuffer = padBuffer(
      serializeToBuffer(this.publicDataWrites),
      PublicDataWrite.SIZE_IN_BYTES * MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
    );

    const encryptedLogsHashKernel0 = this.encryptedLogs.hash();
    const unencryptedLogsHashKernel0 = this.unencryptedLogs.hash();

    const inputValue = Buffer.concat([
      this.revertCode.toHashPreimage(),
      noteHashesBuffer,
      nullifiersBuffer,
      l2ToL1MsgsBuffer,
      publicDataWritesBuffer,
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
      EncryptedTxL2Logs.random(numPrivateCallsPerTx, numEncryptedLogsPerCall),
      UnencryptedTxL2Logs.random(numPublicCallsPerTx, numUnencryptedLogsPerCall),
    );
  }

  static empty(): TxEffect {
    return new TxEffect(RevertCode.OK, [], [], [], [], EncryptedTxL2Logs.empty(), UnencryptedTxL2Logs.empty());
  }

  isEmpty(): boolean {
    return this.nullifiers.length === 0;
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
