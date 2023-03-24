import { randomBytes } from '@aztec/foundation';

import { AppendOnlyTreeSnapshot, ContractData } from './l2_block.js';

export const randomAppendOnlyTreeSnapshot = (nextIndex: number): AppendOnlyTreeSnapshot => {
  return {
    root: randomBytes(32),
    nextAvailableLeafIndex: nextIndex,
  };
};

export const randomContractData = (): ContractData => {
  return {
    aztecAddress: randomBytes(32),
    ethAddress: randomBytes(20),
  };
};
