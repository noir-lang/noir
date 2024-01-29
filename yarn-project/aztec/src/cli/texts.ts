const contractAddresses =
  'Aztec Contract Addresses:\n' +
  'rollupAddress:ROLLUP_CONTRACT_ADDRESS - string - The deployed L1 rollup contract address.\n' +
  'registryAddress:REGISTRY_CONTRACT_ADDRESS - string - The deployed L1 registry contract address.\n' +
  'inboxAddress:INBOX_CONTRACT_ADDRESS - string - The deployed L1 inbox contract address.\n' +
  'outboxAddress:OUTBOX_CONTRACT_ADDRESS - string - The deployed L1 outbox contract address.\n' +
  'contractDeploymentEmitterAddress:CONTRACT_DEPLOYMENT_EMITTER_ADDRESS - string - The deployed L1 contract deployment emitter contract address.\n';
const p2pOptions =
  'p2pBlockCheckIntervalMS:P2P_BLOCK_CHECK_INTERVAL_MS - number - The frequency in which to check for blocks. Default: 100\n' +
  'p2pL2QueueSize:P2P_L2_QUEUE_SIZE - number - Size of queue of L2 blocks to store. Default: 1000\n' +
  'tcpListenPort:TCP_LISTEN_PORT - number - The tcp port on which the P2P service should listen for connections. Default: 40400\n' +
  'tcpListenIp:TCP_LISTEN_IP - string - The tcp IP on which the P2P service should listen for connections. Default: 0.0.0.0\n' +
  'peerIdPrivateKey:PEER_ID_PRIVATE_KEY - string - An optional peer id private key. If blank, will generate a random key.\n' +
  'bootstrapNodes:BOOTSTRAP_NODES - string - A list of bootstrap peers to connect to.\n' +
  'announceHostname:P2P_ANNOUNCE_HOSTNAME - string - P2P Hostname to announce.\n' +
  'announcePort:P2P_ANNOUNCE_PORT - number - P2P Port to announce.\n' +
  'clientKADRouting:P2P_KAD_CLIENT - boolean - Optional specification to run as a client in the Kademlia routing protocol. Default: false\n' +
  'enableNat:P2P_NAT_ENABLED - boolean - Whether to enable NAT from libp2p (ignored for bootstrap node). Default: false\n' +
  'minPeerCount:P2P_MIN_PEERS - number - The minimum number of peers to connect to. Default: 10\n' +
  'maxPeerCount:P2P_MAX_PEERS - number - The maximum number of peers to connect to. Default: 100\n';

export const cliTexts = {
  node:
    'Starts Aztec Node with options.\n' +
    'Available options are listed below as cliProperty:ENV_VARIABLE_NAME.\n' +
    'rcpUrl:ETHEREUM_HOST - string - The host of the Ethereum node to connect to. Default: http://localhost:8545\n' +
    'archiverUrl:ARCHIVER_URL - string - A URL for an archiver service that the node will use.\n' +
    'p2pEnabled:P2P_ENABLED - boolean - A flag dictating whether the P2P subsystem should be enabled.\n\n' +
    'dataDirectory:DATA_DIRECTORY - string - Where to store node data. If not set, will store temporarily.\n' +
    'deployAztecContracts:DEPLOY_AZTEC_CONTRACTS - boolean - A flag dictating whether to deploy the Aztec contracts. Default: false\n' +
    'l2QueueSize:L2_QUEUE_SIZE - number - Size of queue of L2 blocks to store. Default: 1000\n' +
    'worldStateBlockCheckIntervalMS:WS_BLOCK_CHECK_INTERVAL_MS - number - The frequency in which to check for blocks in ms. Default: 100\n' +
    // Contract Addresses
    contractAddresses +
    // P2P Options
    'When P2P is enabled, the following options are available:\n' +
    p2pOptions,
  pxe:
    'Starts a PXE with options. If started additionally to --node, the PXE will attach to that node.' +
    'Available options are listed below as cliProperty:ENV_VARIABLE_NAME.\n' +
    'nodeUrl:AZTEC_NODE_URL - string - The URL of the Aztec Node to connect to.\n' +
    'port:PXE_PORT - number - The port on which the PXE should listen for connections. Default: 79\n' +
    'l2BlockPollingIntervalMS:PXE_BLOCK_POLLING_INTERVAL_MS - number - The frequency in which to check for blocks in ms. Default: 1000\n' +
    'l2StartingBlock:PXE_L2_STARTING_BLOCK - number - The block number from which to start polling. Default: 1\n' +
    'dataDirectory:PXE_DATA_DIRECTORY - string - Where to store PXE data. If not set, will store temporarily.\n',
  archiver:
    'Starts an Archiver with options. If started additionally to --node, the Archiver will attach to that node.' +
    'Available options are listed below as cliProperty:ENV_VARIABLE_NAME.\n' +
    'rcpUrl:ETHEREUM_HOST - string - The host of the Ethereum node to connect to. Default: http://localhost:8545\n' +
    'apiKey:API_KEY - string - The key for the ethereum node if necessary.\n' +
    'archiverPollingIntervalMS:ARCHIVER_POLLING_INTERVAL_MS - number - The polling interval in ms for retrieving new L2 blocks and encrypted logs. Default: 1000\n' +
    'viemPollingIntervalMS:ARCHIVER_VIEM_POLLING_INTERVAL_MS - number - The polling interval viem uses in ms. Default: 1000\n' +
    'dataDirectory:DATA_DIRECTORY - string - Optional dir to store data. If omitted will store temporarily.\n\n' +
    contractAddresses,
  sequencer:
    'Starts a Sequencer with options. If started additionally to --node, the Sequencer will attach to that node.\n' +
    'Available options are listed below as cliProperty:ENV_VARIABLE_NAME.\n' +
    'rcpUrl:ETHEREUM_HOST - string - The host of the Ethereum node to connect to. Default: http://localhost:8545\n' +
    'apiKey:API_KEY - string - The key for the ethereum node if necessary.\n' +
    'chainId:CHAIN_ID - number - The chain id of the ethereum host. Default: 31337\n' +
    'version:VERSION - number - The version of the Aztec rollup. Default: 1\n' +
    'publisherPrivateKey:SEQ_PUBLISHER_PRIVATE_KEY - string - The private key of the publisher. If not provided, will try to infer from default foundry test accounts.\n' +
    'requiredConfirmations:SEQ_REQUIRED_CONFIRMATIONS - number - The number of confirmations required before publishing a block. Default: 1\n' +
    'l1BlockPublishRetryIntervalMS:SEQ_PUBLISH_RETRY_INTERVAL_MS - number - The interval in ms to wait before retrying to publish a block. Default: 1000\n' +
    'transactionPollingIntervalMS:SEQ_TX_POLLING_INTERVAL_MS - number - The interval in ms to wait before polling for new transactions. Default: 1000\n' +
    contractAddresses,
  p2pBootstrap:
    'Starts a P2P bootstrap node with options.\n' +
    'Available options are listed below as cliProperty:ENV_VARIABLE_NAME.\n' +
    p2pOptions,
};
