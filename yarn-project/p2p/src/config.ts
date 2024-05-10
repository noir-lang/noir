import { SemVer } from 'semver';

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
  p2pBlockCheckIntervalMS: number;

  /**
   * The frequency in which to check for new peers.
   */
  p2pPeerCheckIntervalMS: number;

  /**
   * Size of queue of L2 blocks to store.
   */
  p2pL2QueueSize: number;

  /**
   * The tcp port on which the P2P service should listen for connections.
   */
  tcpListenPort: number;

  /**
   * The tcp IP on which the P2P service should listen for connections.
   */
  tcpListenIp: string;

  /**
   * The udp port on which the P2P service should listen for connections. Used for Discv5 peer discovery.
   */
  udpListenPort: number;

  /**
   * The udp IP on which the P2P service should listen for connections. Used for Discv5 peer discovery.
   */
  udpListenIp: string;

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
   * Hostname to announce.
   */
  announceHostname?: string;

  /**
   * Port to announce.
   */
  announcePort?: number;

  /**
   * Whether to enable NAT from libp2p (ignored for bootstrap node).
   */
  enableNat?: boolean;

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
   * The transaction gossiping message version.
   */
  txGossipVersion: SemVer;
}

/**
 * Gets the config values for p2p client from environment variables.
 * @returns The config values for p2p client.
 */
export function getP2PConfigEnvVars(): P2PConfig {
  const {
    P2P_ENABLED,
    P2P_BLOCK_CHECK_INTERVAL_MS,
    P2P_PEER_CHECK_INTERVAL_MS,
    P2P_L2_BLOCK_QUEUE_SIZE,
    P2P_TCP_LISTEN_PORT,
    P2P_TCP_LISTEN_IP,
    P2P_UDP_LISTEN_PORT,
    P2P_UDP_LISTEN_IP,
    PEER_ID_PRIVATE_KEY,
    BOOTSTRAP_NODES,
    P2P_ANNOUNCE_HOSTNAME,
    P2P_ANNOUNCE_PORT,
    P2P_NAT_ENABLED,
    P2P_MIN_PEERS,
    P2P_MAX_PEERS,
    DATA_DIRECTORY,
    TX_GOSSIP_VERSION,
  } = process.env;
  const envVars: P2PConfig = {
    p2pEnabled: P2P_ENABLED === 'true',
    p2pBlockCheckIntervalMS: P2P_BLOCK_CHECK_INTERVAL_MS ? +P2P_BLOCK_CHECK_INTERVAL_MS : 100,
    p2pPeerCheckIntervalMS: P2P_PEER_CHECK_INTERVAL_MS ? +P2P_PEER_CHECK_INTERVAL_MS : 1000,
    p2pL2QueueSize: P2P_L2_BLOCK_QUEUE_SIZE ? +P2P_L2_BLOCK_QUEUE_SIZE : 1000,
    tcpListenPort: P2P_TCP_LISTEN_PORT ? +P2P_TCP_LISTEN_PORT : 40400,
    tcpListenIp: P2P_TCP_LISTEN_IP ? P2P_TCP_LISTEN_IP : '0.0.0.0',
    udpListenPort: P2P_UDP_LISTEN_PORT ? +P2P_UDP_LISTEN_PORT : 40400,
    udpListenIp: P2P_UDP_LISTEN_IP ? P2P_UDP_LISTEN_IP : '0.0.0.0',
    peerIdPrivateKey: PEER_ID_PRIVATE_KEY,
    bootstrapNodes: BOOTSTRAP_NODES ? BOOTSTRAP_NODES.split(',') : [],
    transactionProtocol: '',
    announceHostname: P2P_ANNOUNCE_HOSTNAME,
    announcePort: P2P_ANNOUNCE_PORT ? +P2P_ANNOUNCE_PORT : undefined,
    enableNat: P2P_NAT_ENABLED === 'true',
    minPeerCount: P2P_MIN_PEERS ? +P2P_MIN_PEERS : 10,
    maxPeerCount: P2P_MAX_PEERS ? +P2P_MAX_PEERS : 100,
    dataDirectory: DATA_DIRECTORY,
    txGossipVersion: TX_GOSSIP_VERSION ? new SemVer(TX_GOSSIP_VERSION) : new SemVer('0.1.0'),
  };
  return envVars;
}
