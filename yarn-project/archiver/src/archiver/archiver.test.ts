import { EthAddress } from '@aztec/ethereum.js/eth_address';
import { RollupAbi, YeeterAbi } from '@aztec/l1-contracts/viem';
import { jest } from '@jest/globals';
import { mock, MockProxy } from 'jest-mock-extended';
import { encodeFunctionData, Log, PublicClient, toHex, Transaction } from 'viem';
import { Archiver, mockRandomL2Block } from './archiver.js';

describe('Archiver', () => {
  const rollupAddress = '0x0000000000000000000000000000000000000000';
  const yeeterAddress = '0x0000000000000000000000000000000000000000';
  let publicClient: MockProxy<PublicClient>;

  beforeEach(() => {
    publicClient = mock<PublicClient>();
  });

  it('can start, sync and stop', async () => {
    const archiver = new Archiver(
      publicClient,
      EthAddress.fromString(rollupAddress),
      EthAddress.fromString(yeeterAddress),
    );

    let latestBlockNum = await archiver.getLatestBlockNum();
    expect(latestBlockNum).toEqual(0);

    const rollupLogs = [1, 2, 3].map(makeRollupEvent);
    const rollupTxs = [1, 2, 3].map(makeRollupTx);
    const yeeterLogs = [] as Log<bigint, number, undefined, typeof YeeterAbi, 'Yeet'>[];

    publicClient.getFilterLogs.mockResolvedValueOnce(rollupLogs).mockResolvedValueOnce(yeeterLogs);
    rollupTxs.forEach(tx => publicClient.getTransaction.mockResolvedValueOnce(tx));
    publicClient.watchContractEvent.mockReturnValue(jest.fn());

    await archiver.start();

    latestBlockNum = await archiver.getLatestBlockNum();
    expect(latestBlockNum).toEqual(3);

    await archiver.stop();
  });
});

/**
 * Makes a fake rollup event for testing purposes.
 * @param blockNum - L2Block number.
 * @returns A rollup event log.
 */
function makeRollupEvent(blockNum: number) {
  return { args: { blockNum: BigInt(blockNum) }, transactionHash: `0x${blockNum}` } as unknown as Log<
    bigint,
    number,
    undefined,
    typeof RollupAbi,
    'L2BlockProcessed'
  >;
}

/**
 * Makes a fake rollup tx for testing purposes.
 * @param blockNum - L2Block number.
 * @returns A fake tx with calldata that corresponds to calling process in the Rollup contract.
 */
function makeRollupTx(blockNum: number) {
  const proof = `0x`;
  const block = toHex(mockRandomL2Block(blockNum).encode());
  const input = encodeFunctionData({ abi: RollupAbi, functionName: 'process', args: [proof, block] });
  return { input } as Transaction<bigint, number>;
}
