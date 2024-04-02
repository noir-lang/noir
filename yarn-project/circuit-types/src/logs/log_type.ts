import { type EncryptedL2Log } from './encrypted_l2_log.js';
import { type UnencryptedL2Log } from './unencrypted_l2_log.js';

/**
 * Defines possible log types.
 */
export enum LogType {
  ENCRYPTED,
  UNENCRYPTED,
}

export type FromLogType<TLogType extends LogType> = TLogType extends LogType.ENCRYPTED
  ? EncryptedL2Log
  : UnencryptedL2Log;
