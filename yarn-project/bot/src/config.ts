import { Fr } from '@aztec/circuits.js';
import {
  type ConfigMappingsType,
  booleanConfigHelper,
  getConfigFromMappings,
  getDefaultConfig,
  numberConfigHelper,
} from '@aztec/foundation/config';

export type BotConfig = {
  /** URL to the PXE for sending txs, or undefined if an in-proc PXE is used. */
  pxeUrl: string | undefined;
  /** Signing private key for the sender account. */
  senderPrivateKey: Fr;
  /** Encryption secret for a recipient account. */
  recipientEncryptionSecret: Fr;
  /** Salt for the token contract deployment. */
  tokenSalt: Fr;
  /** Every how many seconds should a new tx be sent. */
  txIntervalSeconds: number;
  /** How many private token transfers are executed per tx. */
  privateTransfersPerTx: number;
  /** How many public token transfers are executed per tx. */
  publicTransfersPerTx: number;
  /** How to handle fee payments. */
  feePaymentMethod: 'fee_juice' | 'none';
  /** True to not automatically setup or start the bot on initialization. */
  noStart: boolean;
  /** How long to wait for a tx to be mined before reporting an error. */
  txMinedWaitSeconds: number;
  /** Don't wait for transfer transactions. */
  noWaitForTransfers: boolean;
};

export const botConfigMappings: ConfigMappingsType<BotConfig> = {
  pxeUrl: {
    env: 'BOT_PXE_URL',
    description: 'URL to the PXE for sending txs, or undefined if an in-proc PXE is used.',
  },
  senderPrivateKey: {
    env: 'BOT_PRIVATE_KEY',
    description: 'Signing private key for the sender account.',
    parseEnv: (val: string) => Fr.fromString(val),
    defaultValue: Fr.random(),
  },
  recipientEncryptionSecret: {
    env: 'BOT_RECIPIENT_ENCRYPTION_SECRET',
    description: 'Encryption secret for a recipient account.',
    parseEnv: (val: string) => Fr.fromString(val),
    defaultValue: Fr.fromString('0xcafecafe'),
  },
  tokenSalt: {
    env: 'BOT_TOKEN_SALT',
    description: 'Salt for the token contract deployment.',
    parseEnv: (val: string) => Fr.fromString(val),
    defaultValue: Fr.fromString('1'),
  },
  txIntervalSeconds: {
    env: 'BOT_TX_INTERVAL_SECONDS',
    description: 'Every how many seconds should a new tx be sent.',
    ...numberConfigHelper(60),
  },
  privateTransfersPerTx: {
    env: 'BOT_PRIVATE_TRANSFERS_PER_TX',
    description: 'How many private token transfers are executed per tx.',
    ...numberConfigHelper(1),
  },
  publicTransfersPerTx: {
    env: 'BOT_PUBLIC_TRANSFERS_PER_TX',
    description: 'How many public token transfers are executed per tx.',
    ...numberConfigHelper(1),
  },
  feePaymentMethod: {
    env: 'BOT_FEE_PAYMENT_METHOD',
    description: 'How to handle fee payments. (Options: fee_juice, none)',
    parseEnv: val => (val as 'fee_juice' | 'none') || undefined,
    defaultValue: 'none',
  },
  noStart: {
    env: 'BOT_NO_START',
    description: 'True to not automatically setup or start the bot on initialization.',
    ...booleanConfigHelper(),
  },
  txMinedWaitSeconds: {
    env: 'BOT_TX_MINED_WAIT_SECONDS',
    description: 'How long to wait for a tx to be mined before reporting an error.',
    ...numberConfigHelper(180),
  },
  noWaitForTransfers: {
    env: 'BOT_NO_WAIT_FOR_TRANSFERS',
    description: "Don't wait for transfer transactions.",
    ...booleanConfigHelper(),
  },
};

export function getBotConfigFromEnv(): BotConfig {
  return getConfigFromMappings<BotConfig>(botConfigMappings);
}

export function getBotDefaultConfig(): BotConfig {
  return getDefaultConfig<BotConfig>(botConfigMappings);
}
