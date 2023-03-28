import { L2Block } from '@aztec/l2-block';

/**
 * Given the necessary rollup data, verifies it, and updates the underlying state accordingly to advance the state of the system.
 * See https://hackmd.io/ouVCnacHQRq2o1oRc5ksNA#RollupReceiver.
 */
export interface L2BlockReceiver {
  processL2Block(l2BlockData: L2Block): Promise<boolean>;
}
