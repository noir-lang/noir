import { L2Block } from '@aztec/circuit-types';
import { EthAddress, Fr } from '@aztec/circuits.js';
import { sleep } from '@aztec/foundation/sleep';
import { NoopTelemetryClient } from '@aztec/telemetry-client/noop';

import { type MockProxy, mock } from 'jest-mock-extended';
import { type GetTransactionReceiptReturnType, type PrivateKeyAccount } from 'viem';

import { type PublisherConfig, type TxSenderConfig } from './config.js';
import { L1Publisher } from './l1-publisher.js';

interface MockAvailabilityOracleWrite {
  publish: (args: readonly [`0x${string}`], options: { account: PrivateKeyAccount }) => Promise<`0x${string}`>;
}

interface MockAvailabilityOracleRead {
  isAvailable: (args: readonly [`0x${string}`]) => Promise<boolean>;
}

class MockAvailabilityOracle {
  constructor(public write: MockAvailabilityOracleWrite, public read: MockAvailabilityOracleRead) {}
}

interface MockPublicClient {
  getTransactionReceipt: ({ hash }: { hash: '0x${string}' }) => Promise<GetTransactionReceiptReturnType>;
  getBlock(): Promise<{ timestamp: number }>;
  getTransaction: ({ hash }: { hash: '0x${string}' }) => Promise<{ input: `0x${string}`; hash: `0x${string}` }>;
}

interface MockRollupContractWrite {
  publishAndProcess: (
    args: readonly [`0x${string}`, `0x${string}`, `0x${string}`],
    options: { account: PrivateKeyAccount },
  ) => Promise<`0x${string}`>;

  process: (
    args: readonly [`0x${string}`, `0x${string}`],
    options: { account: PrivateKeyAccount },
  ) => Promise<`0x${string}`>;
}

interface MockRollupContractRead {
  archive: () => Promise<`0x${string}`>;
}

class MockRollupContract {
  constructor(public write: MockRollupContractWrite, public read: MockRollupContractRead) {}
}

