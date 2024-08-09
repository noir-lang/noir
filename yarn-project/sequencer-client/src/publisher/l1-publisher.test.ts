import { L2Block } from '@aztec/circuit-types';
import { sleep } from '@aztec/foundation/sleep';
import { NoopTelemetryClient } from '@aztec/telemetry-client/noop';

import { type MockProxy, mock } from 'jest-mock-extended';

import { L1Publisher, type L1PublisherTxSender, type MinimalTransactionReceipt } from './l1-publisher.js';

describe('L1Publisher', () => {
  let txSender: MockProxy<L1PublisherTxSender>;
  let publishTxHash: string;
  let processTxHash: string;
  let publishAndProcessTxHash: string;
  let processTxReceipt: MinimalTransactionReceipt;
  let publishTxReceipt: MinimalTransactionReceipt;
  let publishAndProcessTxReceipt: MinimalTransactionReceipt;
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
    publishAndProcessTxHash = `0x${Buffer.from('txHashPublishAndProcess').toString('hex')}`; // random tx hash
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
    publishAndProcessTxReceipt = {
      transactionHash: publishAndProcessTxHash,
      status: true,
      logs: [{ data: txsEffectsHash.toString('hex') }],
    } as MinimalTransactionReceipt;
    txSender.sendPublishTx.mockResolvedValueOnce(publishTxHash);
    txSender.sendProcessTx.mockResolvedValueOnce(processTxHash);
    txSender.sendPublishAndProcessTx.mockResolvedValueOnce(publishAndProcessTxHash);
    txSender.getTransactionReceipt.mockResolvedValueOnce(publishTxReceipt).mockResolvedValueOnce(processTxReceipt);
    txSender.getCurrentArchive.mockResolvedValue(l2Block.header.lastArchive.root.toBuffer());

    publisher = new L1Publisher(txSender, new NoopTelemetryClient(), { l1PublishRetryIntervalMS: 1 });
  });

  it('publishes l2 block to l1', async () => {
    const result = await publisher.processL2Block(l2Block);

    expect(result).toEqual(true);
    expect(txSender.sendPublishAndProcessTx).toHaveBeenCalledWith({ header, archive, body });
    expect(txSender.getTransactionReceipt).toHaveBeenCalledWith(publishAndProcessTxHash);
  });

  it('publishes l2 block to l1 (already published body)', async () => {
    txSender.checkIfTxsAreAvailable.mockResolvedValueOnce(true);

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
    expect(txSender.sendPublishAndProcessTx).not.toHaveBeenCalled();
  });

  it('does not retry if sending a process tx fails', async () => {
    txSender.checkIfTxsAreAvailable.mockResolvedValueOnce(true);
    txSender.sendProcessTx.mockReset().mockRejectedValueOnce(new Error()).mockResolvedValueOnce(processTxHash);

    const result = await publisher.processL2Block(l2Block);

    expect(result).toEqual(false);
    expect(txSender.sendProcessTx).toHaveBeenCalledTimes(1);
  });

  it('does not retry if sending a publish and process tx fails', async () => {
    txSender.sendPublishAndProcessTx.mockReset().mockRejectedValueOnce(new Error());
    // .mockResolvedValueOnce(publishAndProcessTxHash);

    const result = await publisher.processL2Block(l2Block);

    expect(result).toEqual(false);
    expect(txSender.sendPublishAndProcessTx).toHaveBeenCalledTimes(1);
  });

  it('retries if fetching the receipt fails (process)', async () => {
    txSender.checkIfTxsAreAvailable.mockResolvedValueOnce(true);
    txSender.getTransactionReceipt
      .mockReset()
      .mockRejectedValueOnce(new Error())
      .mockResolvedValueOnce(processTxReceipt);

    const result = await publisher.processL2Block(l2Block);

    expect(result).toEqual(true);
    expect(txSender.getTransactionReceipt).toHaveBeenCalledTimes(2);
  });

  it('retries if fetching the receipt fails (publish process)', async () => {
    txSender.getTransactionReceipt
      .mockReset()
      .mockRejectedValueOnce(new Error())
      .mockResolvedValueOnce(publishAndProcessTxReceipt);

    const result = await publisher.processL2Block(l2Block);

    expect(result).toEqual(true);
    expect(txSender.getTransactionReceipt).toHaveBeenCalledTimes(2);
  });

  it('returns false if publish and process tx reverts', async () => {
    txSender.getTransactionReceipt.mockReset().mockResolvedValueOnce({ ...publishAndProcessTxReceipt, status: false });

    const result = await publisher.processL2Block(l2Block);

    expect(result).toEqual(false);
  });

  it('returns false if process tx reverts', async () => {
    txSender.checkIfTxsAreAvailable.mockResolvedValueOnce(true);
    txSender.getTransactionReceipt.mockReset().mockResolvedValueOnce({ ...processTxReceipt, status: false });

    const result = await publisher.processL2Block(l2Block);

    expect(result).toEqual(false);
  });

  it('returns false if sending publish and progress tx is interrupted', async () => {
    txSender.sendPublishAndProcessTx.mockReset().mockImplementationOnce(() => sleep(10, publishAndProcessTxHash));

    const resultPromise = publisher.processL2Block(l2Block);
    publisher.interrupt();
    const result = await resultPromise;

    expect(result).toEqual(false);
    expect(txSender.getTransactionReceipt).not.toHaveBeenCalled();
  });

  it('returns false if sending process tx is interrupted', async () => {
    txSender.checkIfTxsAreAvailable.mockResolvedValueOnce(true);
    txSender.sendProcessTx.mockReset().mockImplementationOnce(() => sleep(10, processTxHash));

    const resultPromise = publisher.processL2Block(l2Block);
    publisher.interrupt();
    const result = await resultPromise;

    expect(result).toEqual(false);
    expect(txSender.getTransactionReceipt).not.toHaveBeenCalled();
  });
});
