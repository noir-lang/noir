import { type AztecAddress, type EthAddress, type Fr, type FunctionSelector } from '@aztec/circuits.js';

type AllowedInstance = { address: AztecAddress };
type AllowedInstanceFunction = { address: AztecAddress; selector: FunctionSelector };
type AllowedClass = { classId: Fr };
type AllowedClassFunction = { classId: Fr; selector: FunctionSelector };

export type AllowedElement = AllowedInstance | AllowedInstanceFunction | AllowedClass | AllowedClassFunction;

/**
 * The sequencer configuration.
 */
export interface SequencerConfig {
  /** The number of ms to wait between polling for pending txs. */
  transactionPollingIntervalMS?: number;
  /** The maximum number of txs to include in a block. */
  maxTxsPerBlock?: number;
  /** The minimum number of txs to include in a block. */
  minTxsPerBlock?: number;
  /** The minimum number of seconds inbetween consecutive blocks. */
  minSecondsBetweenBlocks?: number;
  /** The maximum number of seconds inbetween consecutive blocks. Sequencer will produce a block with less than minTxsPerBlock once this threshold is reached. */
  maxSecondsBetweenBlocks?: number;
  /** Recipient of block reward. */
  coinbase?: EthAddress;
  /** Address to receive fees. */
  feeRecipient?: AztecAddress;
  /** The working directory to use for simulation/proving */
  acvmWorkingDirectory?: string;
  /** The path to the ACVM binary */
  acvmBinaryPath?: string;
  /** The list of functions calls allowed to run in setup */
  allowedInSetup?: AllowedElement[];
  /** The list of functions calls allowed to run teardown */
  allowedInTeardown?: AllowedElement[];
  /** Max block size */
  maxBlockSizeInBytes?: number;
  /** Whether to require every tx to have a fee payer */
  enforceFees?: boolean;
  /** Temporary flag to skip submitting proofs, so a prover-node takes care of it. */
  sequencerSkipSubmitProofs?: boolean;
}
