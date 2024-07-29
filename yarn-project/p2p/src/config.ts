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

  /**
   * If announceUdpAddress or announceTcpAddress are not provided, query for the IP address of the machine. Default is false.
   */
  queryForIp: boolean;

  /** How many blocks have to pass after a block is proven before its txs are deleted (zero to delete immediately once proven) */
  keepProvenTxsInPoolFor: number;
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
    P2P_TCP_LISTEN_ADDR,
    P2P_UDP_LISTEN_ADDR,
    P2P_TCP_ANNOUNCE_ADDR,
    P2P_UDP_ANNOUNCE_ADDR,
    PEER_ID_PRIVATE_KEY,
    BOOTSTRAP_NODES,
    P2P_NAT_ENABLED,
    P2P_MIN_PEERS,
    P2P_MAX_PEERS,
    DATA_DIRECTORY,
    TX_GOSSIP_VERSION,
    P2P_TX_PROTOCOL,
    P2P_QUERY_FOR_IP,
    P2P_TX_POOL_KEEP_PROVEN_FOR,
  } = process.env;
  // P2P listen & announce addresses passed in format: <IP_ADDRESS>:<PORT>
  // P2P announce multiaddrs passed in format: /ip4/<IP_ADDRESS>/<protocol>/<PORT>
  const envVars: P2PConfig = {
    tcpAnnounceAddress: P2P_TCP_ANNOUNCE_ADDR,
    udpAnnounceAddress: P2P_UDP_ANNOUNCE_ADDR,
    tcpListenAddress: P2P_TCP_LISTEN_ADDR || '0.0.0.0:40400',
    udpListenAddress: P2P_UDP_LISTEN_ADDR || '0.0.0.0:40400',
    p2pEnabled: P2P_ENABLED === 'true',
    p2pBlockCheckIntervalMS: P2P_BLOCK_CHECK_INTERVAL_MS ? +P2P_BLOCK_CHECK_INTERVAL_MS : 100,
    p2pPeerCheckIntervalMS: P2P_PEER_CHECK_INTERVAL_MS ? +P2P_PEER_CHECK_INTERVAL_MS : 1000,
    p2pL2QueueSize: P2P_L2_BLOCK_QUEUE_SIZE ? +P2P_L2_BLOCK_QUEUE_SIZE : 1000,
    peerIdPrivateKey: PEER_ID_PRIVATE_KEY,
    bootstrapNodes: BOOTSTRAP_NODES ? BOOTSTRAP_NODES.split(',') : [],
    transactionProtocol: P2P_TX_PROTOCOL ? P2P_TX_PROTOCOL : '/aztec/0.1.0',
    enableNat: P2P_NAT_ENABLED === 'true',
    minPeerCount: P2P_MIN_PEERS ? +P2P_MIN_PEERS : 10,
    maxPeerCount: P2P_MAX_PEERS ? +P2P_MAX_PEERS : 100,
    dataDirectory: DATA_DIRECTORY,
    txGossipVersion: TX_GOSSIP_VERSION ? new SemVer(TX_GOSSIP_VERSION) : new SemVer('0.1.0'),
    queryForIp: P2P_QUERY_FOR_IP === 'true',
    keepProvenTxsInPoolFor: P2P_TX_POOL_KEEP_PROVEN_FOR ? +P2P_TX_POOL_KEEP_PROVEN_FOR : 0,
  };
  return envVars;
}
