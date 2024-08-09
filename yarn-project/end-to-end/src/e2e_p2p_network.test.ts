import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import { type AztecNodeConfig, AztecNodeService } from '@aztec/aztec-node';
import {
  type AztecAddress,
  CompleteAddress,
  type DebugLogger,
  type DeployL1Contracts,
  EthCheatCodes,
  Fr,
  GrumpkinScalar,
  type SentTx,
  TxStatus,
  createDebugLogger,
  sleep,
} from '@aztec/aztec.js';
import { IS_DEV_NET } from '@aztec/circuits.js';
import { RollupAbi } from '@aztec/l1-artifacts';
import { type BootnodeConfig, BootstrapNode, createLibP2PPeerId } from '@aztec/p2p';
import { type PXEService, createPXEService, getPXEServiceConfig as getRpcConfig } from '@aztec/pxe';
import { NoopTelemetryClient } from '@aztec/telemetry-client/noop';

import fs from 'fs';
import { getContract } from 'viem';
import { mnemonicToAccount, privateKeyToAccount } from 'viem/accounts';

import { MNEMONIC } from './fixtures/fixtures.js';
import { setup } from './fixtures/utils.js';

// Don't set this to a higher value than 9 because each node will use a different L1 publisher account and anvil seeds
const NUM_NODES = 4;
const NUM_TXS_PER_BLOCK = 4;
const NUM_TXS_PER_NODE = 2;
const BOOT_NODE_UDP_PORT = 40400;

interface NodeContext {
  node: AztecNodeService;
  pxeService: PXEService;
  txs: SentTx[];
  account: AztecAddress;
}

const PEER_ID_PRIVATE_KEYS = [
  '0802122002f651fd8653925529e3baccb8489b3af4d7d9db440cbf5df4a63ff04ea69683',
  '08021220c3bd886df5fe5b33376096ad0dab3d2dc86ed2a361d5fde70f24d979dc73da41',
  '080212206b6567ac759db5434e79495ec7458e5e93fe479a5b80713446e0bce5439a5655',
  '08021220366453668099bdacdf08fab476ee1fced6bf00ddc1223d6c2ee626e7236fb526',
];

