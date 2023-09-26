import { AztecNodeConfig, AztecNodeService } from '@aztec/aztec-node';
import { ContractDeployer, SentTx, isContractDeployed } from '@aztec/aztec.js';
import { AztecAddress, CompleteAddress, Fr, PublicKey, getContractDeploymentInfo } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { DebugLogger } from '@aztec/foundation/log';
import { TestContractAbi } from '@aztec/noir-contracts/artifacts';
import { BootstrapNode, P2PConfig, createLibP2PPeerId } from '@aztec/p2p';
import { AztecRPCServer, ConstantKeyPair, createAztecRPCServer, getConfigEnvVars as getRpcConfig } from '@aztec/pxe';
import { TxStatus } from '@aztec/types';

import { setup } from './fixtures/utils.js';

const NUM_NODES = 4;
const NUM_TXS_PER_BLOCK = 4;
const NUM_TXS_PER_NODE = 2;
const BOOT_NODE_TCP_PORT = 40400;

interface NodeContext {
  node: AztecNodeService;
  rpcServer: AztecRPCServer;
  txs: SentTx[];
  account: AztecAddress;
}

describe('e2e_p2p_network', () => {
  let config: AztecNodeConfig;
  let logger: DebugLogger;
  let teardown: () => Promise<void>;
  beforeEach(async () => {
    ({ teardown, config, logger } = await setup(0));
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
      const node = await createNode(i + 1 + BOOT_NODE_TCP_PORT, bootstrapNodeAddress);
      const context = await createAztecRpcServerAndSubmitTransactions(node, NUM_TXS_PER_NODE);
      contexts.push(context);
    }

    // now ensure that all txs were successfully mined
    for (const context of contexts) {
      for (const tx of context.txs) {
        const isMined = await tx.isMined({ interval: 0.1 });
        const receiptAfterMined = await tx.getReceipt();

        expect(isMined).toBe(true);
        expect(receiptAfterMined.status).toBe(TxStatus.MINED);
        const contractAddress = receiptAfterMined.contractAddress!;
        expect(await isContractDeployed(context.rpcServer, contractAddress)).toBeTruthy();
        expect(await isContractDeployed(context.rpcServer, AztecAddress.random())).toBeFalsy();
      }
    }

    // shutdown all nodes.
    for (const context of contexts) {
      await context.node.stop();
      await context.rpcServer.stop();
    }
    await bootstrapNode.stop();
  }, 80_000);

  const createBootstrapNode = async () => {
    const peerId = await createLibP2PPeerId();
    const bootstrapNode = new BootstrapNode(logger);
    const config: P2PConfig = {
      p2pEnabled: true,
      tcpListenPort: BOOT_NODE_TCP_PORT,
      tcpListenIp: '0.0.0.0',
      announceHostname: '127.0.0.1',
      announcePort: BOOT_NODE_TCP_PORT,
      peerIdPrivateKey: Buffer.from(peerId.privateKey!).toString('hex'),
      serverMode: false,
      minPeerCount: 10,
      maxPeerCount: 100,

      // TODO: the following config options are not applicable to bootstrap nodes
      p2pBlockCheckIntervalMS: 1000,
      l2QueueSize: 1,
      transactionProtocol: '',
      bootstrapNodes: [''],
    };
    await bootstrapNode.start(config);

    return bootstrapNode;
  };

  // creates a P2P enabled instance of Aztec Node Service
  const createNode = async (tcpListenPort: number, bootstrapNode: string) => {
    const newConfig: AztecNodeConfig = {
      ...config,
      tcpListenPort,
      tcpListenIp: '0.0.0.0',
      enableNat: false,
      bootstrapNodes: [bootstrapNode],
      minTxsPerBlock: NUM_TXS_PER_BLOCK,
      maxTxsPerBlock: NUM_TXS_PER_BLOCK,
      p2pEnabled: true,
      serverMode: false,
    };
    return await AztecNodeService.createAndSync(newConfig);
  };

  // submits a set of transactions to the provided aztec rpc server
  const submitTxsTo = async (
    aztecRpcServer: AztecRPCServer,
    account: AztecAddress,
    numTxs: number,
    publicKey: PublicKey,
  ) => {
    const txs: SentTx[] = [];
    for (let i = 0; i < numTxs; i++) {
      const salt = Fr.random();
      const origin = (await getContractDeploymentInfo(TestContractAbi, [], salt, publicKey)).completeAddress.address;
      const deployer = new ContractDeployer(TestContractAbi, aztecRpcServer, publicKey);
      const tx = deployer.deploy().send({ contractAddressSalt: salt });
      logger(`Tx sent with hash ${await tx.getTxHash()}`);
      const receipt = await tx.getReceipt();
      expect(receipt).toEqual(
        expect.objectContaining({
          status: TxStatus.PENDING,
          error: '',
        }),
      );
      logger(`Receipt received and expecting contract deployment at ${origin}`);
      txs.push(tx);
    }
    return txs;
  };

  // creates an instance of the aztec rpc server and submit a given number of transactions to it.
  const createAztecRpcServerAndSubmitTransactions = async (
    node: AztecNodeService,
    numTxs: number,
  ): Promise<NodeContext> => {
    const rpcConfig = getRpcConfig();
    const aztecRpcServer = await createAztecRPCServer(node, rpcConfig, {}, true);

    const keyPair = ConstantKeyPair.random(await Grumpkin.new());
    const completeAddress = await CompleteAddress.fromPrivateKeyAndPartialAddress(
      await keyPair.getPrivateKey(),
      Fr.random(),
    );
    await aztecRpcServer.registerAccount(await keyPair.getPrivateKey(), completeAddress.partialAddress);

    const txs = await submitTxsTo(aztecRpcServer, completeAddress.address, numTxs, completeAddress.publicKey);
    return {
      txs,
      account: completeAddress.address,
      rpcServer: aztecRpcServer,
      node,
    };
  };
});
