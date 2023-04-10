import { CircuitsWasm } from '@aztec/circuits.js';
import { computeContractLeaf } from '@aztec/circuits.js/abis';
import { EthereumRpc } from '@aztec/ethereum.js/eth_rpc';
import { WalletProvider } from '@aztec/ethereum.js/provider';
import { EthAddress, createDebugLogger, sleep } from '@aztec/foundation';
import { INITIAL_L2_BLOCK_NUM } from '@aztec/l1-contracts';
import { Tx } from '@aztec/tx';
import { AztecNode } from '../index.js';
import { AztecNodeConfig } from './config.js';
import { createProvider, createTx, deployRollupContract, deployUnverifiedDataEmitterContract } from './fixtures.js';

const ETHEREUM_HOST = 'http://127.0.0.1:8545/';
const MNEMONIC = 'test test test test test test test test test test test junk';

const logger = createDebugLogger('aztec:e2e_test');

describe('AztecNode', () => {
  let rollupAddress: EthAddress;
  let unverifiedDataEmitterAddress: EthAddress;
  let node: AztecNode;
  let isReady: boolean;
  let provider: WalletProvider;
  let wasm: CircuitsWasm;

  beforeAll(async () => {
    wasm = await CircuitsWasm.get();
  });

  beforeEach(async () => {
    provider = createProvider(ETHEREUM_HOST, MNEMONIC, 1);
    const ethRpc = new EthereumRpc(provider);
    rollupAddress = await deployRollupContract(provider, ethRpc);
    unverifiedDataEmitterAddress = await deployUnverifiedDataEmitterContract(provider, ethRpc);

    const aztecNodeConfig = {
      rollupContract: rollupAddress,
      unverifiedDataEmitterContract: unverifiedDataEmitterAddress,
      rpcUrl: ETHEREUM_HOST,
      publisherPrivateKey: provider.getPrivateKey(0),
      archiverPollingInterval: 100,
    } as AztecNodeConfig;

    node = await AztecNode.createAndSync(aztecNodeConfig);
    isReady = await node.isReady();
  });

  it('should start and stop all services', async () => {
    expect(isReady).toBeTruthy();
    await node.stop();
  });

  it('should allow sending and retrieval of txs from the underlying p2p pool', async () => {
    const tx: Tx = createTx();
    await node.sendTx(tx);
    const retrievedTx = await node.getPendingTxByHash(await tx.getTxHash());
    expect(tx).toEqual(retrievedTx);
    await node.stop();
  }, 30_000);

  it.only('should rollup a transaction', async () => {
    const tx: Tx = createTx();
    await node.sendTx(tx);

    const [settledBlock] = await waitForBlocks(1);

    expect(settledBlock.number).toBe(1);
    expect(settledBlock.newContracts).toHaveLength(1);
    expect(settledBlock.newContracts[0]).toEqual(computeContractLeaf(wasm, tx.data.end.newContracts[0]));

    const unverifiedDatas = await waitForUnverifiedData(1);
    expect(unverifiedDatas.length).toBe(1);

    await node.stop();
  }, 60_000);

  it('should rollup multiple transactions', async () => {
    const numTxs = 3;
    const txs: Tx[] = Array(numTxs).fill(0).map(createTx);
    for (let i = 0; i < txs.length; i++) {
      logger(`Sending tx ${i + 1} of ${txs.length}`);
      await node.sendTx(txs[i]);
    }
    const blocks = await waitForBlocks(numTxs);

    logger(`Received ${blocks.length} settled blocks`);

    for (let i = 0; i < numTxs; i++) {
      const tx = txs[i];
      const block = blocks[i];
      expect(block.number).toBe(i + 1);
      expect(block.newContracts.length).toBeGreaterThan(0);
      expect(block.newContracts[0]).toEqual(tx.data.end.newContracts[0].functionTreeRoot);
      logger(`Verified tx ${i + 1}`);
    }

    const unverifiedDatas = await waitForUnverifiedData(numTxs);
    expect(unverifiedDatas.length).toBe(numTxs);

    await node.stop();
  }, 60_000 /* timeout in ms */);

  const waitForBlocks = async (take: number) => {
    while (true) {
      const blocks = await node.getBlocks(INITIAL_L2_BLOCK_NUM, take);
      if (blocks.length < take) {
        await sleep(100);
        continue;
      }
      return blocks;
    }
  };

  const waitForUnverifiedData = async (take: number) => {
    while (true) {
      const unverifiedDatas = await node.getUnverifiedData(INITIAL_L2_BLOCK_NUM, take);
      if (unverifiedDatas.length < take) {
        await sleep(100);
        continue;
      }
      return unverifiedDatas;
    }
  };
});
