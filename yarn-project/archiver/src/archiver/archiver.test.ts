import { RollupAbi, UnverifiedDataEmitterAbi } from '@aztec/l1-artifacts';
import { ContractData, ContractPublicData, EncodedContractFunction, L2Block } from '@aztec/types';
import { MockProxy, mock } from 'jest-mock-extended';
import { Chain, HttpTransport, Log, PublicClient, Transaction, encodeFunctionData, toHex } from 'viem';
import { Archiver } from './archiver.js';
import { EthAddress } from '@aztec/foundation/eth-address';
import { sleep } from '@aztec/foundation/sleep';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { randomBytes } from '@aztec/foundation/crypto';
import { toBufferBE } from '@aztec/foundation/bigint-buffer';
import { ArchiverDataStore, MemoryArchiverStore } from './archiver_store.js';

describe('Archiver', () => {
  const rollupAddress = '0x0000000000000000000000000000000000000000';
  const unverifiedDataEmitterAddress = '0x0000000000000000000000000000000000000001';
  let publicClient: MockProxy<PublicClient<HttpTransport, Chain>>;
  let archiverStore: ArchiverDataStore;

  beforeEach(() => {
    publicClient = mock<PublicClient<HttpTransport, Chain>>();
    archiverStore = new MemoryArchiverStore();
  });

  it('can start, sync and stop', async () => {
    const archiver = new Archiver(
      publicClient,
      EthAddress.fromString(rollupAddress),
      EthAddress.fromString(unverifiedDataEmitterAddress),
      archiverStore,
      1000,
    );

    let latestBlockNum = await archiver.getBlockHeight();
    expect(latestBlockNum).toEqual(0);
    let latestUnverifiedDataBlockNum = await archiver.getLatestUnverifiedDataBlockNum();
    expect(latestUnverifiedDataBlockNum).toEqual(0);

    const rollupTxs = [1, 2, 3].map(makeRollupTx);

    publicClient.getBlockNumber.mockResolvedValue(2500n);
    publicClient.getLogs
      .mockResolvedValueOnce([makeL2BlockProcessedEvent(100n, 1n)])
      .mockResolvedValueOnce([makeUnverifiedDataEvent(102n, 1n)])
      .mockResolvedValueOnce([makeContractDeployedEvent(104n, 1n)])
      .mockResolvedValueOnce([makeL2BlockProcessedEvent(1100n, 2n), makeL2BlockProcessedEvent(1150n, 3n)])
      .mockResolvedValueOnce([makeUnverifiedDataEvent(1100n, 2n)])
      .mockResolvedValueOnce([makeContractDeployedEvent(1102n, 2n)])
      .mockResolvedValue([]);
    rollupTxs.forEach(tx => publicClient.getTransaction.mockResolvedValueOnce(tx));

    await archiver.start(false);

    // Wait until block 3 is processed. If this won't happen the test will fail with timeout.
    while ((await archiver.getBlockHeight()) !== 3) {
      await sleep(100);
    }

    latestBlockNum = await archiver.getBlockHeight();
    expect(latestBlockNum).toEqual(3);

    // Wait until unverified data corresponding to block 2 is processed. If this won't happen the test will fail with
    // timeout.
    while ((await archiver.getLatestUnverifiedDataBlockNum()) !== 2) {
      await sleep(100);
    }
    latestUnverifiedDataBlockNum = await archiver.getLatestUnverifiedDataBlockNum();
    expect(latestUnverifiedDataBlockNum).toEqual(2);

    await archiver.stop();
  }, 10_000);
});

/**
 * Makes a fake L2BlockProcessed event for testing purposes.
 * @param l1BlockNum - L1 block number.
 * @param l2BlockNum - L2Block number.
 * @returns An L2BlockProcessed event log.
 */
function makeL2BlockProcessedEvent(l1BlockNum: bigint, l2BlockNum: bigint) {
  return {
    blockNumber: l1BlockNum,
    args: { blockNum: l2BlockNum },
    transactionHash: `0x${l2BlockNum}`,
  } as Log<bigint, number, undefined, typeof RollupAbi, 'L2BlockProcessed'>;
}

/**
 * Makes a fake UnverifiedData event for testing purposes.
 * @param l1BlockNum - L1 block number.
 * @param l2BlockNum - L2Block number.
 * @returns An UnverifiedData event log.
 */
function makeUnverifiedDataEvent(l1BlockNum: bigint, l2BlockNum: bigint) {
  return {
    blockNumber: l1BlockNum,
    args: {
      l2BlockNum,
      sender: EthAddress.random().toString(),
      data: '0x' + createRandomUnverifiedData(16).toString('hex'),
    },
    transactionHash: `0x${l2BlockNum}`,
  } as Log<bigint, number, undefined, typeof UnverifiedDataEmitterAbi, 'UnverifiedData'>;
}

/**
 * Makes a fake ContractDeployed event for testing purposes.
 * @param l1BlockNum - L1 block number.
 * @param l2BlockNum - L2Block number.
 * @returns An UnverifiedData event log.
 */
function makeContractDeployedEvent(l1BlockNum: bigint, l2BlockNum: bigint) {
  // const contractData = ContractData.random();
  const aztecAddress = AztecAddress.random();
  const portalAddress = EthAddress.random();
  const contractData = new ContractPublicData(new ContractData(aztecAddress, portalAddress), [
    EncodedContractFunction.random(),
    EncodedContractFunction.random(),
  ]);
  const acir = contractData.bytecode?.toString('hex');
  return {
    blockNumber: l1BlockNum,
    args: {
      l2BlockNum,
      aztecAddress: aztecAddress.toString(),
      portalAddress: portalAddress.toString(),
      acir: '0x' + acir,
    },
    transactionHash: `0x${l2BlockNum}`,
  } as Log<bigint, number, undefined, typeof UnverifiedDataEmitterAbi, 'ContractDeployment'>;
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

/**
 * Creates random encrypted note preimage.
 * @returns A random encrypted note preimage.
 */
const createRandomEncryptedNotePreimage = () => {
  const encryptedNotePreimageBuf = randomBytes(144);
  return Buffer.concat([toBufferBE(BigInt(encryptedNotePreimageBuf.length), 4), encryptedNotePreimageBuf]);
};

/**
 * Crate random unverified data.
 * @param numPreimages - Number of preimages to create.
 * @returns Unverified data containing `numPreimages` encrypted note preimages.
 */
const createRandomUnverifiedData = (numPreimages: number) => {
  const encryptedNotePreimageBuf = createRandomEncryptedNotePreimage();
  return Buffer.concat(Array(numPreimages).fill(encryptedNotePreimageBuf));
};
