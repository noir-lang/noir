import { type ArchiverConfig, archiverConfigMappings } from '@aztec/archiver';
import { sequencerClientConfigMappings } from '@aztec/aztec-node';
import { botConfigMappings } from '@aztec/bot';
import {
  type ConfigMapping,
  booleanConfigHelper,
  filterConfigMappings,
  isBooleanConfigValue,
} from '@aztec/foundation/config';
import { bootnodeConfigMappings, p2pConfigMappings } from '@aztec/p2p';
import { proverClientConfigMappings } from '@aztec/prover-client';
import { proverNodeConfigMappings } from '@aztec/prover-node';
import { allPxeConfigMappings } from '@aztec/pxe';
import { telemetryClientConfigMappings } from '@aztec/telemetry-client/start';

import { defaultMnemonic } from '../sandbox.js';

// Define an interface for options
export interface AztecStartOption {
  flag: string;
  description: string;
  defaultValue: any | undefined;
  printDefault?: (val: any) => string;
  envVar: string | undefined;
  parseVal?: (val: string) => any;
}

export const getOptions = (namespace: string, configMappings: Record<string, ConfigMapping>) => {
  const options: AztecStartOption[] = [];
  for (const [key, { env, defaultValue: def, parseEnv, description, printDefault }] of Object.entries(configMappings)) {
    if (universalOptions.includes(key)) {
      continue;
    }
    const isBoolean = isBooleanConfigValue(configMappings, key as keyof typeof configMappings);
    options.push({
      flag: `--${namespace}.${key}${isBoolean ? '' : ' <value>'}`,
      description,
      defaultValue: def,
      printDefault,
      envVar: env,
      parseVal: parseEnv,
    });
  }
  return options;
};

// These are options used by multiple modules so should be inputted once
export const universalOptions = ['l1RpcUrl', 'l1ChainId', 'l1Contracts', 'p2pEnabled'];

