import { createAztecNodeClient } from '@aztec/circuit-types';
import { NULL_KEY } from '@aztec/ethereum';
import { type ServerList } from '@aztec/foundation/json-rpc/server';
import { type LogFn } from '@aztec/foundation/log';
import { createProvingJobSourceServer } from '@aztec/prover-client/prover-agent';
import {
  type ProverNodeConfig,
  createProverNode,
  createProverNodeRpcServer,
  proverNodeConfigMappings,
} from '@aztec/prover-node';
import {
  createAndStartTelemetryClient,
  getConfigEnvVars as getTelemetryClientConfig,
} from '@aztec/telemetry-client/start';

import { mnemonicToAccount } from 'viem/accounts';

import { extractL1ContractAddresses, extractRelevantOptions } from '../util.js';

export const startProverNode = async (
  options: any,
  signalHandlers: (() => Promise<void>)[],
  userLog: LogFn,
): Promise<ServerList> => {
  // Services that will be started in a single multi-rpc server
  const services: ServerList = [];

  if (options.node || options.sequencer || options.pxe || options.p2pBootstrap || options.txe) {
    userLog(`Starting a prover-node with --node, --sequencer, --pxe, --p2p-bootstrap, or --txe is not supported.`);
    process.exit(1);
  }

  const proverConfig = {
    ...extractRelevantOptions<ProverNodeConfig>(options, proverNodeConfigMappings, 'proverNode'),
    l1Contracts: extractL1ContractAddresses(options),
  };

  if (!options.archiver && !proverConfig.archiverUrl) {
    userLog('--archiver.archiverUrl is required to start a Prover Node without --archiver option');
    process.exit(1);
  }

  if (options.prover) {
    userLog(`Running prover node with local prover agent.`);
    proverConfig.proverAgentEnabled = true;
  } else {
    userLog(`Running prover node without local prover agent. Connect one or more prover agents to this node.`);
    proverConfig.proverAgentEnabled = false;
  }

  if (!proverConfig.publisherPrivateKey || proverConfig.publisherPrivateKey === NULL_KEY) {
    if (!options.l1Mnemonic) {
      userLog(`--l1-mnemonic is required to start a Prover Node without --node.publisherPrivateKey`);
      process.exit(1);
    }
    const hdAccount = mnemonicToAccount(options.l1Mnemonic);
    const privKey = hdAccount.getHdKey().privateKey;
    proverConfig.publisherPrivateKey = `0x${Buffer.from(privKey!).toString('hex')}`;
  }

  // TODO(palla/prover-node) L1 contract addresses should not silently default to zero,
  // they should be undefined if not set and fail loudly.
  // Load l1 contract addresses from aztec node if not set.
  const isRollupAddressSet =
    proverConfig.l1Contracts?.rollupAddress && !proverConfig.l1Contracts.rollupAddress.isZero();
  const nodeUrl = proverConfig.nodeUrl ?? proverConfig.txProviderNodeUrl;
  if (nodeUrl && !isRollupAddressSet) {
    userLog(`Loading L1 contract addresses from aztec node at ${nodeUrl}`);
    proverConfig.l1Contracts = await createAztecNodeClient(nodeUrl).getL1ContractAddresses();
  }

  const telemetry = createAndStartTelemetryClient(getTelemetryClientConfig());
  const proverNode = await createProverNode(proverConfig, { telemetry });

  services.push({ node: createProverNodeRpcServer(proverNode) });

  if (!options.prover) {
    const provingJobSource = createProvingJobSourceServer(proverNode.getProver().getProvingJobSource());
    services.push({ provingJobSource });
  }

  signalHandlers.push(proverNode.stop);

  // Automatically start proving unproven blocks
  proverNode.start();

  return services;
};
