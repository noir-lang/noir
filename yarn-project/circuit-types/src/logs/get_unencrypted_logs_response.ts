import { type ExtendedUnencryptedL2Log } from './extended_unencrypted_l2_log.js';

/**
 * It provides documentation for the GetUnencryptedLogsResponse type.
 */
export type GetUnencryptedLogsResponse = {
  /**
   * An array of ExtendedUnencryptedL2Log elements.
   */
  logs: ExtendedUnencryptedL2Log[];

  /**
   * Indicates if a limit has been reached.
   */
  maxLogsHit: boolean;
};
