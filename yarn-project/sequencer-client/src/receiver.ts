import { type L2Block } from '@aztec/circuit-types';

/**
 * Given the necessary rollup data, verifies it, and updates the underlying state accordingly to advance the state of the system.
 * See https://hackmd.io/ouVCnacHQRq2o1oRc5ksNA#RollupReceiver.
 */
export interface L2BlockReceiver {
  /**
   * Receive and L2 block and process it, returns true if successful.
   * @param l2BlockData - L2 block to process.
   */
  processL2Block(l2BlockData: L2Block): Promise<boolean>;
}
