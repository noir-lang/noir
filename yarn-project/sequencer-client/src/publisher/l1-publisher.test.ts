import { L2Block } from '@aztec/types';

import { MockProxy, mock } from 'jest-mock-extended';

import { sleep } from '../utils.js';
import { L1Publisher, L1PublisherTxSender, MinimalTransactionReceipt } from './l1-publisher.js';

describe('L1Publisher', () => {
  let txSender: MockProxy<L1PublisherTxSender>;
  let txHash: string;
  let txReceipt: MinimalTransactionReceipt;
  let l2Block: L2Block;
  let l2Inputs: Buffer;
  let l2Proof: Buffer;
  let publisher: L1Publisher;

  beforeEach(() => {
    l2Block = L2Block.random(42);
    l2Inputs = l2Block.encode();
    l2Proof = Buffer.alloc(0);

    txSender = mock<L1PublisherTxSender>();
    txHash = `0x${Buffer.from('txHash').toString('hex')}`; // random tx hash
    txReceipt = { transactionHash: txHash, status: true } as MinimalTransactionReceipt;
    txSender.sendProcessTx.mockResolvedValueOnce(txHash);
    txSender.getTransactionReceipt.mockResolvedValueOnce(txReceipt);

    publisher = new L1Publisher(txSender, { l1BlockPublishRetryIntervalMS: 1 });
  });

  it('publishes l2 block to l1', async () => {
    const result = await publisher.processL2Block(l2Block);

    expect(result).toEqual(true);
    expect(txSender.sendProcessTx).toHaveBeenCalledWith({ proof: l2Proof, inputs: l2Inputs });
    expect(txSender.getTransactionReceipt).toHaveBeenCalledWith(txHash);
  });

  it('does not retry if sending a tx fails', async () => {
    txSender.sendProcessTx.mockReset().mockRejectedValueOnce(new Error()).mockResolvedValueOnce(txHash);

    const result = await publisher.processL2Block(l2Block);

    expect(result).toEqual(false);
    expect(txSender.sendProcessTx).toHaveBeenCalledTimes(1);
  });

  it('retries if fetching the receipt fails', async () => {
    txSender.getTransactionReceipt.mockReset().mockRejectedValueOnce(new Error()).mockResolvedValueOnce(txReceipt);

    const result = await publisher.processL2Block(l2Block);

    expect(result).toEqual(true);
    expect(txSender.getTransactionReceipt).toHaveBeenCalledTimes(2);
  });

  it('returns false if tx reverts', async () => {
    txSender.getTransactionReceipt.mockReset().mockResolvedValueOnce({ ...txReceipt, status: false });

    const result = await publisher.processL2Block(l2Block);

    expect(result).toEqual(false);
  });

  it('returns false if interrupted', async () => {
    txSender.sendProcessTx.mockReset().mockImplementationOnce(() => sleep(10, txHash));

    const resultPromise = publisher.processL2Block(l2Block);
    publisher.interrupt();
    const result = await resultPromise;

    expect(result).toEqual(false);
    expect(txSender.getTransactionReceipt).not.toHaveBeenCalled();
  });

  it.skip('waits for fee distributor balance', () => {});

  it.skip('fails if contract is changed underfoot', () => {});
});
