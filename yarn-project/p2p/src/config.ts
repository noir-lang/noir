import {
  type ConfigMappingsType,
  booleanConfigHelper,
  getConfigFromMappings,
  numberConfigHelper,
  pickConfigMappings,
} from '@aztec/foundation/config';

/**
 * P2P client configuration values.
 */
export interface P2PConfig {
  /**
   * A flag dictating whether the P2P subsystem should be enabled.
   */
  p2pEnabled: boolean;

  /**
   * The frequency in which to check for new L2 blocks.
   */
  blockCheckIntervalMS: number;

  /**
   * The frequency in which to check for new peers.
   */
  peerCheckIntervalMS: number;

  /**
   * Size of queue of L2 blocks to store.
   */
  l2QueueSize: number;

  /**
   * The announce address for TCP.
   */
  tcpAnnounceAddress?: string;

  /**
   * The announce address for UDP.
   */
  udpAnnounceAddress?: string;

  /**
   * The listen address for TCP.
   */
  tcpListenAddress: string;

  /**
   * The listen address for UDP.
   */
  udpListenAddress: string;

  /**
   * An optional peer id private key. If blank, will generate a random key.
   */
  peerIdPrivateKey?: string;

  /**
   * A list of bootstrap peers to connect to.
   */
  bootstrapNodes: string[];

  /**
   * Protocol identifier for transaction gossiping.
   */
  transactionProtocol: string;

  /**
   * The minimum number of peers (a peer count below this will cause the node to look for more peers)
   */
  minPeerCount: number;

  /**
   * The maximum number of peers (a peer count above this will cause the node to refuse connection attempts)
   */
  maxPeerCount: number;

  /**
   * Data directory for peer & tx databases.
   */
  dataDirectory?: string;

  /**
   * If announceUdpAddress or announceTcpAddress are not provided, query for the IP address of the machine. Default is false.
   */
  queryForIp: boolean;

  /** How many blocks have to pass after a block is proven before its txs are deleted (zero to delete immediately once proven) */
  keepProvenTxsInPoolFor: number;
}

export const p2pConfigMappings: ConfigMappingsType<P2PConfig> = {
  p2pEnabled: {
    env: 'P2P_ENABLED',
    description: 'A flag dictating whether the P2P subsystem should be enabled.',
    ...booleanConfigHelper(),
  },
  blockCheckIntervalMS: {
    env: 'P2P_BLOCK_CHECK_INTERVAL_MS',
    description: 'The frequency in which to check for new L2 blocks.',
    ...numberConfigHelper(100),
  },
  peerCheckIntervalMS: {
    env: 'P2P_PEER_CHECK_INTERVAL_MS',
    description: 'The frequency in which to check for new peers.',
    ...numberConfigHelper(1_000),
  },
  l2QueueSize: {
    env: 'P2P_L2_QUEUE_SIZE',
    description: 'Size of queue of L2 blocks to store.',
    ...numberConfigHelper(1_000),
  },
  tcpListenAddress: {
    env: 'TCP_LISTEN_ADDR',
    defaultValue: '0.0.0.0:40400',
    description: 'The listen address for TCP. Format: <IP_ADDRESS>:<PORT>.',
  },
  udpListenAddress: {
    env: 'UDP_LISTEN_ADDR',
    defaultValue: '0.0.0.0:40400',
    description: 'The listen address for UDP. Format: <IP_ADDRESS>:<PORT>.',
  },
  tcpAnnounceAddress: {
    env: 'P2P_TCP_ANNOUNCE_ADDR',
    description:
      'The announce address for TCP. Format: <IP_ADDRESS>:<PORT>. Leave IP_ADDRESS blank to query for public IP.',
  },
  udpAnnounceAddress: {
    env: 'P2P_UDP_ANNOUNCE_ADDR',
    description:
      'The announce address for UDP. Format: <IP_ADDRESS>:<PORT>. Leave IP_ADDRESS blank to query for public IP.',
  },
  peerIdPrivateKey: {
    env: 'PEER_ID_PRIVATE_KEY',
    description: 'An optional peer id private key. If blank, will generate a random key.',
  },
  bootstrapNodes: {
    env: 'BOOTSTRAP_NODES',
    parseEnv: (val: string) => val.split(','),
    description: 'A list of bootstrap peer ENRs to connect to. Separated by commas.',
  },
  transactionProtocol: {
    env: 'P2P_TX_PROTOCOL',
    description: 'Protocol identifier for transaction gossiping.',
    defaultValue: '/aztec/0.1.0',
  },
  minPeerCount: {
    env: 'P2P_MIN_PEERS',
    description: 'The minimum number of peers to connect to.',
    ...numberConfigHelper(10),
  },
  maxPeerCount: {
    env: 'P2P_MAX_PEERS',
    description: 'The maximum number of peers to connect to.',
    ...numberConfigHelper(100),
  },
  dataDirectory: {
    env: 'DATA_DIRECTORY',
    description: 'Data directory for peer & tx databases. Will use temporary location if not set.',
  },
  queryForIp: {
    env: 'P2P_QUERY_FOR_IP',
    description:
      'If announceUdpAddress or announceTcpAddress are not provided, query for the IP address of the machine. Default is false.',
    ...booleanConfigHelper(),
  },
  keepProvenTxsInPoolFor: {
    env: 'P2P_TX_POOL_KEEP_PROVEN_FOR',
    description:
      'How many blocks have to pass after a block is proven before its txs are deleted (zero to delete immediately once proven)',
    ...numberConfigHelper(0),
  },
};

/**
 * Gets the config values for p2p client from environment variables.
 * @returns The config values for p2p client.
 */
export function getP2PConfigEnvVars(): P2PConfig {
  return getConfigFromMappings<P2PConfig>(p2pConfigMappings);
}

/**
 * Required P2P config values for a bootstrap node.
 */
export type BootnodeConfig = Pick<
  P2PConfig,
  'udpAnnounceAddress' | 'peerIdPrivateKey' | 'minPeerCount' | 'maxPeerCount'
> &
  Required<Pick<P2PConfig, 'udpListenAddress'>>;

const bootnodeConfigKeys: (keyof BootnodeConfig)[] = [
  'udpAnnounceAddress',
  'peerIdPrivateKey',
  'minPeerCount',
  'maxPeerCount',
  'udpListenAddress',
];

export const bootnodeConfigMappings = pickConfigMappings(p2pConfigMappings, bootnodeConfigKeys);
