import { UInt8Vector } from '@aztec/circuits.js';
import { makeKernelPublicInputs } from '@aztec/circuits.js/factories';
import { AztecAddress } from '@aztec/foundation';
import { ContractData, L2Block, L2BlockSource } from '@aztec/types';
import { Tx } from '@aztec/types';
import { UnverifiedData } from '@aztec/types';

export const MockTx = () => {
  return Tx.createPrivate(makeKernelPublicInputs(), new UInt8Vector(Buffer.alloc(0)), UnverifiedData.random(8));
};

export class MockBlockSource implements L2BlockSource {
  private l2Blocks: L2Block[];

  constructor(private numBlocks = 100) {
    this.l2Blocks = [];
    for (let i = 0; i < this.numBlocks; i++) {
      this.l2Blocks.push(L2Block.random(i));
    }
  }

  /**
   * Lookup the L2 contract data for this contract.
   * Contains information such as the ethereum portal address.
   * @param contractAddress - The contract data address.
   * @returns The portal address (if we didn't throw an error).
   */
  public getL2ContractData(contractAddress: AztecAddress): Promise<ContractData | undefined> {
    for (const block of this.l2Blocks) {
      for (const contractData of block.newContractData) {
        if (contractData.contractAddress.equals(contractAddress)) {
          return Promise.resolve(contractData);
        }
      }
    }
    return Promise.resolve(undefined);
  }

  public getBlockHeight() {
    return Promise.resolve(this.l2Blocks.length - 1);
  }

  public getL2Blocks(from: number, take: number) {
    return Promise.resolve(this.l2Blocks.slice(from, from + take));
  }

  public start(): Promise<void> {
    return Promise.resolve();
  }

  public stop(): Promise<void> {
    return Promise.resolve();
  }
}
