import { L2Block, L2BlockSource } from '@aztec/l2-block';
import { UInt8Vector } from '@aztec/circuits.js';
import { Tx } from '@aztec/tx';
import { makePrivateKernelPublicInputs } from '@aztec/circuits.js/factories';
import { randomBytes } from 'crypto';
import { toBufferBE } from '@aztec/foundation';

export const MockTx = () => {
  return new Tx(makePrivateKernelPublicInputs(), new UInt8Vector(Buffer.alloc(0)), createRandomUnverifiedData(8));
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

const createRandomEncryptedNotePreimage = () => {
  const encryptedNotePreimageBuf = randomBytes(144);
  return Buffer.concat([toBufferBE(BigInt(encryptedNotePreimageBuf.length), 4), encryptedNotePreimageBuf]);
};

const createRandomUnverifiedData = (numPreimages: number) => {
  const encryptedNotePreimageBuf = createRandomEncryptedNotePreimage();
  return Buffer.concat(Array(numPreimages).fill(encryptedNotePreimageBuf));
};
