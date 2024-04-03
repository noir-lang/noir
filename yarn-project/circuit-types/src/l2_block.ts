import { Body } from '@aztec/circuit-types';
import { AppendOnlyTreeSnapshot, Header, STRING_ENCODING } from '@aztec/circuits.js';
import { sha256, sha256ToField } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { makeAppendOnlyTreeSnapshot, makeHeader } from './l2_block_code_to_purge.js';

/**
 * The data that makes up the rollup proof, with encoder decoder functions.
 */
export class L2Block {
  constructor(
    /** Snapshot of archive tree after the block is applied. */
    public archive: AppendOnlyTreeSnapshot,
    /** L2 block header. */
    public header: Header,
    /** L2 block body. */
    public body: Body,
  ) {}

  /**
   * Constructs a new instance from named fields.
   * @param fields - Fields to pass to the constructor.
   * @param blockHash - Hash of the block.
   * @returns A new instance.
   */
  static fromFields(fields: {
    /** Snapshot of archive tree after the block is applied. */
    archive: AppendOnlyTreeSnapshot;
    /** L2 block header. */
    header: Header;
    body: Body;
  }) {
    return new this(fields.archive, fields.header, fields.body);
  }

  /**
   * Deserializes a block from a buffer
   * @returns A deserialized L2 block.
   */
  static fromBuffer(buf: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buf);
    const header = reader.readObject(Header);
    const archive = reader.readObject(AppendOnlyTreeSnapshot);
    const body = reader.readObject(Body);

    return L2Block.fromFields({
      archive,
      header,
      body,
    });
  }

  /**
   * Serializes a block
   * @returns A serialized L2 block as a Buffer.
   */
  toBuffer() {
    return serializeToBuffer(this.header, this.archive, this.body);
  }

  /**
   * Deserializes L2 block from a buffer.
   * @param str - A serialized L2 block.
   * @returns Deserialized L2 block.
   */
  static fromString(str: string): L2Block {
    return L2Block.fromBuffer(Buffer.from(str, STRING_ENCODING));
  }

  /**
   * Serializes a block to a string.
   * @returns A serialized L2 block as a string.
   */
  toString(): string {
    return this.toBuffer().toString(STRING_ENCODING);
  }

  /**
   * Creates an L2 block containing random data.
   * @param l2BlockNum - The number of the L2 block.
   * @param txsPerBlock - The number of transactions to include in the block.
   * @param numPrivateCallsPerTx - The number of private function calls to include in each transaction.
   * @param numPublicCallsPerTx - The number of public function calls to include in each transaction.
   * @param numEncryptedLogsPerCall - The number of encrypted logs per 1 private function invocation.
   * @param numUnencryptedLogsPerCall - The number of unencrypted logs per 1 public function invocation.
   * @param inHash - The hash of the L1 to L2 messages subtree which got inserted in this block.
   * @returns The L2 block.
   */
  static random(
    l2BlockNum: number,
    txsPerBlock = 4,
    numPrivateCallsPerTx = 2,
    numPublicCallsPerTx = 3,
    numEncryptedLogsPerCall = 2,
    numUnencryptedLogsPerCall = 1,
    inHash: Buffer | undefined = undefined,
  ): L2Block {
    const body = Body.random(
      txsPerBlock,
      numPrivateCallsPerTx,
      numPublicCallsPerTx,
      numEncryptedLogsPerCall,
      numUnencryptedLogsPerCall,
    );

    const txsEffectsHash = body.getTxsEffectsHash();

    return L2Block.fromFields({
      archive: makeAppendOnlyTreeSnapshot(1),
      header: makeHeader(0, l2BlockNum, txsEffectsHash, inHash),
      body,
    });
  }

  /**
   * Creates an L2 block containing empty data.
   * @returns The L2 block.
   */
  static empty(): L2Block {
    return L2Block.fromFields({
      archive: AppendOnlyTreeSnapshot.zero(),
      header: Header.empty(),
      body: Body.empty(),
    });
  }

  get number(): number {
    return Number(this.header.globalVariables.blockNumber.toBigInt());
  }

  /**
   * Returns the block's hash (hash of block header).
   * @returns The block's hash.
   */
  public hash(): Fr {
    return this.header.hash();
  }

  /**
   * Computes the public inputs hash for the L2 block.
   * The same output as the hash of RootRollupPublicInputs.
   * @returns The public input hash for the L2 block as a field element.
   */
  // TODO(#4844)
  getPublicInputsHash(): Fr {
    const preimage = [
      this.header.globalVariables,
      AppendOnlyTreeSnapshot.zero(), // this.startNoteHashTreeSnapshot / commitments,
      AppendOnlyTreeSnapshot.zero(), // this.startNullifierTreeSnapshot,
      AppendOnlyTreeSnapshot.zero(), // this.startPublicDataTreeSnapshot,
      AppendOnlyTreeSnapshot.zero(), // this.startL1ToL2MessageTreeSnapshot,
      this.header.lastArchive,
      this.header.state.partial.noteHashTree,
      this.header.state.partial.nullifierTree,
      this.header.state.partial.publicDataTree,
      this.header.state.l1ToL2MessageTree,
      this.archive,
      this.body.getTxsEffectsHash(),
    ];

    return sha256ToField(preimage);
  }

  /**
   * Computes the start state hash (should equal contract data before block).
   * @returns The start state hash for the L2 block.
   */
  // TODO(#4844)
  getStartStateHash() {
    const inputValue = serializeToBuffer(
      new Fr(Number(this.header.globalVariables.blockNumber.toBigInt()) - 1),
      AppendOnlyTreeSnapshot.zero(), // this.startNoteHashTreeSnapshot,
      AppendOnlyTreeSnapshot.zero(), // this.startNullifierTreeSnapshot,
      AppendOnlyTreeSnapshot.zero(), // this.startPublicDataTreeSnapshot,
      AppendOnlyTreeSnapshot.zero(), // this.startL1ToL2MessageTreeSnapshot,
      this.header.lastArchive,
    );
    return sha256(inputValue);
  }

  /**
   * Computes the end state hash (should equal contract data after block).
   * @returns The end state hash for the L2 block.
   */
  // TODO(#4844)
  getEndStateHash() {
    const inputValue = serializeToBuffer(
      this.header.globalVariables.blockNumber,
      this.header.state.partial.noteHashTree,
      this.header.state.partial.nullifierTree,
      this.header.state.partial.publicDataTree,
      this.header.state.l1ToL2MessageTree,
      this.archive,
    );
    return sha256(inputValue);
  }

  /**
   * Returns stats used for logging.
   * @returns Stats on tx count, number, and log size and count.
   */
  getStats() {
    const logsStats = {
      encryptedLogLength: this.body.txEffects.reduce(
        (logCount, txEffect) => logCount + txEffect.encryptedLogs.getSerializedLength(),
        0,
      ),
      encryptedLogCount: this.body.txEffects.reduce(
        (logCount, txEffect) => logCount + txEffect.encryptedLogs.getTotalLogCount(),
        0,
      ),
      unencryptedLogCount: this.body.txEffects.reduce(
        (logCount, txEffect) => logCount + txEffect.unencryptedLogs.getSerializedLength(),
        0,
      ),
      unencryptedLogSize: this.body.txEffects.reduce(
        (logCount, txEffect) => logCount + txEffect.unencryptedLogs.getTotalLogCount(),
        0,
      ),
    };

    return {
      txCount: this.body.txEffects.length,
      blockNumber: this.number,
      ...logsStats,
    };
  }
}
