import { type AztecAddress, type EthAddress, type Fr } from '@aztec/circuits.js';

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

  /** The list of permitted fee payment contract classes */
  allowedFeePaymentContractClasses?: Fr[];

  /** The list of permitted fee payment contract instances. Takes precedence over contract classes */
  allowedFeePaymentContractInstances?: AztecAddress[];
}
