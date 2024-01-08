import { AztecAddress, FunctionSelector } from '@aztec/circuits.js';

import { TxHash } from '../tx/index.js';
import { LogId } from './log_id.js';

/**
 * Log filter used to fetch L2 logs.
 * @remarks This filter is applied as an intersection of all it's params.
 */
export type LogFilter = {
  /**
   * Hash of a transaction from which to fetch the logs.
   */
  txHash?: TxHash;
  /**
   * The block number from which to start fetching logs (inclusive).
   */
  fromBlock?: number;
  /** The block number until which to fetch logs (not inclusive). */
  toBlock?: number;
  /**
   * Log id after which to start fetching logs.
   */
  afterLog?: LogId;
  /** The contract address to filter logs by. */
  contractAddress?: AztecAddress;
  /**
   * Selector of the event/log topic.
   * TODO: https://github.com/AztecProtocol/aztec-packages/issues/2632
   */
  selector?: FunctionSelector;
};
