import { EthAddress } from '@aztec/ethereum.js/eth_address';
import { randomBytes, toBufferBE } from '@aztec/foundation';
import { RollupAbi, YeeterAbi } from '@aztec/l1-contracts/viem';
import { L2Block } from '@aztec/l2-block';
import { jest } from '@jest/globals';
import { mock, MockProxy } from 'jest-mock-extended';
import { encodeFunctionData, Log, PublicClient, toHex, Transaction } from 'viem';
import { Archiver } from './archiver.js';

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
    let getLatestUnverifiedDataBlockNum = await archiver.getLatestUnverifiedDataBlockNum();
    expect(getLatestUnverifiedDataBlockNum).toEqual(0);

    const rollupLogs = [1, 2, 3].map(makeL2BlockProcessedEvent);
    const rollupTxs = [1, 2, 3].map(makeRollupTx);
    const yeeterLogs: Log<bigint, number, undefined, typeof YeeterAbi, 'Yeet'>[] = [1, 2].map(makeYeetEvent);

    publicClient.getFilterLogs.mockResolvedValueOnce(rollupLogs).mockResolvedValueOnce(yeeterLogs);
    rollupTxs.forEach(tx => publicClient.getTransaction.mockResolvedValueOnce(tx));
    publicClient.watchContractEvent.mockReturnValue(jest.fn());

    await archiver.start();

    latestBlockNum = await archiver.getLatestBlockNum();
    expect(latestBlockNum).toEqual(3);
    getLatestUnverifiedDataBlockNum = await archiver.getLatestUnverifiedDataBlockNum();
    expect(getLatestUnverifiedDataBlockNum).toEqual(2);

    await archiver.stop();
  });
});

/**
 * Makes a fake L2BlockProcessed event for testing purposes.
 * @param blockNum - L2Block number.
 * @returns An L2BlockProcessed event log.
 */
function makeL2BlockProcessedEvent(blockNum: number) {
  return { args: { blockNum: BigInt(blockNum) }, transactionHash: `0x${blockNum}` } as unknown as Log<
    bigint,
    number,
    undefined,
    typeof RollupAbi,
    'L2BlockProcessed'
  >;
}

/**
 * Makes a fake Yeet event for testing purposes.
 * @param blockNum - L2Block number.
 * @returns An Yeet event log.
 */
function makeYeetEvent(blockNum: number) {
  return {
    args: {
      l2blockNum: BigInt(blockNum),
      sender: EthAddress.random(),
      blabber: '0x' + createRandomUnverifiedData(16).toString('hex'),
    },
    transactionHash: `0x${blockNum}`,
  } as unknown as Log<bigint, number, undefined, typeof YeeterAbi, 'Yeet'>;
}

/**
 * Makes a fake rollup tx for testing purposes.
 * @param blockNum - L2Block number.
 * @returns A fake tx with calldata that corresponds to calling process in the Rollup contract.
 */
function makeRollupTx(blockNum: number) {
  const proof = `0x`;
  const block = toHex(L2Block.random(blockNum).encode());
  const input = encodeFunctionData({ abi: RollupAbi, functionName: 'process', args: [proof, block] });
  return { input } as Transaction<bigint, number>;
}

const createRandomEncryptedNotePreimage = () => {
  const encryptedNotePreimageBuf = randomBytes(144);
  return Buffer.concat([toBufferBE(BigInt(encryptedNotePreimageBuf.length), 4), encryptedNotePreimageBuf]);
};

const createRandomUnverifiedData = (numPreimages: number) => {
  const encryptedNotePreimageBuf = createRandomEncryptedNotePreimage();
  return Buffer.concat(Array(numPreimages).fill(encryptedNotePreimageBuf));
};