describe('L1Publisher', () => {
  let rollupContractRead: MockProxy<MockRollupContractRead>;
  let rollupContractWrite: MockProxy<MockRollupContractWrite>;
  let rollupContract: MockRollupContract;

  let availabilityOracleRead: MockProxy<MockAvailabilityOracleRead>;
  let availabilityOracleWrite: MockProxy<MockAvailabilityOracleWrite>;
  let availabilityOracle: MockAvailabilityOracle;

  let publicClient: MockProxy<MockPublicClient>;

  let processTxHash: `0x${string}`;
  let publishAndProcessTxHash: `0x${string}`;
  let processTxReceipt: GetTransactionReceiptReturnType;
  let publishAndProcessTxReceipt: GetTransactionReceiptReturnType;
  let l2Block: L2Block;

  let header: Buffer;
  let archive: Buffer;
  let body: Buffer;

  let account: PrivateKeyAccount;

  let publisher: L1Publisher;

  beforeEach(() => {
    l2Block = L2Block.random(42);

    header = l2Block.header.toBuffer();
    archive = l2Block.archive.root.toBuffer();
    body = l2Block.body.toBuffer();

    processTxHash = `0x${Buffer.from('txHashProcess').toString('hex')}`; // random tx hash
    publishAndProcessTxHash = `0x${Buffer.from('txHashPublishAndProcess').toString('hex')}`; // random tx hash

    processTxReceipt = {
      transactionHash: processTxHash,
      status: 'success',
      logs: [],
    } as unknown as GetTransactionReceiptReturnType;
    publishAndProcessTxReceipt = {
      transactionHash: publishAndProcessTxHash,
      status: 'success',
      logs: [],
    } as unknown as GetTransactionReceiptReturnType;

    rollupContractWrite = mock<MockRollupContractWrite>();
    rollupContractRead = mock<MockRollupContractRead>();
    rollupContract = new MockRollupContract(rollupContractWrite, rollupContractRead);

    availabilityOracleWrite = mock<MockAvailabilityOracleWrite>();
    availabilityOracleRead = mock<MockAvailabilityOracleRead>();
    availabilityOracle = new MockAvailabilityOracle(availabilityOracleWrite, availabilityOracleRead);

    publicClient = mock<MockPublicClient>();

    const config = {
      l1RpcUrl: `http://127.0.0.1:8545`,
      l1ChainId: 1,
      publisherPrivateKey: `0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80`,
      l1Contracts: {
        availabilityOracleAddress: EthAddress.ZERO.toString(),
        rollupAddress: EthAddress.ZERO.toString(),
      },
      l1PublishRetryIntervalMS: 1,
    } as unknown as TxSenderConfig & PublisherConfig;

    publisher = new L1Publisher(config, new NoopTelemetryClient());

    (publisher as any)['availabilityOracleContract'] = availabilityOracle;
    (publisher as any)['rollupContract'] = rollupContract;
    (publisher as any)['publicClient'] = publicClient;

    account = (publisher as any)['account'];
  });

  it('publishes and process l2 block to l1', async () => {
    rollupContractRead.archive.mockResolvedValue(l2Block.header.lastArchive.root.toString() as `0x${string}`);
    rollupContractWrite.publishAndProcess.mockResolvedValueOnce(publishAndProcessTxHash);
    publicClient.getTransactionReceipt.mockResolvedValueOnce(publishAndProcessTxReceipt);

    const result = await publisher.processL2Block(l2Block);

    expect(result).toEqual(true);

    const args = [`0x${header.toString('hex')}`, `0x${archive.toString('hex')}`, `0x${body.toString('hex')}`] as const;
    expect(rollupContractWrite.publishAndProcess).toHaveBeenCalledWith(args, { account: account });
    expect(publicClient.getTransactionReceipt).toHaveBeenCalledWith({ hash: publishAndProcessTxHash });
  });

  it('publishes l2 block to l1 (already published body)', async () => {
    availabilityOracleRead.isAvailable.mockResolvedValueOnce(true);
    rollupContractRead.archive.mockResolvedValue(l2Block.header.lastArchive.root.toString() as `0x${string}`);
    rollupContractWrite.process.mockResolvedValueOnce(processTxHash);
    publicClient.getTransactionReceipt.mockResolvedValueOnce(processTxReceipt);

    const result = await publisher.processL2Block(l2Block);

    expect(result).toEqual(true);
    const args = [`0x${header.toString('hex')}`, `0x${archive.toString('hex')}`] as const;
    expect(rollupContractWrite.process).toHaveBeenCalledWith(args, { account });
    expect(publicClient.getTransactionReceipt).toHaveBeenCalledWith({ hash: processTxHash });
  });

  it('does not publish if last archive root is different to expected', async () => {
    rollupContractRead.archive.mockResolvedValue(Fr.random().toString());

    const result = await publisher.processL2Block(l2Block);
    expect(result).toBe(false);
    expect(availabilityOracleWrite.publish).not.toHaveBeenCalled();
    expect(rollupContractWrite.process).not.toHaveBeenCalled();
    expect(rollupContractWrite.publishAndProcess).not.toHaveBeenCalled();
  });

  it('does not retry if sending a process tx fails', async () => {
    availabilityOracleRead.isAvailable.mockResolvedValueOnce(true);
    rollupContractRead.archive.mockResolvedValue(l2Block.header.lastArchive.root.toString() as `0x${string}`);
    rollupContractWrite.process
      .mockRejectedValueOnce(new Error())
      .mockResolvedValueOnce(processTxHash as `0x${string}`);

    const result = await publisher.processL2Block(l2Block);

    expect(result).toEqual(false);
    expect(rollupContractWrite.process).toHaveBeenCalledTimes(1);
  });

  it('does not retry if sending a publish and process tx fails', async () => {
    rollupContractRead.archive.mockResolvedValue(l2Block.header.lastArchive.root.toString() as `0x${string}`);
    rollupContractWrite.publishAndProcess.mockRejectedValueOnce(new Error());

    const result = await publisher.processL2Block(l2Block);

    expect(result).toEqual(false);
    expect(rollupContractWrite.publishAndProcess).toHaveBeenCalledTimes(1);
  });

  it('retries if fetching the receipt fails (process)', async () => {
    availabilityOracleRead.isAvailable.mockResolvedValueOnce(true);
    rollupContractRead.archive.mockResolvedValue(l2Block.header.lastArchive.root.toString() as `0x${string}`);
    rollupContractWrite.process.mockResolvedValueOnce(processTxHash);
    publicClient.getTransactionReceipt.mockRejectedValueOnce(new Error()).mockResolvedValueOnce(processTxReceipt);

    const result = await publisher.processL2Block(l2Block);

    expect(result).toEqual(true);
    expect(publicClient.getTransactionReceipt).toHaveBeenCalledTimes(2);
  });

  it('retries if fetching the receipt fails (publish process)', async () => {
    rollupContractRead.archive.mockResolvedValue(l2Block.header.lastArchive.root.toString() as `0x${string}`);
    rollupContractWrite.publishAndProcess.mockResolvedValueOnce(publishAndProcessTxHash as `0x${string}`);
    publicClient.getTransactionReceipt
      .mockRejectedValueOnce(new Error())
      .mockResolvedValueOnce(publishAndProcessTxReceipt);

    const result = await publisher.processL2Block(l2Block);

    expect(result).toEqual(true);
    expect(publicClient.getTransactionReceipt).toHaveBeenCalledTimes(2);
  });

  it('returns false if publish and process tx reverts', async () => {
    rollupContractRead.archive.mockResolvedValue(l2Block.header.lastArchive.root.toString() as `0x${string}`);
    rollupContractWrite.publishAndProcess.mockResolvedValueOnce(publishAndProcessTxHash);
    publicClient.getTransactionReceipt.mockResolvedValueOnce({ ...publishAndProcessTxReceipt, status: 'reverted' });

    const result = await publisher.processL2Block(l2Block);

    expect(result).toEqual(false);
  });

  it('returns false if process tx reverts', async () => {
    availabilityOracleRead.isAvailable.mockResolvedValueOnce(true);
    rollupContractRead.archive.mockResolvedValue(l2Block.header.lastArchive.root.toString() as `0x${string}`);

    publicClient.getTransactionReceipt.mockResolvedValueOnce({ ...processTxReceipt, status: 'reverted' });

    const result = await publisher.processL2Block(l2Block);

    expect(result).toEqual(false);
  });

  it('returns false if sending publish and progress tx is interrupted', async () => {
    rollupContractRead.archive.mockResolvedValue(l2Block.header.lastArchive.root.toString() as `0x${string}`);
    rollupContractWrite.publishAndProcess.mockImplementationOnce(
      () => sleep(10, publishAndProcessTxHash) as Promise<`0x${string}`>,
    );

    const resultPromise = publisher.processL2Block(l2Block);
    publisher.interrupt();
    const result = await resultPromise;

    expect(result).toEqual(false);
    expect(publicClient.getTransactionReceipt).not.toHaveBeenCalled();
  });

  it('returns false if sending process tx is interrupted', async () => {
    availabilityOracleRead.isAvailable.mockResolvedValueOnce(true);
    rollupContractRead.archive.mockResolvedValue(l2Block.header.lastArchive.root.toString() as `0x${string}`);
    rollupContractWrite.process.mockImplementationOnce(() => sleep(10, processTxHash) as Promise<`0x${string}`>);

    const resultPromise = publisher.processL2Block(l2Block);
    publisher.interrupt();
    const result = await resultPromise;

    expect(result).toEqual(false);
    expect(publicClient.getTransactionReceipt).not.toHaveBeenCalled();
  });
});
