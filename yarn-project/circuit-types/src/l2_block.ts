import { Body, L2Tx, TxHash } from '@aztec/circuit-types';
import { AppendOnlyTreeSnapshot, Header, STRING_ENCODING } from '@aztec/circuits.js';
import { makeAppendOnlyTreeSnapshot, makeHeader } from '@aztec/circuits.js/testing';
import { sha256 } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

/**
 * The data that makes up the rollup proof, with encoder decoder functions.
 */
export class L2Block {
  #l1BlockNumber?: bigint;

  constructor(
    /** Snapshot of archive tree after the block is applied. */
    public archive: AppendOnlyTreeSnapshot,
    /** L2 block header. */
    public header: Header,
    /** L2 block body. */
    public body: Body,
    /** Associated L1 block num */
    l1BlockNumber?: bigint,
  ) {
    this.#l1BlockNumber = l1BlockNumber;
  }

  /**
   * Constructs a new instance from named fields.
   * @param fields - Fields to pass to the constructor.
   * @param blockHash - Hash of the block.
   * @param l1BlockNumber - The block number of the L1 block that contains this L2 block.
   * @returns A new instance.
   */
  static fromFields(
    fields: {
      /** Snapshot of archive tree after the block is applied. */
      archive: AppendOnlyTreeSnapshot;
      /** L2 block header. */
      header: Header;
      body: Body;
    },
    l1BlockNumber?: bigint,
  ) {
    return new this(fields.archive, fields.header, fields.body, l1BlockNumber);
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
   * @remarks This can be used specifying no logs, which is used when the block is being served via JSON-RPC because the logs are expected to be served
   * separately.
   * @returns A serialized L2 block logs.
   */
  toBuffer() {
    return serializeToBuffer(this.header, this.archive, this.body);
  }

  /**
   * Deserializes L2 block without logs from a buffer.
   * @param str - A serialized L2 block.
   * @returns Deserialized L2 block.
   */
  static fromString(str: string): L2Block {
    return L2Block.fromBuffer(Buffer.from(str, STRING_ENCODING));
  }

  /**
   * Serializes a block without logs to a string.
   * @remarks This is used when the block is being served via JSON-RPC because the logs are expected to be served
   * separately.
   * @returns A serialized L2 block without logs.
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
   * @returns The L2 block.
   */
  static random(
    l2BlockNum: number,
    txsPerBlock = 4,
    numPrivateCallsPerTx = 2,
    numPublicCallsPerTx = 3,
    numEncryptedLogsPerCall = 2,
    numUnencryptedLogsPerCall = 1,
  ): L2Block {
    return L2Block.fromFields(
      {
        archive: makeAppendOnlyTreeSnapshot(1),
        header: makeHeader(0, l2BlockNum),
        body: Body.random(
          txsPerBlock,
          numPrivateCallsPerTx,
          numPublicCallsPerTx,
          numEncryptedLogsPerCall,
          numUnencryptedLogsPerCall,
        ),
      },
      // just for testing purposes, each random L2 block got emitted in the equivalent L1 block
      BigInt(l2BlockNum),
    );
  }

  get number(): number {
    return Number(this.header.globalVariables.blockNumber.toBigInt());
  }

  /**
   * Gets the L1 block number that included this block
   */
  public getL1BlockNumber(): bigint {
    if (typeof this.#l1BlockNumber === 'undefined') {
      throw new Error('L1 block number has to be attached before calling "getL1BlockNumber"');
    }

    return this.#l1BlockNumber;
  }

  /**
   * Sets the L1 block number that included this block
   * @param l1BlockNumber - The block number of the L1 block that contains this L2 block.
   */
  public setL1BlockNumber(l1BlockNumber: bigint) {
    this.#l1BlockNumber = l1BlockNumber;
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
    const buf = serializeToBuffer(
      this.header.globalVariables,
      AppendOnlyTreeSnapshot.zero(), // this.startNoteHashTreeSnapshot / commitments,
      AppendOnlyTreeSnapshot.zero(), // this.startNullifierTreeSnapshot,
      AppendOnlyTreeSnapshot.zero(), // this.startContractTreeSnapshot,
      AppendOnlyTreeSnapshot.zero(), // this.startPublicDataTreeSnapshot,
      AppendOnlyTreeSnapshot.zero(), // this.startL1ToL2MessageTreeSnapshot,
      this.header.lastArchive,
      this.header.state.partial.noteHashTree,
      this.header.state.partial.nullifierTree,
      this.header.state.partial.contractTree,
      this.header.state.partial.publicDataTree,
      this.header.state.l1ToL2MessageTree,
      this.archive,
      this.body.getCalldataHash(),
      this.getL1ToL2MessagesHash(),
    );

    return Fr.fromBufferReduce(sha256(buf));
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
      AppendOnlyTreeSnapshot.zero(), // this.startContractTreeSnapshot,
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
      this.header.state.partial.contractTree,
      this.header.state.partial.publicDataTree,
      this.header.state.l1ToL2MessageTree,
      this.archive,
    );
    return sha256(inputValue);
  }

