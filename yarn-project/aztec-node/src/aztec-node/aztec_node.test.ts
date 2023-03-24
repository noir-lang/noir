import { EthAddress } from '@aztec/ethereum.js/eth_address';
import { EthereumRpc } from '@aztec/ethereum.js/eth_rpc';
import { WalletProvider } from '@aztec/ethereum.js/provider';
import { Tx } from '@aztec/p2p';
import { AztecNodeConfig } from './config.js';
import { AztecNode } from '../index.js';
import { createProvider, createTx, deployRollupContract, deployYeeterContract } from './fixtures.js';
import { createDebugLogger, sleep } from '@aztec/foundation';

const ETHEREUM_HOST = 'http://localhost:8545/';
const MNEMONIC = 'test test test test test test test test test test test junk';

const logger = createDebugLogger('aztec:e2e_test');

describe('AztecNode', () => {
  let rollupAddress: EthAddress;
  let yeeterAddress: EthAddress;
  let node: AztecNode;
  let isReady: boolean;
  let provider: WalletProvider;

  beforeEach(async () => {
    provider = createProvider(ETHEREUM_HOST, MNEMONIC, 1);
    const ethRpc = new EthereumRpc(provider);
    rollupAddress = await deployRollupContract(provider, ethRpc);
    yeeterAddress = await deployYeeterContract(provider, ethRpc);

    const aztecNodeConfig = {
      rollupContract: rollupAddress,
      yeeterContract: yeeterAddress,
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

  it('should rollup a transaction', async () => {
    const tx: Tx = createTx();
    await node.sendTx(tx);

    const [settledBlock] = await waitForBlocks(1);

    expect(settledBlock.number).toBe(1);
    expect(settledBlock.newContracts.length).toBeGreaterThan(0);
    expect(settledBlock.newContracts[0]).toEqual(tx.data.end.newContracts[0].functionTreeRoot);

    await node.stop();
  }, 30_000);

  it('should rollup multiple transactions', async () => {
    const txs: Tx[] = Array(3).fill(0).map(createTx);
    for (let i = 0; i < txs.length; i++) {
      logger(`Sending tx ${i + 1} of ${txs.length}`);
      await node.sendTx(txs[i]);
    }
    const blocks = await waitForBlocks(3);

    logger(`Received ${blocks.length} settled blocks`);

    for (let i = 0; i < 3; i++) {
      const tx = txs[i];
      const block = blocks[i];
      expect(block.number).toBe(i + 1);
      expect(block.newContracts.length).toBeGreaterThan(0);
      expect(block.newContracts[0]).toEqual(tx.data.end.newContracts[0].functionTreeRoot);
      logger(`Verified tx ${i + 1}`);
    }

    await node.stop();
  }, 30_000 /* timeout in ms */);

  const waitForBlocks = async (take: number) => {
    while (true) {
      const blocks = await node.getBlocks(1, take);
      if (blocks.length < take) {
        await sleep(100);
        continue;
      }
      return blocks;
    }
  };
});
