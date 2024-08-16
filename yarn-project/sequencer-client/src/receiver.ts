import { type L2Block, type Signature } from '@aztec/circuit-types';

/**
 * Given the necessary rollup data, verifies it, and updates the underlying state accordingly to advance the state of the system.
 * See https://hackmd.io/ouVCnacHQRq2o1oRc5ksNA#RollupReceiver.
 */
export interface L2BlockReceiver {
  processL2Block(block: L2Block, attestations?: Signature[]): Promise<boolean>;
}