  /**
   * Compute the hash of all of this blocks l1 to l2 messages,
   * The hash is also calculated within the contract when the block is submitted.
   * @returns The hash of all of the l1 to l2 messages.
   */
  getL1ToL2MessagesHash(): Buffer {
    // Create a long buffer of all of the l1 to l2 messages
    const l1ToL2Messages = Buffer.concat(this.body.l1ToL2Messages.map(message => message.toBuffer()));
    return sha256(l1ToL2Messages);
  }

  /**
   * Get the ith transaction in an L2 block.
   * @param txIndex - The index of the tx in the block.
   * @returns The tx.
   */
  getTx(txIndex: number) {
    this.assertIndexInRange(txIndex);

    const txEffect = this.body.txEffects[txIndex];

    const newNoteHashes = txEffect.newNoteHashes.filter(x => !x.isZero());
    const newNullifiers = txEffect.newNullifiers.filter(x => !x.isZero());
    const newPublicDataWrites = txEffect.newPublicDataWrites.filter(x => !x.isEmpty());
    const newL2ToL1Msgs = txEffect.newL2ToL1Msgs.filter(x => !x.isZero());
    const newContracts = txEffect.contractLeaves.filter(x => !x.isZero());
    const newContractData = txEffect.contractData.filter(x => !x.isEmpty());

    return new L2Tx(
      newNoteHashes,
      newNullifiers,
      newPublicDataWrites,
      newL2ToL1Msgs,
      newContracts,
      newContractData,
      this.hash(),
      Number(this.header.globalVariables.blockNumber.toBigInt()),
    );
  }

  /**
   * A lightweight method to get the tx hash of a tx in the block.
   * @param txIndex - the index of the tx in the block
   * @returns a hash of the tx, which is the first nullifier in the tx
   */
  getTxHash(txIndex: number): TxHash {
    this.assertIndexInRange(txIndex);

    // Gets the first nullifier of the tx specified by txIndex
    const firstNullifier = this.body.txEffects[txIndex].newNullifiers[0];

    return new TxHash(firstNullifier.toBuffer());
  }

  /**
   * Get all the transaction in an L2 block.
   * @returns The tx.
   */
  getTxs() {
    return Array(this.body.numberOfTxs)
      .fill(0)
      .map((_, i) => this.getTx(i));
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
      txCount: this.body.numberOfTxs,
      blockNumber: this.number,
      ...logsStats,
    };
  }

  assertIndexInRange(txIndex: number) {
    if (txIndex < 0 || txIndex >= this.body.numberOfTxs) {
      throw new IndexOutOfRangeError({
        txIndex,
        numberOfTxs: this.body.numberOfTxs,
        blockNumber: this.number,
      });
    }
  }
}

/**
 * Custom error class for when a requested tx index is out of range.
 */
export class IndexOutOfRangeError extends Error {
  constructor({
    txIndex,
    numberOfTxs,
    blockNumber,
  }: {
    /**
     * The requested index of the tx in the block.
     */
    txIndex: number;
    /**
     * The number of txs in the block.
     */
    numberOfTxs: number;
    /**
     * The number of the block.
     */
    blockNumber: number;
  }) {
    super(`IndexOutOfRangeError: Failed to get tx at index ${txIndex}. Block ${blockNumber} has ${numberOfTxs} txs.`);
  }
}
