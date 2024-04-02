import { type Socket } from './socket.js';

/**
 * Opens a socket with corresponding TransportListener.
 */
export interface Connector {
  // eslint-disable-next-line jsdoc/require-jsdoc
  createSocket(): Promise<Socket>;
}
