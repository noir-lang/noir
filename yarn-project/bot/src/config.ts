import { Fr } from '@aztec/circuits.js';
import { compact } from '@aztec/foundation/collection';

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
  feePaymentMethod: 'native' | 'none';
  /** True to not automatically setup or start the bot on initialization. */
  noStart: boolean;
};

export function getBotConfigFromEnv(): BotConfig {
  const {
    BOT_FEE_PAYMENT_METHOD,
    BOT_PRIVATE_KEY,
    BOT_TOKEN_SALT,
    BOT_RECIPIENT_ENCRYPTION_SECRET,
    BOT_TX_INTERVAL_SECONDS,
    BOT_PRIVATE_TRANSFERS_PER_TX,
    BOT_PUBLIC_TRANSFERS_PER_TX,
    BOT_NO_START,
  } = process.env;
  if (BOT_FEE_PAYMENT_METHOD && !['native', 'none'].includes(BOT_FEE_PAYMENT_METHOD)) {
    throw new Error(`Invalid bot fee payment method: ${BOT_FEE_PAYMENT_METHOD}`);
  }

  return getBotDefaultConfig({
    pxeUrl: process.env.BOT_PXE_URL,
    senderPrivateKey: BOT_PRIVATE_KEY ? Fr.fromString(BOT_PRIVATE_KEY) : undefined,
    recipientEncryptionSecret: BOT_RECIPIENT_ENCRYPTION_SECRET
      ? Fr.fromString(BOT_RECIPIENT_ENCRYPTION_SECRET)
      : undefined,
    tokenSalt: BOT_TOKEN_SALT ? Fr.fromString(BOT_TOKEN_SALT) : undefined,
    txIntervalSeconds: BOT_TX_INTERVAL_SECONDS ? parseInt(BOT_TX_INTERVAL_SECONDS) : undefined,
    privateTransfersPerTx: BOT_PRIVATE_TRANSFERS_PER_TX ? parseInt(BOT_PRIVATE_TRANSFERS_PER_TX) : undefined,
    publicTransfersPerTx: BOT_PUBLIC_TRANSFERS_PER_TX ? parseInt(BOT_PUBLIC_TRANSFERS_PER_TX) : undefined,
    feePaymentMethod: BOT_FEE_PAYMENT_METHOD ? (BOT_FEE_PAYMENT_METHOD as 'native' | 'none') : undefined,
    noStart: BOT_NO_START ? ['1', 'true'].includes(BOT_NO_START) : undefined,
  });
}

export function getBotDefaultConfig(overrides: Partial<BotConfig> = {}): BotConfig {
  return {
    pxeUrl: undefined,
    senderPrivateKey: Fr.random(),
    recipientEncryptionSecret: Fr.fromString('0xcafecafe'),
    tokenSalt: Fr.fromString('1'),
    txIntervalSeconds: 60,
    privateTransfersPerTx: 1,
    publicTransfersPerTx: 1,
    feePaymentMethod: 'none',
    noStart: false,
    ...compact(overrides),
  };
}
