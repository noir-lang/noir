import { L2Block, L2BlockSource } from '@aztec/l2-block';
import { UInt8Vector } from '@aztec/circuits.js';
import { makePrivateKernelPublicInputs } from '@aztec/circuits.js/factories';
import { Tx } from './tx.js';

export const MockTx = () => {
  return new Tx(makePrivateKernelPublicInputs(), new UInt8Vector(Buffer.alloc(0)));
};

export class MockBlockSource implements L2BlockSource {
  private l2Blocks: L2Block[];

  constructor(private numBlocks = 100) {
    this.l2Blocks = [];
    for (let i = 0; i < this.numBlocks; i++) {
      this.l2Blocks.push(L2Block.random(i));
    }
  }

  public getLatestBlockNum() {
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
