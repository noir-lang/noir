import {
  type AztecNodeConfig,
  createAztecNodeRpcServer,
  getConfigEnvVars as getNodeConfigEnvVars,
} from '@aztec/aztec-node';
import { type PXE } from '@aztec/circuit-types';
import { NULL_KEY } from '@aztec/ethereum';
import { type ServerList } from '@aztec/foundation/json-rpc/server';
import { type LogFn } from '@aztec/foundation/log';
import { createProvingJobSourceServer } from '@aztec/prover-client/prover-agent';
import {
  createAndStartTelemetryClient,
  getConfigEnvVars as getTelemetryClientConfig,
} from '@aztec/telemetry-client/start';

import { mnemonicToAccount, privateKeyToAccount } from 'viem/accounts';

import { MNEMONIC, createAztecNode, deployContractsToL1 } from '../../sandbox.js';
import { mergeEnvVarsAndCliOptions, parseModuleOptions } from '../util.js';

const { DEPLOY_AZTEC_CONTRACTS } = process.env;

export const startNode = async (
  options: any,
  signalHandlers: (() => Promise<void>)[],
  userLog: LogFn,
): Promise<ServerList> => {
  // Services that will be started in a single multi-rpc server
  const services: ServerList = [];
  // get env vars first
  const aztecNodeConfigEnvVars = getNodeConfigEnvVars();
  // get config from options
  const nodeCliOptions = parseModuleOptions(options.node);
  // merge env vars and cli options
  let nodeConfig = mergeEnvVarsAndCliOptions<AztecNodeConfig>(aztecNodeConfigEnvVars, nodeCliOptions);

  if (options.proverNode) {
    // TODO(palla/prover-node) We need to tweak the semantics of disableProver so that it doesn't inject
    // a null prover into the sequencer, but instead injects a circuit simulator, which is what the
    // sequencer ultimately needs.
    userLog(`Running a Prover Node within a Node is not yet supported`);
    process.exit(1);
  }

  // Deploy contracts if needed
  if (nodeCliOptions.deployAztecContracts || ['1', 'true'].includes(DEPLOY_AZTEC_CONTRACTS ?? '')) {
    const account =
      nodeConfig.publisherPrivateKey === NULL_KEY
        ? mnemonicToAccount(MNEMONIC)
        : privateKeyToAccount(nodeConfig.publisherPrivateKey);
    await deployContractsToL1(nodeConfig, account);
  }

  // if no publisher private key, then use MNEMONIC
  if (!options.archiver) {
    // expect archiver url in node config
    const archiverUrl = nodeCliOptions.archiverUrl;
    if (!archiverUrl) {
      userLog('Archiver Service URL is required to start Aztec Node without --archiver option');
      throw new Error('Archiver Service URL is required to start Aztec Node without --archiver option');
    }
    nodeConfig.archiverUrl = archiverUrl;
  } else {
    const archiverCliOptions = parseModuleOptions(options.archiver);
    nodeConfig = mergeEnvVarsAndCliOptions<AztecNodeConfig>(nodeConfig, archiverCliOptions, true);
  }

  if (!options.sequencer) {
    nodeConfig.disableSequencer = true;
  } else if (nodeConfig.publisherPrivateKey === NULL_KEY) {
    // If we have a sequencer, ensure there's a publisher private key set.
    const hdAccount = mnemonicToAccount(MNEMONIC);
    const privKey = hdAccount.getHdKey().privateKey;
    nodeConfig.publisherPrivateKey = `0x${Buffer.from(privKey!).toString('hex')}`;
  }

  if (!options.prover) {
    userLog(`Prover is disabled, using mocked proofs`);
    nodeConfig.disableProver = true;
  } else {
    nodeConfig = mergeEnvVarsAndCliOptions<AztecNodeConfig>(nodeConfig, parseModuleOptions(options.prover));
  }

  // ensure bootstrapNodes is an array
  if (nodeConfig.bootstrapNodes && typeof nodeConfig.bootstrapNodes === 'string') {
    nodeConfig.bootstrapNodes = (nodeConfig.bootstrapNodes as string).split(',');
  }

  if (!nodeConfig.disableSequencer && nodeConfig.disableProver) {
    // TODO(palla/prover-node) Sequencer should not need a prover unless we are running the prover
    // within it, it should just need a circuit simulator. We need to refactor the sequencer so it can accept either.
    throw new Error('Cannot run a sequencer without a prover');
  }

  // Create and start Aztec Node.
  const telemetryClient = createAndStartTelemetryClient(getTelemetryClientConfig());
  const node = await createAztecNode(telemetryClient, nodeConfig);
  const nodeServer = createAztecNodeRpcServer(node);

  // Add node to services list
  services.push({ node: nodeServer });

  if (!nodeConfig.disableProver) {
    const provingJobSource = createProvingJobSourceServer(node.getProver()!.getProvingJobSource());
    services.push({ provingJobSource });
  }

  // Add node stop function to signal handlers
  signalHandlers.push(node.stop);

  // Add a PXE client that connects to this node if requested
  let pxe: PXE | undefined;
  if (options.pxe) {
    const { addPXE } = await import('./start_pxe.js');
    pxe = await addPXE(options, services, signalHandlers, userLog, { node });
  }

  // Add a txs bot if requested
  if (options.bot) {
    const { addBot } = await import('./start_bot.js');
    await addBot(options, services, signalHandlers, { pxe });
  }

  return services;
};
