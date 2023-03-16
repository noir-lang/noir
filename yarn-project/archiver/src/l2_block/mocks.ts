import nodeCrypto from 'crypto';
import { AppendOnlyTreeSnapshot, ContractData } from './l2_block.js';

export const randomBytes = (len: number) => {
  return nodeCrypto.randomBytes(len) as Buffer;
};

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