// Define categories and options
export const aztecStartOptions: { [key: string]: AztecStartOption[] } = {
  SANDBOX: [
    {
      flag: '--sandbox',
      description: 'Starts Aztec Sandbox',
      defaultValue: undefined,
      envVar: undefined,
    },
    {
      flag: '--sandbox.testAccounts',
      description: 'Deploy test accounts on sandbox start',
      envVar: 'TEST_ACCOUNTS',
      ...booleanConfigHelper(true),
    },
    {
      flag: '--sandbox.enableGas',
      description: 'Enable gas on sandbox start',
      envVar: 'ENABLE_GAS',
      ...booleanConfigHelper(),
    },
  ],
  API: [
    {
      flag: '--port <value>',
      description: 'Port to run the Aztec Services on on',
      defaultValue: 8080,
      envVar: 'AZTEC_PORT',
      parseVal: val => parseInt(val, 10),
    },
    {
      flag: '--api-prefix <value>',
      description: 'Prefix for API routes on any service that is started',
      defaultValue: '',
      envVar: 'API_PREFIX',
    },
  ],
  ETHEREUM: [
    {
      flag: '--l1-rpc-url <value>',
      description: 'URL of the Ethereum RPC node that services will connect to',
      defaultValue: 'http://localhost:8545',
      envVar: 'ETHEREUM_HOST',
    },
    {
      flag: '--l1-chain-id <value>',
      description: 'The L1 chain ID',
      defaultValue: 31337,
      envVar: 'L1_CHAIN_ID',
      parseVal: val => parseInt(val, 10),
    },
    {
      flag: '--l1-mnemonic <value>',
      description: 'Mnemonic for L1 accounts. Will be used if no publisher private keys are provided',
      defaultValue: defaultMnemonic,
      envVar: 'MNEMONIC',
    },
  ],
  'L1 CONTRACT ADDRESSES': [
    {
      flag: '--rollup-address <value>',
      description: 'The deployed L1 rollup contract address',
      defaultValue: undefined,
      envVar: 'ROLLUP_CONTRACT_ADDRESS',
    },
    {
      flag: '--registry-address <value>',
      description: 'The deployed L1 registry contract address',
      defaultValue: undefined,
      envVar: 'REGISTRY_CONTRACT_ADDRESS',
    },
    {
      flag: '--inbox-address <value>',
      description: 'The deployed L1 -> L2 inbox contract address',
      defaultValue: undefined,
      envVar: 'INBOX_CONTRACT_ADDRESS',
    },
    {
      flag: '--outbox-address <value>',
      description: 'The deployed L2 -> L1 outbox contract address',
      defaultValue: undefined,
      envVar: 'OUTBOX_CONTRACT_ADDRESS',
    },
    {
      flag: '--availability-oracle-address <value>',
      description: 'The deployed L1 availability oracle contract address',
      defaultValue: undefined,
      envVar: 'AVAILABILITY_ORACLE_CONTRACT_ADDRESS',
    },
    {
      flag: '--fee-juice-address <value>',
      description: 'The deployed L1 Fee Juice contract address',
      defaultValue: undefined,
      envVar: 'FEE_JUICE_CONTRACT_ADDRESS',
    },
    {
      flag: '--fee-juice-portal-address <value>',
      description: 'The deployed L1 Fee Juice portal contract address',
      defaultValue: undefined,
      envVar: 'FEE_JUICE_PORTAL_CONTRACT_ADDRESS',
    },
  ],
  // We can't easily auto-generate node options as they're parts of modules defined below
  'AZTEC NODE': [
    {
      flag: '--node',
      description: 'Starts Aztec Node with options',
      defaultValue: undefined,
      envVar: undefined,
    },
    {
      flag: '--node.archiverUrl <value>',
      description: 'URL for an archiver service',
      defaultValue: undefined,
      envVar: 'ARCHIVER_URL',
    },
    {
      flag: '--node.dataDirectory <value>',
      description: 'Where to store node data. If not set, will store temporarily',
      defaultValue: undefined,
      envVar: 'NODE_DATA_DIRECTORY',
    },
    {
      flag: '--node.deployAztecContracts',
      description: 'Deploys L1 Aztec contracts before starting the node. Needs mnemonic or private key to be set',
      envVar: 'DEPLOY_AZTEC_CONTRACTS',
      ...booleanConfigHelper(),
    },
    {
      flag: '--node.assumeProvenUntilBlockNumber',
      description:
        'Cheats the rollup contract into assuming every block until this one is proven. Useful for speeding up bootstraps.',
      envVar: 'ASSUME_PROVEN_UNTIL_BLOCK_NUMBER',
      parseVal: (val: string) => parseInt(val, 10),
      defaultValue: 0,
    },
    {
      flag: '--node.publisherPrivateKey <value>',
      description: 'Private key of account for publishing L1 contracts',
      defaultValue: undefined,
      envVar: 'L1_PRIVATE_KEY',
    },
    {
      flag: '--node.l2QueueSize <value>',
      description: 'Size of queue of L2 blocks to store in world state',
      defaultValue: 1000,
      envVar: 'L2_QUEUE_SIZE',
      parseVal: val => parseInt(val, 10),
    },
    {
      flag: '--node.worldStateBlockCheckIntervalMS <value>',
      description: 'Frequency in which to check for blocks in ms',
      defaultValue: 100,
      envVar: 'WS_BLOCK_CHECK_INTERVAL_MS',
      parseVal: val => parseInt(val, 10),
    },
  ],
  'P2P SUBSYSTEM': [
    {
      flag: '--p2p-enabled',
      description: 'Enable P2P subsystem',
      envVar: 'P2P_ENABLED',
      ...booleanConfigHelper(),
    },
    ...getOptions('p2p', p2pConfigMappings),
  ],
  TELEMETRY: [...getOptions('tel', telemetryClientConfigMappings)],
  PXE: [
    {
      flag: '--pxe',
      description: 'Starts Aztec PXE with options',
      defaultValue: undefined,
      envVar: undefined,
    },
    ...getOptions('pxe', allPxeConfigMappings),
  ],
  ARCHIVER: [
    {
      flag: '--archiver',
      description: 'Starts Aztec Archiver with options',
      defaultValue: undefined,
      envVar: undefined,
    },
    // filter out archiverUrl as it's passed separately in --node & --prover-node
    ...getOptions('archiver', archiverConfigMappings).filter(opt => !opt.flag.includes('archiverUrl')),
  ],
  SEQUENCER: [
    {
      flag: '--sequencer',
      description: 'Starts Aztec Sequencer with options',
      defaultValue: undefined,
      envVar: undefined,
    },
    ...getOptions('sequencer', sequencerClientConfigMappings),
  ],
  'PROVER AGENT': [
    {
      flag: '--prover',
      description: 'Starts Aztec Prover Agent with options',
      defaultValue: undefined,
      envVar: undefined,
    },
    ...getOptions('prover', proverClientConfigMappings),
  ],
  'PROVER NODE': [
    {
      flag: '--prover-node',
      description: 'Starts Aztec Prover Node with options',
      defaultValue: undefined,
      envVar: undefined,
    },
    {
      flag: '--proverNode.archiverUrl <value>',
      description: 'URL for an archiver service',
      defaultValue: undefined,
      envVar: 'ARCHIVER_URL',
    },
    ...getOptions(
      'proverNode',
      // filter out archiver options from prover node options as they're passed separately in --archiver
      filterConfigMappings(proverNodeConfigMappings, Object.keys(archiverConfigMappings) as (keyof ArchiverConfig)[]),
    ),
  ],
  'P2P BOOTSTRAP': [
    {
      flag: '--p2p-bootstrap',
      description: 'Starts Aztec P2P Bootstrap with options',
      defaultValue: undefined,
      envVar: undefined,
    },
    ...getOptions('p2pBootstrap', bootnodeConfigMappings),
  ],
  BOT: [
    {
      flag: '--bot',
      description: 'Starts Aztec Bot with options',
      defaultValue: undefined,
      envVar: undefined,
    },
    ...getOptions('bot', botConfigMappings),
  ],
  TXE: [
    {
      flag: '--txe',
      description: 'Starts Aztec TXE with options',
      defaultValue: undefined,
      envVar: undefined,
    },
  ],
};
