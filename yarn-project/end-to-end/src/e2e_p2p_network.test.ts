import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import { type AztecNodeConfig, AztecNodeService } from '@aztec/aztec-node';
import {
  type AztecAddress,
  CompleteAddress,
  type DebugLogger,
  Fr,
  Grumpkin,
  GrumpkinScalar,
  type SentTx,
  TxStatus,
} from '@aztec/aztec.js';
import { BootstrapNode, type P2PConfig, createLibP2PPeerId } from '@aztec/p2p';
import { ConstantKeyPair, type PXEService, createPXEService, getPXEServiceConfig as getRpcConfig } from '@aztec/pxe';

import { mnemonicToAccount } from 'viem/accounts';

import { MNEMONIC } from './fixtures/fixtures.js';
import { setup } from './fixtures/utils.js';

// Don't set this to a higher value than 9 because each node will use a different L1 publisher account and anvil seeds
const NUM_NODES = 4;
const NUM_TXS_PER_BLOCK = 4;
const NUM_TXS_PER_NODE = 2;
const BOOT_NODE_TCP_PORT = 40400;

interface NodeContext {
  node: AztecNodeService;
  pxeService: PXEService;
  txs: SentTx[];
  account: AztecAddress;
}

describe('e2e_p2p_network', () => {
  let config: AztecNodeConfig;
  let logger: DebugLogger;
  let teardown: () => Promise<void>;

  beforeEach(async () => {
    ({ teardown, config, logger } = await setup(1));
  }, 100_000);

  afterEach(() => teardown());

  it('should rollup txs from all peers', async () => {
    // create the bootstrap node for the network
    const bootstrapNode = await createBootstrapNode();
    const bootstrapNodeAddress = `/ip4/127.0.0.1/tcp/${BOOT_NODE_TCP_PORT}/p2p/${bootstrapNode
      .getPeerId()!
      .toString()}`;
    // create our network of nodes and submit txs into each of them
    // the number of txs per node and the number of txs per rollup
    // should be set so that the only way for rollups to be built
    // is if the txs are successfully gossiped around the nodes.
    const contexts: NodeContext[] = [];
    for (let i = 0; i < NUM_NODES; i++) {
      const node = await createNode(i + 1 + BOOT_NODE_TCP_PORT, bootstrapNodeAddress, i);
      const context = await createPXEServiceAndSubmitTransactions(node, NUM_TXS_PER_NODE);
      contexts.push(context);
    }

    // now ensure that all txs were successfully mined
    await Promise.all(contexts.flatMap(context => context.txs.map(tx => tx.wait())));

    // shutdown all nodes.
    for (const context of contexts) {
      await context.node.stop();
      await context.pxeService.stop();
    }
    await bootstrapNode.stop();
  }, 120_000);

  const createBootstrapNode = async () => {
    const peerId = await createLibP2PPeerId();
    const bootstrapNode = new BootstrapNode(logger);
    const config: P2PConfig = {
      p2pEnabled: true,
      tcpListenPort: BOOT_NODE_TCP_PORT,
      tcpListenIp: '0.0.0.0',
      announceHostname: '/tcp/127.0.0.1',
      announcePort: BOOT_NODE_TCP_PORT,
      peerIdPrivateKey: Buffer.from(peerId.privateKey!).toString('hex'),
      clientKADRouting: false,
      minPeerCount: 10,
      maxPeerCount: 100,

      // TODO: the following config options are not applicable to bootstrap nodes
      p2pBlockCheckIntervalMS: 1000,
      p2pL2QueueSize: 1,
      transactionProtocol: '',
      bootstrapNodes: [''],
    };
    await bootstrapNode.start(config);

    return bootstrapNode;
  };

  // creates a P2P enabled instance of Aztec Node Service
  const createNode = async (tcpListenPort: number, bootstrapNode: string, publisherAddressIndex: number) => {
    // We use different L1 publisher accounts in order to avoid duplicate tx nonces. We start from
    // publisherAddressIndex + 1 because index 0 was already used during test environment setup.
    const hdAccount = mnemonicToAccount(MNEMONIC, { addressIndex: publisherAddressIndex + 1 });
    const publisherPrivKey = Buffer.from(hdAccount.getHdKey().privateKey!);
    config.publisherPrivateKey = `0x${publisherPrivKey!.toString('hex')}`;

    const newConfig: AztecNodeConfig = {
      ...config,
      tcpListenPort,
      tcpListenIp: '0.0.0.0',
      enableNat: false,
      bootstrapNodes: [bootstrapNode],
      minTxsPerBlock: NUM_TXS_PER_BLOCK,
      maxTxsPerBlock: NUM_TXS_PER_BLOCK,
      p2pEnabled: true,
      clientKADRouting: false,
    };
    return await AztecNodeService.createAndSync(newConfig);
  };

  // submits a set of transactions to the provided Private eXecution Environment (PXE)
  const submitTxsTo = async (pxe: PXEService, account: AztecAddress, numTxs: number) => {
    const txs: SentTx[] = [];
    for (let i = 0; i < numTxs; i++) {
      const tx = await getSchnorrAccount(pxe, GrumpkinScalar.random(), GrumpkinScalar.random(), Fr.random()).deploy();
      logger(`Tx sent with hash ${await tx.getTxHash()}`);
      const receipt = await tx.getReceipt();
      expect(receipt).toEqual(
        expect.objectContaining({
          status: TxStatus.PENDING,
          error: '',
        }),
      );
      logger(`Receipt received`);
      txs.push(tx);
    }
    return txs;
  };

  // creates an instance of the PXE and submit a given number of transactions to it.
  const createPXEServiceAndSubmitTransactions = async (
    node: AztecNodeService,
    numTxs: number,
  ): Promise<NodeContext> => {
    const rpcConfig = getRpcConfig();
    const pxeService = await createPXEService(node, rpcConfig, true);

    const keyPair = ConstantKeyPair.random(new Grumpkin());
    const completeAddress = CompleteAddress.fromPrivateKeyAndPartialAddress(keyPair.getPrivateKey(), Fr.random());
    await pxeService.registerAccount(keyPair.getPrivateKey(), completeAddress.partialAddress);

    const txs = await submitTxsTo(pxeService, completeAddress.address, numTxs);
    return {
      txs,
      account: completeAddress.address,
      pxeService,
      node,
    };
  };
});
