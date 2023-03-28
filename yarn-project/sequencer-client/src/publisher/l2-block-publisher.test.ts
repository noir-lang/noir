import { L2Block } from '@aztec/l2-block';
import { TxHash } from '@aztec/ethereum.js/eth_rpc';
import { mock, MockProxy } from 'jest-mock-extended';
import { sleep } from '../utils.js';
import { L2BlockPublisher, PublisherTxSender } from './l2-block-publisher.js';

describe('L2BlockPublisher', () => {
  let txSender: MockProxy<PublisherTxSender>;
  let txHash: string;
  let txReceipt: { transactionHash: string; status: boolean };
  let l2Block: L2Block;
  let l2Inputs: Buffer;
  let l2Proof: Buffer;
  let publisher: L2BlockPublisher;

  beforeEach(() => {
    l2Block = L2Block.random(42);
    l2Inputs = l2Block.encode();
    l2Proof = Buffer.alloc(0);

    txSender = mock<PublisherTxSender>();
    txHash = TxHash.random().toString();
    txReceipt = { transactionHash: txHash, status: true };
    txSender.sendTransaction.mockResolvedValueOnce(txHash);
    txSender.getTransactionReceipt.mockResolvedValueOnce(txReceipt);

    publisher = new L2BlockPublisher(txSender, { retryIntervalMs: 1 });
  });

  it('publishes l2 block to l1', async () => {
    const result = await publisher.processL2Block(l2Block);

    expect(result).toEqual(true);
    expect(txSender.sendTransaction).toHaveBeenCalledWith({ proof: l2Proof, inputs: l2Inputs });
    expect(txSender.getTransactionReceipt).toHaveBeenCalledWith(txHash);
  });

  it('retries if sending a tx fails', async () => {
    txSender.sendTransaction.mockReset().mockRejectedValueOnce(new Error()).mockResolvedValueOnce(txHash);

    const result = await publisher.processL2Block(l2Block);

    expect(result).toEqual(true);
    expect(txSender.sendTransaction).toHaveBeenCalledTimes(2);
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
    txSender.sendTransaction.mockReset().mockImplementationOnce(() => sleep(10, txHash));

    const resultPromise = publisher.processL2Block(l2Block);
    publisher.interrupt();
    const result = await resultPromise;

    expect(result).toEqual(false);
    expect(txSender.getTransactionReceipt).not.toHaveBeenCalled();
  });

  it.skip('waits for fee distributor balance', () => {});

  it.skip('fails if contract is changed underfoot', () => {});
});
