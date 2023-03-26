import { Socket } from './socket.js';

/**
 * Opens a socket with corresponding TransportListener.
 */
export interface Connector {
  createSocket(): Promise<Socket>;
}
