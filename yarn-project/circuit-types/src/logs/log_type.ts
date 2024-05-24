import { type EncryptedL2Log } from './encrypted_l2_log.js';
import { type EncryptedL2NoteLog } from './encrypted_l2_note_log.js';
import { type UnencryptedL2Log } from './unencrypted_l2_log.js';

/**
 * Defines possible log types.
 */
export enum LogType {
  NOTEENCRYPTED,
  ENCRYPTED,
  UNENCRYPTED,
}

export type FromLogType<TLogType extends LogType> = TLogType extends LogType.UNENCRYPTED
  ? UnencryptedL2Log
  : TLogType extends LogType.ENCRYPTED
  ? EncryptedL2Log
  : EncryptedL2NoteLog;
