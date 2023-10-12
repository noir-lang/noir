/** Stats associated with an L2 block. */
export type L2BlockStats = {
  /** Number of txs in the L2 block. */
  txCount: number;
  /** Number of the L2 block. */
  blockNumber: number;
  /** Number of encrypted logs. */
  encryptedLogCount?: number;
  /** Number of unencrypted logs. */
  unencryptedLogCount?: number;
  /** Serialised size of encrypted logs. */
  encryptedLogSize?: number;
  /** Serialised size of unencrypted logs. */
  unencryptedLogSize?: number;
};

/** Stats logged for each L1 rollup publish tx.*/
export type L1PublishStats = {
  /** Name of the event for metrics purposes */
  eventName: 'rollup-published-to-l1';
  /** Effective gas price of the tx. */
  gasPrice: bigint;
  /** Effective gas used in the tx. */
  gasUsed: bigint;
  /** Hash of the L1 tx. */
  transactionHash: string;
  /** Gas cost of the calldata. */
  calldataGas: number;
  /** Size in bytes of the calldata. */
  calldataSize: number;
} & L2BlockStats;

/** Stats logged for synching node chain history.  */
export type NodeSyncedChainHistoryStats = {
  /** Name of the event. */
  eventName: 'node-synced-chain-history';
  /** Number of txs in the L2 block.. */
  txCount: number;
  /** Number of txs in each block. */
  txsPerBlock: number;
  /** Duration in ms. */
  duration: number;
  /** Id of the L2 block. */
  blockNumber: number;
  /** Number of blocks processed. */
  blockCount: number;
  /** Size of the db in bytes. */
  dbSize: number;
};

/** Stats for circuit simulation. */
export type CircuitSimulationStats = {
  /** name of the event. */
  eventName: 'circuit-simulation';
  /** Name of the circuit. */
  circuitName:
    | 'base-rollup'
    | 'private-kernel-init'
    | 'private-kernel-ordering'
    | 'root-rollup'
    | 'merge-rollup'
    | 'private-kernel-inner'
    | 'public-kernel-private-input'
    | 'public-kernel-non-first-iteration';
  /** Duration in ms. */
  duration: number;
  /** Size in bytes of circuit inputs. */
  inputSize: number;
  /** Size in bytes of circuit outputs (aka public inputs). */
  outputSize: number;
};

/** Stats for an L2 block built by a sequencer. */
export type L2BlockBuiltStats = {
  /** Name of the event. */
  eventName: 'l2-block-built';
  /** Total duration in ms. */
  duration: number;
  /** Time for processing public txs in ms. */
  publicProcessDuration: number;
  /** Time for running rollup circuits in ms.  */
  rollupCircuitsDuration: number;
} & L2BlockStats;

/** Stats for an L2 block processed by the world state synchronizer. */
export type L2BlockHandledStats = {
  /** Name of the event. */
  eventName: 'l2-block-handled';
  /** Total duration in ms. */
  duration: number;
  /** Whether the block was produced by this node. */
  isBlockOurs: boolean;
} & L2BlockStats;

/** Stats for a note processor that has caught up with the chain. */
export type NoteProcessorCaughtUpStats = {
  /** Name of the event. */
  eventName: 'note-processor-caught-up';
  /** Public key of the note processor. */
  publicKey: string;
  /** Total time to catch up with the tip of the chain from scratch in ms. */
  duration: number;
  /** Size of the notes db. */
  dbSize: number;
} & NoteProcessorStats;

/** Accumulated rolling stats for a note processor.  */
export type NoteProcessorStats = {
  /** How many notes have been seen and trial-decrypted. */
  seen: number;
  /** How many notes were successfully decrypted. */
  decrypted: number;
  /** How many notes failed processing. */
  failed: number;
  /** How many blocks were spanned.  */
  blocks: number;
  /** How many txs were spanned.  */
  txs: number;
};

/** Stats for a tx. */
export type TxStats = {
  /** Hash of the tx. */
  txHash: string;
  /** Total size in bytes. */
  size: number;
  /** Size of the proof. */
  proofSize: number;
  /** Number of encrypted logs. */
  encryptedLogCount: number;
  /** Number of unencrypted logs. */
  unencryptedLogCount: number;
  /** Serialised size of encrypted logs. */
  encryptedLogSize: number;
  /** Serialised size of unencrypted logs. */
  unencryptedLogSize: number;
  /** Serialised size of new contract data. */
  newContractDataSize: number;
  /** Number of new contracts deployed in this tx. */
  newContractCount: number;
};

/** A new tx was added to the tx pool. */
export type TxAddedToPoolStats = {
  /** Name of the event. */
  eventName: 'tx-added-to-pool';
} & TxStats;

/** Stats emitted in structured logs with an `eventName` for tracking. */
export type Stats =
  | L1PublishStats
  | NodeSyncedChainHistoryStats
  | CircuitSimulationStats
  | L2BlockBuiltStats
  | L2BlockHandledStats
  | NoteProcessorCaughtUpStats
  | TxAddedToPoolStats;

/** Set of event names across emitted stats. */
export type StatsEventName = Stats['eventName'];
