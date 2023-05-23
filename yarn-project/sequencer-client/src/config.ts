import { SequencerConfig } from './sequencer/config.js';
import { PublisherConfig, TxSenderConfig } from './publisher/config.js';
import { EthAddress } from '@aztec/foundation/eth-address';

/**
 * Configuration settings for the SequencerClient.
 */
export type SequencerClientConfig = PublisherConfig & TxSenderConfig & SequencerConfig;

/**
 * Creates an instance of SequencerClientConfig out of environment variables using sensible defaults for integration testing if not set.
 */
export function getConfigEnvVars(): SequencerClientConfig {
  const {
    SEQ_PUBLISHER_PRIVATE_KEY,
    ETHEREUM_HOST,
    CHAIN_ID,
    SEQ_REQUIRED_CONFS,
    SEQ_RETRY_INTERVAL,
    SEQ_TX_POLLING_INTERVAL,
    SEQ_MAX_TX_PER_BLOCK,
    ROLLUP_CONTRACT_ADDRESS,
    INBOX_CONTRACT_ADDRESS,
    UNVERIFIED_DATA_EMITTER_ADDRESS,
  } = process.env;

  return {
    rpcUrl: ETHEREUM_HOST ? ETHEREUM_HOST : '',
    chainId: CHAIN_ID ? +CHAIN_ID : 31337, // 31337 is the default chain id for anvil
    requiredConfirmations: SEQ_REQUIRED_CONFS ? +SEQ_REQUIRED_CONFS : 1,
    retryIntervalMs: SEQ_RETRY_INTERVAL ? +SEQ_RETRY_INTERVAL : 1_000,
    transactionPollingInterval: SEQ_TX_POLLING_INTERVAL ? +SEQ_TX_POLLING_INTERVAL : 1_000,
    rollupContract: ROLLUP_CONTRACT_ADDRESS ? EthAddress.fromString(ROLLUP_CONTRACT_ADDRESS) : EthAddress.ZERO,
    inboxContract: INBOX_CONTRACT_ADDRESS ? EthAddress.fromString(INBOX_CONTRACT_ADDRESS) : EthAddress.ZERO,
    unverifiedDataEmitterContract: UNVERIFIED_DATA_EMITTER_ADDRESS
      ? EthAddress.fromString(UNVERIFIED_DATA_EMITTER_ADDRESS)
      : EthAddress.ZERO,
    publisherPrivateKey: Buffer.from(SEQ_PUBLISHER_PRIVATE_KEY || ''),
    maxTxsPerBlock: SEQ_MAX_TX_PER_BLOCK ? +SEQ_MAX_TX_PER_BLOCK : 32,
  };
}