describe('e2e_p2p_network', () => {
  let config: AztecNodeConfig;
  let logger: DebugLogger;
  let teardown: () => Promise<void>;
  let bootstrapNode: BootstrapNode;
  let bootstrapNodeEnr: string;
  let deployL1ContractsValues: DeployL1Contracts;

  beforeEach(async () => {
    ({ teardown, config, logger, deployL1ContractsValues } = await setup(0));
    // It would likely be useful if we had the sequencers in such that they don't spam each other.
    // However, even if they do, it should still work. Not sure what caused the failure
    // Would be easier if I could see the errors from anvil as well, but those seem to be hidden.

    const rollup = getContract({
      address: deployL1ContractsValues.l1ContractAddresses.rollupAddress.toString(),
      abi: RollupAbi,
      client: deployL1ContractsValues.walletClient,
    });

    if (IS_DEV_NET) {
      // Add just ONE of the peers as sequencer, he will be the proposer all blocks.
      const hdAccount = mnemonicToAccount(MNEMONIC, { addressIndex: 1 });
      const publisherPrivKey = Buffer.from(hdAccount.getHdKey().privateKey!);
      const account = privateKeyToAccount(`0x${publisherPrivKey!.toString('hex')}`);
      await rollup.write.addValidator([account.address]);
      logger.info(`Adding sequencer ${account.address}`);
    } else {
      // @todo  Should be updated when we have attestations to add all the sequencers
      //        Since it is currently a mess because sequencer selection needs attestations for
      //        validity, but we currently have no way to collect them.
      //        When attestations works, add all 4, and lets ROLL!

      for (let i = 0; i < 1; i++) {
        const hdAccount = mnemonicToAccount(MNEMONIC, { addressIndex: i + 1 });
        const publisherPrivKey = Buffer.from(hdAccount.getHdKey().privateKey!);
        const account = privateKeyToAccount(`0x${publisherPrivKey!.toString('hex')}`);
        await rollup.write.addValidator([account.address]);
        logger.info(`Adding sequencer ${account.address}`);
      }
    }

    // Now we jump ahead to the next epoch, such that the next epoch begins
    const timeToJump = (await rollup.read.EPOCH_DURATION()) * (await rollup.read.SLOT_DURATION());

    const cheatCodes = new EthCheatCodes(config.l1RpcUrl);
    const timestamp = (await cheatCodes.timestamp()) + Number(timeToJump);
    await cheatCodes.warp(timestamp);

    bootstrapNode = await createBootstrapNode();
    bootstrapNodeEnr = bootstrapNode.getENR().encodeTxt();
  });

  afterEach(() => teardown());

  afterAll(() => {
    for (let i = 0; i < NUM_NODES; i++) {
      fs.rmSync(`./data-${i}`, { recursive: true, force: true });
    }
  });

  it('should rollup txs from all peers', async () => {
    // create the bootstrap node for the network
    if (!bootstrapNodeEnr) {
      throw new Error('Bootstrap node ENR is not available');
    }
    // create our network of nodes and submit txs into each of them
    // the number of txs per node and the number of txs per rollup
    // should be set so that the only way for rollups to be built
    // is if the txs are successfully gossiped around the nodes.
    const contexts: NodeContext[] = [];
    const nodes: AztecNodeService[] = [];
    for (let i = 0; i < NUM_NODES; i++) {
      const node = await createNode(i + 1 + BOOT_NODE_UDP_PORT, bootstrapNodeEnr, i);
      nodes.push(node);
    }

    // wait a bit for peers to discover each other
    await sleep(2000);

    for (const node of nodes) {
      const context = await createPXEServiceAndSubmitTransactions(node, NUM_TXS_PER_NODE);
      contexts.push(context);
    }

    // now ensure that all txs were successfully mined
    await Promise.all(
      contexts.flatMap((context, i) =>
        context.txs.map(async (tx, j) => {
          logger.info(`Waiting for tx ${i}-${j}: ${await tx.getTxHash()} to be mined`);
          return tx.wait();
        }),
      ),
    );

    // shutdown all nodes.
    for (const context of contexts) {
      await context.node.stop();
      await context.pxeService.stop();
    }
    await bootstrapNode.stop();
  });

  it('should re-discover stored peers without bootstrap node', async () => {
    const contexts: NodeContext[] = [];
    const nodes: AztecNodeService[] = [];
    for (let i = 0; i < NUM_NODES; i++) {
      const node = await createNode(i + 1 + BOOT_NODE_UDP_PORT, bootstrapNodeEnr, i, `./data-${i}`);
      nodes.push(node);
    }
    // wait a bit for peers to discover each other
    await sleep(3000);

    // stop bootstrap node
    await bootstrapNode.stop();

    // create new nodes from datadir
    const newNodes: AztecNodeService[] = [];

    // stop all nodes
    for (let i = 0; i < NUM_NODES; i++) {
      const node = nodes[i];
      await node.stop();
      logger.info(`Node ${i} stopped`);
      await sleep(1200);
      const newNode = await createNode(i + 1 + BOOT_NODE_UDP_PORT, undefined, i, `./data-${i}`);
      logger.info(`Node ${i} restarted`);
      newNodes.push(newNode);
      // const context = await createPXEServiceAndSubmitTransactions(node, NUM_TXS_PER_NODE);
      // contexts.push(context);
    }

    // wait a bit for peers to discover each other
    await sleep(2000);

    for (const node of newNodes) {
      const context = await createPXEServiceAndSubmitTransactions(node, NUM_TXS_PER_NODE);
      contexts.push(context);
    }

    // now ensure that all txs were successfully mined
    await Promise.all(
      contexts.flatMap((context, i) =>
        context.txs.map(async (tx, j) => {
          logger.info(`Waiting for tx ${i}-${j}: ${await tx.getTxHash()} to be mined`);
          return tx.wait();
        }),
      ),
    );

    // shutdown all nodes.
    // for (const context of contexts) {
    for (const context of contexts) {
      await context.node.stop();
      await context.pxeService.stop();
    }
  });

  const createBootstrapNode = async () => {
    const peerId = await createLibP2PPeerId();
    const bootstrapNode = new BootstrapNode();
    const config: BootnodeConfig = {
      udpListenAddress: `0.0.0.0:${BOOT_NODE_UDP_PORT}`,
      udpAnnounceAddress: `127.0.0.1:${BOOT_NODE_UDP_PORT}`,
      peerIdPrivateKey: Buffer.from(peerId.privateKey!).toString('hex'),
      minPeerCount: 10,
      maxPeerCount: 100,
    };
    await bootstrapNode.start(config);

    return bootstrapNode;
  };

  // creates a P2P enabled instance of Aztec Node Service
  const createNode = async (
    tcpListenPort: number,
    bootstrapNode: string | undefined,
    publisherAddressIndex: number,
    dataDirectory?: string,
  ) => {
    // We use different L1 publisher accounts in order to avoid duplicate tx nonces. We start from
    // publisherAddressIndex + 1 because index 0 was already used during test environment setup.
    const hdAccount = mnemonicToAccount(MNEMONIC, { addressIndex: publisherAddressIndex + 1 });
    const publisherPrivKey = Buffer.from(hdAccount.getHdKey().privateKey!);
    config.publisherPrivateKey = `0x${publisherPrivKey!.toString('hex')}`;

    const newConfig: AztecNodeConfig = {
      ...config,
      peerIdPrivateKey: PEER_ID_PRIVATE_KEYS[publisherAddressIndex],
      udpListenAddress: `0.0.0.0:${tcpListenPort}`,
      tcpListenAddress: `0.0.0.0:${tcpListenPort}`,
      tcpAnnounceAddress: `127.0.0.1:${tcpListenPort}`,
      udpAnnounceAddress: `127.0.0.1:${tcpListenPort}`,
      minTxsPerBlock: NUM_TXS_PER_BLOCK,
      maxTxsPerBlock: NUM_TXS_PER_BLOCK,
      p2pEnabled: true,
      blockCheckIntervalMS: 1000,
      l2QueueSize: 1,
      transactionProtocol: '',
      dataDirectory,
      bootstrapNodes: bootstrapNode ? [bootstrapNode] : [],
    };
    return await AztecNodeService.createAndSync(
      newConfig,
      new NoopTelemetryClient(),
      createDebugLogger(`aztec:node-${tcpListenPort}`),
    );
  };

  // creates an instance of the PXE and submit a given number of transactions to it.
  const createPXEServiceAndSubmitTransactions = async (
    node: AztecNodeService,
    numTxs: number,
  ): Promise<NodeContext> => {
    const rpcConfig = getRpcConfig();
    const pxeService = await createPXEService(node, rpcConfig, true);

    const secretKey = Fr.random();
    const completeAddress = CompleteAddress.fromSecretKeyAndPartialAddress(secretKey, Fr.random());
    await pxeService.registerAccount(secretKey, completeAddress.partialAddress);

    const txs = await submitTxsTo(pxeService, numTxs);
    return {
      txs,
      account: completeAddress.address,
      pxeService,
      node,
    };
  };

  // submits a set of transactions to the provided Private eXecution Environment (PXE)
  const submitTxsTo = async (pxe: PXEService, numTxs: number) => {
    const txs: SentTx[] = [];
    for (let i = 0; i < numTxs; i++) {
      // const tx = getSchnorrAccount(pxe, Fr.random(), GrumpkinScalar.random(), Fr.random()).deploy();
      const accountManager = getSchnorrAccount(pxe, Fr.random(), GrumpkinScalar.random(), Fr.random());
      const deployMethod = await accountManager.getDeployMethod();
      await deployMethod.create({
        contractAddressSalt: accountManager.salt,
        skipClassRegistration: true,
        skipPublicDeployment: true,
        universalDeploy: true,
      });
      await deployMethod.prove({});
      const tx = deployMethod.send();

      const txHash = await tx.getTxHash();

      logger.info(`Tx sent with hash ${txHash}`);
      const receipt = await tx.getReceipt();
      expect(receipt).toEqual(
        expect.objectContaining({
          status: TxStatus.PENDING,
          error: '',
        }),
      );
      logger.info(`Receipt received for ${txHash}`);
      txs.push(tx);
    }
    return txs;
  };
});
