import { jest } from '@jest/globals';
import { getAddress, PublicClient } from 'viem';
import { Archiver } from './archiver.js';

jest.mock('viem');

describe('Archiver', () => {
  const rollupAddress = getAddress('0x0000000000000000000000000000000000000000');
  const yeeterAddress = getAddress('0x0000000000000000000000000000000000000000');
  let publicClient: PublicClient;

  beforeEach(() => {
    publicClient = {
      readContract: jest.fn().mockReturnValue(3n),
      createEventFilter: jest.fn(),
      getFilterLogs: jest.fn().mockReturnValue([
        {
          args: {
            blockNum: 0n,
          },
        },
        {
          args: {
            blockNum: 1n,
          },
        },
        {
          args: {
            blockNum: 2n,
          },
        },
      ]),
      watchEvent: jest.fn().mockReturnValue(jest.fn()),
    } as unknown as PublicClient;
  });

  it('can start, sync and stop', async () => {
    const archiver = new Archiver(publicClient, rollupAddress, yeeterAddress);
    let syncStatus = await archiver.getSyncStatus();
    let latestBlockNum = archiver.getLatestBlockNum();
    expect(syncStatus).toStrictEqual({
      syncedToBlock: -1,
      latestBlock: 2,
    });
    expect(latestBlockNum).toBe(syncStatus.syncedToBlock);

    await archiver.start();

    syncStatus = await archiver.getSyncStatus();
    latestBlockNum = archiver.getLatestBlockNum();
    expect(syncStatus).toStrictEqual({
      syncedToBlock: 2,
      latestBlock: 2,
    });
    expect(latestBlockNum).toBe(syncStatus.syncedToBlock);

    archiver.stop();
  });
});
