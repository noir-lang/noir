import { L2Block } from '@aztec/circuit-types';
import { sleep } from '@aztec/foundation/sleep';

import { type MockProxy, mock } from 'jest-mock-extended';

import { L1Publisher, type L1PublisherTxSender, type MinimalTransactionReceipt } from './l1-publisher.js';

describe('L1Publisher', () => {
  let txSender: MockProxy<L1PublisherTxSender>;
  let publishTxHash: string;
  let processTxHash: string;
  let processTxReceipt: MinimalTransactionReceipt;
  let publishTxReceipt: MinimalTransactionReceipt;
  let l2Block: L2Block;

  let header: Buffer;
  let archive: Buffer;
  let txsEffectsHash: Buffer;
  let body: Buffer;

  let publisher: L1Publisher;

  beforeEach(() => {
    l2Block = L2Block.random(42);

    header = l2Block.header.toBuffer();
    archive = l2Block.archive.root.toBuffer();
    txsEffectsHash = l2Block.body.getTxsEffectsHash();
    body = l2Block.body.toBuffer();

    txSender = mock<L1PublisherTxSender>();

    publishTxHash = `0x${Buffer.from('txHashPublish').toString('hex')}`; // random tx hash
    processTxHash = `0x${Buffer.from('txHashProcess').toString('hex')}`; // random tx hash
    publishTxReceipt = {
      transactionHash: publishTxHash,
      status: true,
      logs: [{ data: txsEffectsHash.toString('hex') }],
    } as MinimalTransactionReceipt;
    processTxReceipt = {
      transactionHash: processTxHash,
      status: true,
      logs: [{ data: '' }],
    } as MinimalTransactionReceipt;
    txSender.sendPublishTx.mockResolvedValueOnce(publishTxHash);
    txSender.sendProcessTx.mockResolvedValueOnce(processTxHash);
    txSender.getTransactionReceipt.mockResolvedValueOnce(publishTxReceipt).mockResolvedValueOnce(processTxReceipt);
    txSender.getCurrentArchive.mockResolvedValue(l2Block.header.lastArchive.root.toBuffer());

    publisher = new L1Publisher(txSender, { l1PublishRetryIntervalMS: 1 });
  });

  it('publishes l2 block to l1', async () => {
    const result = await publisher.processL2Block(l2Block);

    expect(result).toEqual(true);
    expect(txSender.sendProcessTx).toHaveBeenCalledWith({ header, archive, body });
    expect(txSender.getTransactionReceipt).toHaveBeenCalledWith(processTxHash);
  });

  it('does not publish if last archive root is different to expected', async () => {
    txSender.getCurrentArchive.mockResolvedValueOnce(L2Block.random(43).archive.root.toBuffer());
    const result = await publisher.processL2Block(l2Block);
    expect(result).toBe(false);
    expect(txSender.sendPublishTx).not.toHaveBeenCalled();
    expect(txSender.sendProcessTx).not.toHaveBeenCalled();
  });

  it('does not retry if sending a publish tx fails', async () => {
    txSender.sendPublishTx.mockReset().mockRejectedValueOnce(new Error()).mockResolvedValueOnce(publishTxHash);

    const result = await publisher.processL2Block(l2Block);

    expect(result).toEqual(false);
    expect(txSender.sendPublishTx).toHaveBeenCalledTimes(1);
    expect(txSender.sendProcessTx).toHaveBeenCalledTimes(0);
  });

  it('does not retry if sending a process tx fails', async () => {
    txSender.sendProcessTx.mockReset().mockRejectedValueOnce(new Error()).mockResolvedValueOnce(processTxHash);

    const result = await publisher.processL2Block(l2Block);

    expect(result).toEqual(false);
    expect(txSender.sendPublishTx).toHaveBeenCalledTimes(1);
    expect(txSender.sendProcessTx).toHaveBeenCalledTimes(1);
  });

  it('retries if fetching the receipt fails', async () => {
    txSender.getTransactionReceipt
      .mockReset()
      .mockRejectedValueOnce(new Error())
      .mockResolvedValueOnce(publishTxReceipt)
      .mockRejectedValueOnce(new Error())
      .mockResolvedValueOnce(processTxReceipt);

    const result = await publisher.processL2Block(l2Block);

    expect(result).toEqual(true);
    expect(txSender.getTransactionReceipt).toHaveBeenCalledTimes(4);
  });

  it('returns false if publish tx reverts', async () => {
    txSender.getTransactionReceipt.mockReset().mockResolvedValueOnce({ ...publishTxReceipt, status: false });

    const result = await publisher.processL2Block(l2Block);

    expect(result).toEqual(false);
  });

  it('returns false if process tx reverts', async () => {
    txSender.getTransactionReceipt
      .mockReset()
      .mockResolvedValueOnce(publishTxReceipt)
      .mockResolvedValueOnce({ ...publishTxReceipt, status: false });

    const result = await publisher.processL2Block(l2Block);

    expect(result).toEqual(false);
  });

  it('returns false if sending publish tx is interrupted', async () => {
    txSender.sendPublishTx.mockReset().mockImplementationOnce(() => sleep(10, publishTxHash));

    const resultPromise = publisher.processL2Block(l2Block);
    publisher.interrupt();
    const result = await resultPromise;

    expect(result).toEqual(false);
    expect(txSender.getTransactionReceipt).not.toHaveBeenCalled();
  });

  it('returns false if sending process tx is interrupted', async () => {
    txSender.sendProcessTx.mockReset().mockImplementationOnce(() => sleep(10, processTxHash));

    const resultPromise = publisher.processL2Block(l2Block);
    publisher.interrupt();
    const result = await resultPromise;

    expect(result).toEqual(false);
    expect(txSender.getTransactionReceipt).not.toHaveBeenCalled();
  });
});
