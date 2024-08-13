import { aztecNodeConfigMappings, createAztecNodeRpcServer } from '@aztec/aztec-node';
import { type PXE } from '@aztec/circuit-types';
import { NULL_KEY } from '@aztec/ethereum';
import { type ServerList } from '@aztec/foundation/json-rpc/server';
import { type LogFn } from '@aztec/foundation/log';
import {
  type TelemetryClientConfig,
  createAndStartTelemetryClient,
  telemetryClientConfigMappings,
} from '@aztec/telemetry-client/start';

import { mnemonicToAccount, privateKeyToAccount } from 'viem/accounts';

import { createAztecNode, deployContractsToL1 } from '../../sandbox.js';
import { extractL1ContractAddresses, extractNamespacedOptions, extractRelevantOptions } from '../util.js';

export const startNode = async (
  options: any,
  signalHandlers: (() => Promise<void>)[],
  userLog: LogFn,
  // ): Promise<ServerList> => {
) => {
  // Services that will be started in a single multi-rpc server
  const services: ServerList = [];

  // options specifically namespaced with --node.<option>
  const nodeSpecificOptions = extractNamespacedOptions(options, 'node');
  // All options that are relevant to the Aztec Node
  const nodeConfig = {
    ...extractRelevantOptions(options, aztecNodeConfigMappings),
    l1Contracts: extractL1ContractAddresses(options),
  };

  if (options.proverNode) {
    userLog(`Running a Prover Node within a Node is not yet supported`);
    process.exit(1);
  }

  // Deploy contracts if needed
  if (nodeSpecificOptions.deployAztecContracts) {
    let account;
    if (nodeSpecificOptions.publisherPrivateKey) {
      account = privateKeyToAccount(nodeSpecificOptions.publisherPrivateKey);
    } else if (options.l1Mnemonic) {
      account = mnemonicToAccount(options.l1Mnemonic);
    } else {
      throw new Error('--node.publisherPrivateKey or --l1-mnemonic is required to deploy L1 contracts');
    }
    await deployContractsToL1(nodeConfig, account!, undefined, {
      assumeProvenUntilBlockNumber: nodeSpecificOptions.assumeProvenUntilBlockNumber,
    });
  }

  // if no publisher private key, then use l1Mnemonic
  if (!options.archiver) {
    // expect archiver url in node config
    const archiverUrl = nodeConfig.archiverUrl;
    if (!archiverUrl) {
      userLog('Archiver Service URL is required to start Aztec Node without --archiver option');
      throw new Error('Archiver Service URL is required to start Aztec Node without --archiver option');
    }
    nodeConfig.archiverUrl = archiverUrl;
  }

  if (!options.sequencer) {
    nodeConfig.disableSequencer = true;
  } else {
    const sequencerConfig = extractNamespacedOptions(options, 'sequencer');
    let account;
    if (!sequencerConfig.publisherPrivateKey || sequencerConfig.publisherPrivateKey === NULL_KEY) {
      if (!options.l1Mnemonic) {
        userLog(
          '--sequencer.publisherPrivateKey or --l1-mnemonic is required to start Aztec Node with --sequencer option',
        );
        throw new Error('Private key or Mnemonic is required to start Aztec Node with --sequencer option');
      } else {
        account = mnemonicToAccount(options.l1Mnemonic);
        const privKey = account.getHdKey().privateKey;
        nodeConfig.publisherPrivateKey = `0x${Buffer.from(privKey!).toString('hex')}`;
      }
    } else {
      nodeConfig.publisherPrivateKey = sequencerConfig.publisherPrivateKey;
    }
  }

  if (nodeConfig.p2pEnabled) {
    // ensure bootstrapNodes is an array
    if (nodeConfig.bootstrapNodes && typeof nodeConfig.bootstrapNodes === 'string') {
      nodeConfig.bootstrapNodes = (nodeConfig.bootstrapNodes as string).split(',');
    }
  }

  const telemetryConfig = extractRelevantOptions<TelemetryClientConfig>(options, telemetryClientConfigMappings);
  const telemetryClient = createAndStartTelemetryClient(telemetryConfig);

  // Create and start Aztec Node.
  const node = await createAztecNode(nodeConfig, telemetryClient);
  const nodeServer = createAztecNodeRpcServer(node);

  // Add node to services list
  services.push({ node: nodeServer });

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
