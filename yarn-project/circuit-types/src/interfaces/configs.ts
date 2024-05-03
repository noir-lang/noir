import { type AztecAddress, type EthAddress, type Fr, type FunctionSelector } from '@aztec/circuits.js';

/** A function that the sequencer allows to run in either setup or teardown phase */
export type AllowedFunction =
  | {
      /** The contract address this selector is valid for */
      address: AztecAddress;
      /** The function selector */
      selector: FunctionSelector;
    }
  | {
      /** The contract class this selector is valid for */
      classId: Fr;
      /** The function selector */
      selector: FunctionSelector;
    };

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
  /** Recipient of block reward. */
  coinbase?: EthAddress;
  /** Address to receive fees. */
  feeRecipient?: AztecAddress;
  /** The working directory to use for simulation/proving */
  acvmWorkingDirectory?: string;
  /** The path to the ACVM binary */
  acvmBinaryPath?: string;
  /** The list of functions calls allowed to run in setup */
  allowedFunctionsInSetup?: AllowedFunction[];
  /** The list of functions calls allowed to run teardown */
  allowedFunctionsInTeardown?: AllowedFunction[];
}
